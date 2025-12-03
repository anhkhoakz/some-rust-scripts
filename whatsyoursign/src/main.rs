use clap::Parser;
use std::fmt::Write;
use std::io;
use std::path::Path;
use std::process::{Command, ExitCode};
use which::which;

const HASH_INDENT: &str = "  ";
const AUTH_PREFIX: &str = " › ";
const AUTH_CONTINUATION: &str = "\n            › ";

enum AppFormat {
    Application,
    Executable,
    Unknown,
}

impl std::fmt::Display for AppFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Application => write!(f, "Application"),
            Self::Executable => write!(f, "Executable"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the file to inspect
    #[arg(short, long)]
    path: String,
}

struct HashInfo {
    md5: String,
    sha1: String,
    sha256: String,
    sha512: String,
    code_directory: String,
}

struct SignatureInfo {
    name: String,
    path: String,
    format: AppFormat,
    is_notarized: bool,
    is_valid: bool,
    signer_type: String,
    authorities: Vec<String>,
    hashes: Option<HashInfo>,
    entitlements: Option<String>,
}

/// Parses the stderr output from `codesign -dvvv` command.
///
/// Returns a tuple of `SignatureInfo` and an optional executable path
/// (for app bundles where the executable differs from the bundle path).
fn parse_codesign_output(stderr: &str) -> (SignatureInfo, Option<String>) {
    let mut identifier = String::new();
    let mut format = String::new();
    let mut is_notarized = false;
    let mut authorities = Vec::new();
    let mut code_directory_hash = String::new();
    let mut executable_path = None;

    for line in stderr.lines() {
        if line.starts_with("Identifier=") {
            identifier = line.split('=').nth(1).unwrap_or("").to_string();
        } else if line.starts_with("Format=") {
            format = line.split('=').nth(1).unwrap_or("").to_string();
        } else if line.starts_with("Notarization Ticket=") {
            is_notarized = line.contains("stapled");
        } else if line.starts_with("Authority=") {
            if let Some(auth) = line.split('=').nth(1) {
                authorities.push(auth.to_string());
            }
        } else if line.starts_with("CDHash=") {
            code_directory_hash = line
                .split('=')
                .nth(1)
                .unwrap_or("")
                .to_string()
                .to_uppercase();
        } else if line.starts_with("CandidateCDHashFull sha256=") {
            if code_directory_hash.is_empty() {
                code_directory_hash = line
                    .split('=')
                    .nth(1)
                    .unwrap_or("")
                    .to_string()
                    .to_uppercase();
            }
        } else if line.starts_with("Executable=") {
            executable_path = Some(line.split('=').nth(1).unwrap_or("").to_string());
        }
    }

    // Determine signer type from first authority
    let signer_type = if authorities.is_empty() {
        "Unknown".to_string()
    } else if authorities[0].contains("Developer ID") {
        "Apple Developer ID".to_string()
    } else if authorities[0].contains("Apple") {
        "Apple".to_string()
    } else {
        "Unknown".to_string()
    };

    // Extract name from identifier (remove com. prefix and company name)
    let name: String = if identifier.contains('.') {
        let last = identifier.split('.').next_back().unwrap_or(&identifier);

        let mut chars = last.chars();
        chars.next().map_or_else(String::new, |first| {
            first.to_uppercase().chain(chars).collect()
        })
    } else {
        identifier
    };

    // Determine type from format
    let app_type = if format.contains("app bundle") {
        AppFormat::Application
    } else if format.contains("Mach-O") {
        AppFormat::Executable
    } else {
        AppFormat::Unknown
    };

    (
        SignatureInfo {
            name,
            path: String::new(), // Will be set from args
            format: app_type,
            is_notarized,
            is_valid: false, // Will be set from spctl
            signer_type,
            authorities,
            hashes: None,       // Will be set from hash commands
            entitlements: None, // Will be set from entitlements command
        },
        executable_path,
    )
}

fn parse_spctl_output(stderr: &str) -> (bool, String) {
    let is_valid = stderr.contains("accepted");
    let source = if stderr.contains("source=") {
        stderr
            .lines()
            .find(|l| l.contains("source="))
            .and_then(|l| l.split('=').nth(1))
            .unwrap_or("")
            .to_string()
    } else {
        String::new()
    };
    (is_valid, source)
}

fn format_output(info: &SignatureInfo) -> String {
    let mut output = String::new();

    // First line: Status line
    let status = if info.is_valid && info.is_notarized {
        format!(
            "{} is validly signed & notarized (Signer: {})",
            info.name, info.signer_type
        )
    } else if info.is_valid {
        format!(
            "{} is validly signed (Signer: {})",
            info.name, info.signer_type
        )
    } else {
        format!("{} is not validly signed", info.name)
    };
    output.push_str(&status);
    output.push('\n');

    // Name
    output.push_str(&info.name);
    output.push('\n');

    // Path
    output.push_str(&info.path);
    output.push('\n');

    // Type
    let _ = writeln!(output, "Type: {}", info.format);

    // Hashes
    if let Some(ref hashes) = info.hashes {
        output.push_str("Hashes:\n");
        let _ = writeln!(output, "{HASH_INDENT}MD5:    {}", hashes.md5);
        let _ = writeln!(output, "{HASH_INDENT}SHA1:   {}", hashes.sha1);
        let _ = writeln!(output, "{HASH_INDENT}SHA256: {}", hashes.sha256);
        let _ = writeln!(output, "{HASH_INDENT}SHA512: {}", hashes.sha512);
        let _ = writeln!(
            output,
            "{HASH_INDENT}Code Directory Hash (SHA-256): {}",
            hashes.code_directory
        );
    } else {
        output.push_str("Hashes: View Hashes\n");
    }

    // Entitled
    if let Some(ref entitlements) = info.entitlements {
        output.push_str("Entitlements:\n");
        output.push_str(entitlements);
        output.push('\n');
    } else {
        output.push_str("Entitled: View Entitlements\n");
    }

    // Sign Auths
    output.push_str("Sign Auths:");
    for (i, auth) in info.authorities.iter().enumerate() {
        if i == 0 {
            let _ = write!(output, "{AUTH_PREFIX}{auth}");
        } else {
            let _ = write!(output, "{AUTH_CONTINUATION}{auth}");
        }
    }
    if info.authorities.is_empty() {
        output.push_str(" (none)");
    }

    output
}

fn get_hash(algorithm: &str, path: &str) -> io::Result<String> {
    let output = if algorithm == "md5" {
        Command::new("md5").arg("-q").arg(path).output()?
    } else {
        Command::new("shasum")
            .args(["-a", algorithm, path])
            .output()?
    };

    Ok(String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_uppercase())
}

fn get_file_hashes(path: &str) -> io::Result<HashInfo> {
    Ok(HashInfo {
        md5: get_hash("md5", path)?,
        sha1: get_hash("1", path)?,
        sha256: get_hash("256", path)?,
        sha512: get_hash("512", path)?,
        code_directory: String::new(), // Will be set from codesign output
    })
}

fn get_entitlements(path: &str) -> io::Result<Option<String>> {
    let entitlements_out = Command::new("codesign")
        .args(["-d", "--entitlements", ":-", path])
        .output()?;

    if entitlements_out.status.success() {
        let entitlements_str = String::from_utf8_lossy(&entitlements_out.stdout);

        // Filter out non-XML lines (like "Executable=..." warnings)
        let plist_content: String = entitlements_str
            .lines()
            .filter(|line| line.trim().starts_with('<') || line.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        if !plist_content.trim().is_empty() && plist_content.contains("<dict>") {
            // Convert plist XML to a more readable format
            return Ok(Some(format_entitlements(&plist_content)));
        }
    }
    Ok(None)
}

fn format_entitlements(plist: &str) -> String {
    // Simple formatting - convert plist to a more readable format
    // This is a basic implementation; could be improved with proper plist parsing
    let mut formatted: String = String::new();
    let mut indent: usize = 0;

    for line in plist.lines() {
        let trimmed: &str = line.trim();
        if trimmed.starts_with("</") {
            indent = indent.saturating_sub(2);
        }

        if !trimmed.is_empty() && !trimmed.starts_with("<?xml") && !trimmed.starts_with("<!DOCTYPE")
        {
            let _ = writeln!(formatted, "{}{}", " ".repeat(indent), trimmed);
        }

        if trimmed.starts_with('<') && !trimmed.starts_with("</") && !trimmed.contains("/>") {
            indent += 2;
        }
    }

    // If formatting didn't work well, try to extract key-value pairs
    if formatted.trim().is_empty() || formatted.lines().count() < 3 {
        // Try to parse as plist and format as JSON-like structure
        return format_entitlements_simple(plist);
    }

    formatted
}

/// Find the first value type and its position after a given start position
fn find_first_value_type(content: &str, start: usize) -> Option<(usize, &'static str)> {
    let true_pos = content[start..].find("<true/>");
    let false_pos = content[start..].find("<false/>");
    let string_pos = content[start..].find("<string>");
    let int_pos = content[start..].find("<integer>");

    let (first_pos, first_type) = if let Some(p) = false_pos
        && (true_pos.is_none() || p < true_pos.unwrap())
    {
        (Some(p), Some("false"))
    } else {
        (true_pos, true_pos.map(|_| "true"))
    };

    let (first_pos, first_type) = if let Some(p) = string_pos
        && (first_pos.is_none() || p < first_pos.unwrap())
    {
        (Some(p), Some("string"))
    } else {
        (first_pos, first_type)
    };

    let (first_pos, first_type) = if let Some(p) = int_pos
        && (first_pos.is_none() || p < first_pos.unwrap())
    {
        (Some(p), Some("integer"))
    } else {
        (first_pos, first_type)
    };

    first_pos.zip(first_type)
}

fn format_entitlements_simple(plist: &str) -> String {
    // Extract key-value pairs from plist format and format as JSON-like structure
    // Handle both multi-line and single-line plists
    let mut result = String::new();
    result.push_str("{\n");

    let mut entries = Vec::new();

    // Process the entire plist string, not just line by line
    // This handles cases where codesign outputs everything on one line
    let plist_content = plist.trim();

    // Find all key-value pairs by searching sequentially
    // In plist format, keys and values appear in pairs: <key>...</key><value>...</value>
    let mut pos = 0;

    while pos < plist_content.len() {
        // Look for <key> tags
        let Some(key_start) = plist_content[pos..].find("<key>") else {
            break;
        };
        let key_start = pos + key_start;
        let Some(key_end) = plist_content[key_start..].find("</key>") else {
            break;
        };
        let key_end = key_start + key_end;
        let key = plist_content[key_start + 5..key_end].to_string();

        // Now look for the value immediately after this key
        // Start searching right after </key>
        let value_search_start = key_end + 6;

        // Find which value type appears first after the key
        let value_found = if let Some((offset, vtype)) =
            find_first_value_type(plist_content, value_search_start)
        {
            match vtype {
                "true" => {
                    entries.push((key.clone(), "true".to_string()));
                    pos = value_search_start + offset + 7;
                    true
                }
                "false" => {
                    entries.push((key.clone(), "false".to_string()));
                    pos = value_search_start + offset + 8;
                    true
                }
                "string" => {
                    let string_start = value_search_start + offset;
                    plist_content[string_start..].find("</string>").is_some_and(
                        |string_end_offset| {
                            let string_end = string_start + string_end_offset;
                            let value = plist_content[string_start + 8..string_end].to_string();
                            entries.push((key.clone(), format!("\"{value}\"")));
                            pos = string_end + 9;
                            true
                        },
                    )
                }
                "integer" => {
                    let int_start = value_search_start + offset;
                    plist_content[int_start..]
                        .find("</integer>")
                        .is_some_and(|int_end_offset| {
                            let int_end = int_start + int_end_offset;
                            let value = plist_content[int_start + 9..int_end].to_string();
                            entries.push((key.clone(), value));
                            pos = int_end + 10;
                            true
                        })
                }
                _ => false,
            }
        } else {
            false
        };

        if !value_found {
            // No value found, move past this key and continue
            pos = key_end + 6;
        }
    }

    // Format entries
    if !entries.is_empty() {
        for (key, value) in &entries {
            let _ = writeln!(result, "  \"{key}\": {value},");
        }
        // Remove trailing comma from last entry
        if let Some(last_comma_pos) = result.rfind(',') {
            result.replace_range(last_comma_pos..=last_comma_pos, "");
        }
    }
    result.push_str("}\n");

    result
}

/// Inspects the code signature of a macOS application or executable.
///
/// # Arguments
///
/// * `path` - Path to the application bundle or executable.
///
/// # Errors
///
/// Returns an `io::Error` if any of the external tooling invocations fail.
fn inspect_signature(path: &str) -> io::Result<()> {
    // codesign -dvvv --verbose=4 <path>
    let codesign_out = Command::new("codesign")
        .args(["-dvvv", "--verbose=4", path])
        .output()?;

    let codesign_stderr = String::from_utf8_lossy(&codesign_out.stderr);
    let (mut info, executable_path) = parse_codesign_output(&codesign_stderr);
    info.path = path.to_string();

    // spctl -a -v <path>
    let spctl_out = Command::new("spctl").args(["-a", "-v", path]).output()?;

    let spctl_stderr = String::from_utf8_lossy(&spctl_out.stderr);
    let (is_valid, source) = parse_spctl_output(&spctl_stderr);
    info.is_valid = is_valid;

    // Update notarization status based on spctl source
    if source.contains("Notarized") {
        info.is_notarized = true;
    }

    // Get file hashes - use executable path for app bundles, otherwise use the provided path
    let hash_path = executable_path
        .as_ref()
        .map_or(path, |exec_path| exec_path.as_str());
    if let Ok(mut hash_info) = get_file_hashes(hash_path) {
        // Extract code directory hash from codesign output
        for line in codesign_stderr.lines() {
            if line.starts_with("CandidateCDHashFull sha256=") {
                hash_info.code_directory = line.split('=').nth(1).unwrap_or("").to_uppercase();
                break;
            } else if line.starts_with("CDHash=") && hash_info.code_directory.is_empty() {
                // Fallback to short CDHash if full is not available
                hash_info.code_directory = line.split('=').nth(1).unwrap_or("").to_uppercase();
            }
        }
        info.hashes = Some(hash_info);
    }

    // Get entitlements
    info.entitlements = get_entitlements(path)?;

    println!("{}", format_output(&info));

    Ok(())
}

fn check_dependencies() -> io::Result<()> {
    which("codesign").map_err(|_| {
        io::Error::other("`codesign` not found in PATH; install Xcode command line tools")
    })?;

    which("spctl").map_err(|_| {
        io::Error::other("`spctl` not found in PATH; install Xcode command line tools")
    })?;

    Ok(())
}

fn main() -> ExitCode {
    if !cfg!(target_os = "macos") {
        eprintln!("This tool only works on macOS");
        return ExitCode::FAILURE;
    }

    let args = Args::parse();

    let path = Path::new(&args.path);
    if !path.exists() {
        eprintln!("Error: Path does not exist: {}", args.path);
        return ExitCode::FAILURE;
    }

    if let Err(e) = check_dependencies() {
        eprintln!("Error: {e}");
        eprintln!("This tool requires `codesign` and `spctl` (macOS developer tools).");
        return ExitCode::FAILURE;
    }

    if let Err(e) = inspect_signature(&args.path) {
        eprintln!("Error inspecting signature: {e}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
