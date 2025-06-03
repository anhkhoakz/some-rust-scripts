//! A command-line tool for pasting text to https://snip.dssr.ch/
//!
//! This tool allows you to paste text to a PrivateBin instance with various options:
//! - Password protection
//! - File attachments
//! - Syntax highlighting
//! - Markdown support
//! - Burn after reading
//! - Open discussion
//!
//! The tool uses client-side encryption to ensure the privacy of your pastes.

use base64::{Engine as _, engine::general_purpose};
use clap::{Arg, Command};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use rand::Rng;
use reqwest::blocking::Client;
use ring::aead::{AES_256_GCM, Aad, LessSafeKey, Nonce, UnboundKey};
use ring::pbkdf2;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::io::{self, Read};
use std::num::NonZeroU32;
use std::path::Path;
use syntect::parsing::SyntaxSet;

/// Default configuration values
const DEFAULT_PASTE_URL: &str = "https://snip.dssr.ch/";
const DEFAULT_EXPIRE: &str = "1month";
const DEFAULT_FORMATTER: &str = "plaintext";
const DEFAULT_BURN: bool = false;
const DEFAULT_OPEN_DISCUSSION: bool = false;
const DEFAULT_PASSWORD: &str = "";
const DEFAULT_SOURCE: bool = false;
const KDF_ITERATIONS: u32 = 100_000;
const KDF_KEYSIZE: u32 = 256;
const ADATA_SIZE: u32 = 96; // 12 bytes for IV (96 bits)

/// Error types that can occur during paste operations
#[derive(Debug)]
enum PasteError {
    /// No data was provided to paste
    NoData,
    /// Error opening or reading a file
    FileOpen(String, Option<String>),
    /// Error communicating with the paste server
    ApiError(String, Option<u16>, Option<serde_json::Value>),
    /// Invalid response from the server
    InvalidResponse(String),
    /// Invalid command line options
    InvalidOptions(String),
    /// Error detecting language for syntax highlighting
    LanguageDetection(String),
}

impl std::fmt::Display for PasteError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PasteError::NoData => write!(f, "Error: No data provided to paste"),
            PasteError::FileOpen(err, path) => {
                if let Some(p) = path {
                    write!(f, "Error: Could not open file {}: {}", p, err)
                } else {
                    write!(f, "Error: Could not open file: {}", err)
                }
            }
            PasteError::ApiError(msg, status, response) => {
                write!(
                    f,
                    "Error: API request failed: {} {:?} {:?}",
                    msg, status, response
                )
            }
            PasteError::InvalidResponse(err) => {
                write!(f, "Error: Invalid response from server: {}", err)
            }
            PasteError::InvalidOptions(msg) => write!(f, "Error: {}", msg),
            PasteError::LanguageDetection(err) => {
                write!(f, "Error: Language detection failed: {}", err)
            }
        }
    }
}

/// Structure for paste data
#[derive(Serialize)]
struct PasteData {
    /// The content to paste
    paste: Option<String>,
    /// Base64 encoded attachment data
    #[serde(skip_serializing_if = "Option::is_none")]
    attachment: Option<String>,
    /// Name of the attached file
    #[serde(skip_serializing_if = "Option::is_none")]
    attachment_name: Option<String>,
}

/// Structure for associated data used in encryption
#[derive(Serialize, Deserialize)]
struct PasteAdata {
    /// Encryption parameters
    cipher: [String; 8],
    /// Formatting type
    formatter: String,
    /// Whether discussion is enabled
    opendiscussion: u8,
    /// Whether to burn after reading
    burn: u8,
}

/// Structure for the paste payload
#[derive(Serialize)]
struct Payload {
    /// Version number
    v: u8,
    /// Associated data
    adata: PasteAdata,
    /// Encrypted content
    ct: String,
    /// Paste metadata
    meta: PasteMeta,
}

/// Structure for paste metadata
#[derive(Serialize)]
struct PasteMeta {
    /// Expiration time
    expire: String,
}

/// Structure for server response
#[derive(Deserialize)]
struct PasteResponse {
    /// Status code
    status: Option<u8>,
    /// Paste ID
    id: Option<String>,
    /// Paste URL
    url: Option<String>,
    /// Token for deleting the paste
    deletetoken: Option<String>,
    /// Error message if any
    #[serde(rename = "message")]
    error_message: Option<String>,
}

/// Encode bytes to base58 string
fn base58_encode(v: &[u8]) -> String {
    let alphabet = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let mut x: u128 = 0;
    let mut n_pad = v.len();
    let v = v.iter().skip_while(|&&b| b == 0).collect::<Vec<_>>();
    n_pad -= v.len();

    for (i, &c) in v.iter().rev().enumerate() {
        x += (*c as u128) << (8 * i);
    }

    let mut result = String::new();
    while x > 0 {
        let idx = (x % alphabet.len() as u128) as usize;
        result.insert(0, alphabet[idx] as char);
        x /= alphabet.len() as u128;
    }

    String::from_utf8(vec![alphabet[0]; n_pad]).unwrap() + &result
}

/// Generate a key using PBKDF2
fn generate_kdf_key(passphrase: &[u8]) -> Result<(Vec<u8>, Vec<u8>), PasteError> {
    let mut kdf_salt = vec![0u8; 8];
    rand::rng().fill(&mut kdf_salt[..]);

    let mut key = vec![0u8; (KDF_KEYSIZE / 8) as usize];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        NonZeroU32::new(KDF_ITERATIONS).unwrap(),
        &kdf_salt,
        passphrase,
        &mut key,
    );
    Ok((key, kdf_salt))
}

/// Prepare paste data for encryption
fn prepare_paste_data(
    plaintext: Option<&str>,
    attachment_name: Option<&str>,
    attachment: Option<&str>,
    compress: bool,
) -> Result<Vec<u8>, PasteError> {
    let paste_data = PasteData {
        paste: plaintext.map(String::from),
        attachment: attachment.map(String::from),
        attachment_name: attachment_name.map(String::from),
    };

    // Convert to JSON with no whitespace, matching Python's json.dumps(separators=(',', ':'))
    let json_data =
        serde_json::to_vec(&paste_data).map_err(|e| PasteError::InvalidResponse(e.to_string()))?;

    if compress {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(&json_data)
            .map_err(|e| PasteError::InvalidResponse(e.to_string()))?;
        encoder
            .finish()
            .map_err(|e| PasteError::InvalidResponse(e.to_string()))
    } else {
        Ok(json_data)
    }
}

/// Prepare associated data for encryption
fn prepare_associated_data(
    cipher_iv: &[u8],
    kdf_salt: &[u8],
    formatter: &str,
    opendiscussion: bool,
    burn: bool,
    compression_type: &str,
) -> Result<Vec<u8>, PasteError> {
    let adata = PasteAdata {
        cipher: [
            general_purpose::STANDARD.encode(cipher_iv),
            general_purpose::STANDARD.encode(kdf_salt),
            KDF_ITERATIONS.to_string(),
            KDF_KEYSIZE.to_string(),
            ADATA_SIZE.to_string(),
            String::from("aes"),
            String::from("gcm"),
            String::from(compression_type),
        ],
        formatter: String::from(formatter),
        opendiscussion: opendiscussion as u8,
        burn: burn as u8,
    };

    // Convert to JSON with no whitespace, matching Python's json.dumps(separators=(',', ':'))
    serde_json::to_vec(&adata).map_err(|e| PasteError::InvalidResponse(e.to_string()))
}

/// Encrypt paste data using PrivateBin's encryption format
fn privatebin_encrypt(
    passphrase: &[u8],
    password: Option<&str>,
    plaintext: Option<&str>,
    formatter: &str,
    attachment_name: Option<&str>,
    attachment: Option<&str>,
    compress: bool,
    burn: bool,
    opendiscussion: bool,
) -> Result<(PasteAdata, String), PasteError> {
    let final_passphrase = if let Some(pwd) = password {
        let mut combined = passphrase.to_vec();
        combined.extend_from_slice(pwd.as_bytes());
        combined
    } else {
        passphrase.to_vec()
    };

    let (kdf_key, kdf_salt) = generate_kdf_key(&final_passphrase)?;

    // Generate a random IV for AES-GCM
    let mut cipher_iv = vec![0u8; 12]; // AES-GCM uses 12 bytes for IV
    rand::rng().fill(&mut cipher_iv[..]);

    let compression_type = if compress { "zlib" } else { "none" };
    let paste_blob = prepare_paste_data(plaintext, attachment_name, attachment, compress)?;
    let paste_adata = prepare_associated_data(
        &cipher_iv,
        &kdf_salt,
        formatter,
        opendiscussion,
        burn,
        compression_type,
    )?;

    let unbound_key = UnboundKey::new(&AES_256_GCM, &kdf_key)
        .map_err(|_| PasteError::InvalidResponse("Failed to create AES key".to_string()))?;
    let key = LessSafeKey::new(unbound_key);

    let mut data = paste_blob;
    let nonce = Nonce::try_assume_unique_for_key(&cipher_iv)
        .map_err(|_| PasteError::InvalidResponse("Invalid nonce".to_string()))?;

    key.seal_in_place_append_tag(nonce, Aad::from(&paste_adata), &mut data)
        .map_err(|_| PasteError::InvalidResponse("Encryption failed".to_string()))?;

    let ciphertext = general_purpose::STANDARD.encode(&data);
    let adata: PasteAdata = serde_json::from_slice(&paste_adata)
        .map_err(|e| PasteError::InvalidResponse(e.to_string()))?;

    Ok((adata, ciphertext))
}

/// Send encrypted paste to PrivateBin server
fn privatebin_send(
    url: &str,
    password: Option<&str>,
    plaintext: Option<&str>,
    formatter: &str,
    attachment_name: Option<&str>,
    attachment: Option<&str>,
    compress: bool,
    burn: bool,
    opendiscussion: bool,
    expire: &str,
) -> Result<(String, String, String, String), PasteError> {
    // Validate inputs
    if plaintext.is_none() && attachment.is_none() {
        return Err(PasteError::InvalidOptions(
            "No content to paste".to_string(),
        ));
    }

    let mut rng = rand::rng();
    let mut passphrase = [0u8; 32];
    rng.fill(&mut passphrase);
    let (adata, ciphertext) = privatebin_encrypt(
        &passphrase,
        password,
        plaintext,
        formatter,
        attachment_name,
        attachment,
        compress,
        burn,
        opendiscussion,
    )?;

    let payload = Payload {
        v: 2,
        adata,
        ct: ciphertext,
        meta: PasteMeta {
            expire: expire.to_string(),
        },
    };

    // Validate URL
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(PasteError::InvalidOptions(format!("Invalid URL: {}", url)));
    }

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("rpaste/0.1.1")
        .build()
        .map_err(|e| PasteError::ApiError(e.to_string(), None, None))?;

    // Make the request
    let response = client
        .post(url)
        .header("X-Requested-With", "JSONHttpRequest")
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .map_err(|e| PasteError::ApiError(e.to_string(), None, None))?;

    let status = response.status().as_u16();

    // Get the response text first for better error reporting
    let response_text = response
        .text()
        .map_err(|e| PasteError::InvalidResponse(format!("Failed to read response: {}", e)))?;

    // Check for empty response
    if response_text.trim().is_empty() {
        let error_msg = match status {
            500 => String::from(
                "Server internal error (500) - The server encountered an error processing your request",
            ),
            503 => String::from(
                "Service unavailable (503) - The server is temporarily unable to handle your request",
            ),
            502 => String::from(
                "Bad gateway (502) - The server received an invalid response from upstream",
            ),
            _ => String::from("Empty response from server"),
        };
        return Err(PasteError::ApiError(
            format!("{} (Status: {})", error_msg, status),
            Some(status),
            None,
        ));
    }

    // Try to parse the response as JSON
    let result: PasteResponse = serde_json::from_str(&response_text).map_err(|e| {
        PasteError::InvalidResponse(format!(
            "Failed to parse response as JSON: {}\nResponse body: {}",
            e, response_text
        ))
    })?;

    if let Some(status_code) = result.status {
        if status_code != 0 {
            let error_msg = match status {
                500 => String::from(
                    "Server internal error (500) - The server encountered an error processing your request",
                ),
                503 => String::from(
                    "Service unavailable (503) - The server is temporarily unable to handle your request",
                ),
                502 => String::from(
                    "Bad gateway (502) - The server received an invalid response from upstream",
                ),
                _ => result
                    .error_message
                    .unwrap_or_else(|| String::from("Unknown error")),
            };
            return Err(PasteError::ApiError(
                error_msg,
                Some(status),
                Some(serde_json::to_value(&response_text).unwrap_or_default()),
            ));
        }
    }

    Ok((
        result
            .id
            .ok_or_else(|| PasteError::InvalidResponse("Missing ID".to_string()))?,
        result
            .url
            .ok_or_else(|| PasteError::InvalidResponse("Missing URL".to_string()))?,
        result
            .deletetoken
            .ok_or_else(|| PasteError::InvalidResponse("Missing delete token".to_string()))?,
        base58_encode(&passphrase),
    ))
}

/// Handle file attachment
fn handle_attachment(path: &str) -> Result<(String, String), PasteError> {
    let path = Path::new(path);
    let attachment_name = path
        .file_name()
        .ok_or_else(|| {
            PasteError::FileOpen(
                "Invalid path".to_string(),
                Some(path.to_string_lossy().into_owned()),
            )
        })?
        .to_string_lossy()
        .into_owned();

    let data = fs::read(path).map_err(|e| {
        PasteError::FileOpen(e.to_string(), Some(path.to_string_lossy().into_owned()))
    })?;
    let mime = mime_guess::from_path(path)
        .first_raw()
        .unwrap_or("application/octet-stream");
    let encoded = general_purpose::STANDARD.encode(&data);
    Ok((attachment_name, format!("data:{};base64,{}", mime, encoded)))
}

/// Read input from file or stdin
fn read_input(filename: Option<&str>) -> Result<String, PasteError> {
    if let Some(path) = filename {
        fs::read_to_string(path)
            .map_err(|e| PasteError::FileOpen(e.to_string(), Some(path.to_string())))
    } else if !atty::is(atty::Stream::Stdin) {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|e| PasteError::FileOpen(e.to_string(), None))?;
        Ok(buffer)
    } else {
        Err(PasteError::NoData)
    }
}

/// Detect language for syntax highlighting
fn detect_language(content: &str, filename: Option<&str>) -> Result<String, PasteError> {
    let ss = SyntaxSet::load_defaults_newlines();
    let syntax = if let Some(fname) = filename {
        ss.find_syntax_for_file(fname)
            .map_err(|e| PasteError::LanguageDetection(e.to_string()))?
            .unwrap_or_else(|| ss.find_syntax_plain_text())
    } else {
        ss.find_syntax_by_first_line(content)
            .unwrap_or_else(|| ss.find_syntax_plain_text())
    };

    let syntax_name = syntax.name.to_lowercase();
    if syntax_name.contains("markdown") {
        Ok("markdown".to_string())
    } else if syntax_name != "plain text" {
        Ok("syntaxhighlighting".to_string())
    } else {
        Ok("plaintext".to_string())
    }
}

/// Main entry point
fn main() -> Result<(), PasteError> {
    let matches = Command::new("PasteBin CLI")
        .version("0.1.1")
        .about("A script to paste to https://snip.dssr.ch/")
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .value_name("FILE")
                .help("Read from a file instead of stdin"),
        )
        .arg(
            Arg::new("password")
                .short('p')
                .long("password")
                .value_name("PASSWORD")
                .default_value(DEFAULT_PASSWORD)
                .help("Create a password protected paste"),
        )
        .arg(
            Arg::new("expire")
                .short('e')
                .long("expire")
                .value_name("EXPIRE")
                .default_value(DEFAULT_EXPIRE)
                .value_parser([
                    "5min", "10min", "1hour", "1day", "1week", "1month", "1year", "never",
                ])
                .help("Expiration time of the paste"),
        )
        .arg(
            Arg::new("sourcecode")
                .short('s')
                .long("sourcecode")
                .action(clap::ArgAction::SetTrue)
                .help("Use source code highlighting"),
        )
        .arg(
            Arg::new("markdown")
                .short('m')
                .long("markdown")
                .action(clap::ArgAction::SetTrue)
                .help("Parse paste as markdown"),
        )
        .arg(
            Arg::new("burn")
                .short('b')
                .long("burn")
                .action(clap::ArgAction::SetTrue)
                .help("Burn paste after reading"),
        )
        .arg(
            Arg::new("opendiscussion")
                .short('o')
                .long("opendiscussion")
                .action(clap::ArgAction::SetTrue)
                .help("Allow discussion for the paste"),
        )
        .arg(
            Arg::new("attachment")
                .short('a')
                .long("attachment")
                .value_name("FILE")
                .help("Specify path to a file to attach"),
        )
        .get_matches();

    let mut paste_config = (
        DEFAULT_PASTE_URL.to_string(),
        DEFAULT_FORMATTER.to_string(),
        true,
        matches.get_one::<String>("expire").unwrap().to_string(),
        matches.get_flag("opendiscussion") || DEFAULT_OPEN_DISCUSSION,
        matches.get_flag("burn") || DEFAULT_BURN,
        matches.get_one::<String>("password").map(|s| s.to_string()),
        None::<String>,
        None::<String>,
        None::<String>,
    );

    if matches.get_flag("sourcecode") && matches.get_flag("markdown") {
        return Err(PasteError::InvalidOptions(
            "Cannot specify both --source and --markdown".to_string(),
        ));
    }

    if let Some(attachment) = matches.get_one::<String>("attachment") {
        let (name, data) = handle_attachment(attachment)?;
        paste_config.7 = Some(name);
        paste_config.8 = Some(data);
    }

    paste_config.9 = Some(read_input(
        matches.get_one::<String>("file").map(|s| s.as_str()),
    )?);

    if matches.get_flag("sourcecode") || DEFAULT_SOURCE {
        paste_config.1 = "syntaxhighlighting".to_string();
    } else if matches.get_flag("markdown") {
        paste_config.1 = "markdown".to_string();
    } else {
        // Try to detect language if no specific formatter is set
        paste_config.1 = detect_language(
            paste_config.9.as_ref().unwrap(),
            matches.get_one::<String>("file").map(|s| s.as_str()),
        )?;
    }

    let (id, url, deletetoken, passphrase) = privatebin_send(
        &paste_config.0,
        paste_config.6.as_deref(),
        paste_config.9.as_deref(),
        &paste_config.1,
        paste_config.7.as_deref(),
        paste_config.8.as_deref(),
        paste_config.2,
        paste_config.5,
        paste_config.4,
        &paste_config.3,
    )?;

    println!(
        "\x1b[92mPaste ({}): \x1b[0m{}{}#{}",
        paste_config.1, paste_config.0, url, passphrase
    );
    println!(
        "\x1b[31mDelete paste: \x1b[0m{}/?pasteid={}&deletetoken={}",
        paste_config.0, id, deletetoken
    );

    Ok(())
}
