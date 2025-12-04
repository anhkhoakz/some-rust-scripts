use clap::Parser;
use owo_colors::{
    OwoColorize,
    Style, //
};
use serde::Serialize;
use std::env;
use std::fmt::Write;
use std::io::{
    self,
    Write as IoWrite, //
};
use std::path::{
    Path,
    PathBuf, //
};
use std::process::{
    Command,
    ExitCode,
    Stdio, //
};
use which::which;

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum OutputFormat {
    Human,
    Plain,
    Json,
}

#[derive(Clone, Copy, Debug)]
struct ColorConfig {
    enabled: bool,
}

impl ColorConfig {
    fn new() -> Self {
        let enabled = Self::should_enable_color();
        Self { enabled }
    }

    fn should_enable_color() -> bool {
        // Check if `NO_COLOR` is set.
        if env::var("NO_COLOR").is_ok() {
            return false;
        }

        // Check if `WHATSYOURSIGN_NO_COLOR` is set.
        if env::var("WHATSYOURSIGN_NO_COLOR").is_ok() {
            return false;
        }

        // Check if `TERM` is "dumb".
        if env::var("TERM").map(|term| term == "dumb").unwrap_or(false) {
            return false;
        }

        // Check if stdout is a TTY.
        atty::is(atty::Stream::Stdout)
    }

    const fn style() -> Style {
        Style::new()
    }
}

#[derive(Clone)]
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

impl Serialize for AppFormat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Application => serializer.serialize_str("Application"),
            Self::Executable => serializer.serialize_str("Executable"),
            Self::Unknown => serializer.serialize_str("Unknown"),
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the file to inspect.
    #[arg(short, long)]
    path: String,

    /// Output format.
    #[arg(long, value_enum, default_value = "human")]
    format: OutputFormat,

    /// Disable colored output.
    #[arg(long)]
    no_color: bool,

    /// Suppress all non-essential output.
    #[arg(short, long)]
    quiet: bool,

    /// Show detailed debug information for errors.
    #[arg(long)]
    debug: bool,
}

struct HashInfo {
    md5: String,
    sha1: String,
    sha256: String,
    sha512: String,
    code_directory: String,
}

#[derive(Serialize)]
struct HashInfoJson {
    md5: String,
    sha1: String,
    sha256: String,
    sha512: String,
    #[serde(rename = "code_directory")]
    code_directory: String,
}

struct SignatureInfo {
    identifier: String,
    name: String,
    path: String,
    resolved_path: Option<String>, // The actual file path if original was a symlink.
    format: AppFormat,
    is_notarized: bool,
    is_valid: bool,
    signer_type: String,
    authorities: Vec<String>,
    hashes: Option<HashInfo>,
    entitlements: Option<String>,
}

#[derive(Serialize)]
struct SignatureInfoJson {
    name: String,
    path: String,
    format: AppFormat,
    #[serde(rename = "is_notarized")]
    is_notarized: bool,
    #[serde(rename = "is_valid")]
    is_valid: bool,
    #[serde(rename = "signer_type")]
    signer_type: String,
    authorities: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hashes: Option<HashInfoJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    entitlements: Option<serde_json::Value>,
}

impl SignatureInfo {
    fn to_json(&self) -> SignatureInfoJson {
        SignatureInfoJson {
            name: self.name.clone(),
            path: self.path.clone(),
            format: self.format.clone(),
            is_notarized: self.is_notarized,
            is_valid: self.is_valid,
            signer_type: self.signer_type.clone(),
            authorities: self.authorities.clone(),
            hashes: self.hashes.as_ref().map(|h| HashInfoJson {
                md5: h.md5.clone(),
                sha1: h.sha1.clone(),
                sha256: h.sha256.clone(),
                sha512: h.sha512.clone(),
                code_directory: h.code_directory.clone(),
            }),
            entitlements: self
                .entitlements
                .as_ref()
                .and_then(|e| serde_json::from_str(e).ok()),
        }
    }
}

/// Parses the stderr output from `codesign -dvvv` command.
///
/// Returns a tuple of [`SignatureInfo`] and an optional executable path
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

    // Determine signer type from first authority.
    let signer_type = if authorities.is_empty() {
        "Unknown".to_string()
    } else if authorities[0].contains("Developer ID") {
        "Apple Developer ID".to_string()
    } else if authorities[0].contains("Apple") {
        "Apple".to_string()
    } else {
        "Unknown".to_string()
    };

    // Extract name from identifier (remove `com.` prefix and company name).
    let name: String = if !identifier.contains('.') {
        identifier.clone()
    } else {
        let last = identifier.split('.').next_back().unwrap_or(&identifier);
        let mut chars = last.chars();
        chars.next().map_or_else(String::new, |first| {
            first.to_uppercase().chain(chars).collect()
        })
    };

    // Determine type from format.
    let app_type = if format.contains("app bundle") {
        AppFormat::Application
    } else if format.contains("Mach-O") {
        AppFormat::Executable
    } else {
        AppFormat::Unknown
    };

    (
        SignatureInfo {
            identifier,
            name,
            path: String::new(), // Will be set from args.
            resolved_path: None, // Will be set if original path was a symlink.
            format: app_type,
            is_notarized,
            is_valid: false, // Will be set from signature check.
            signer_type,
            authorities,
            hashes: None,       // Will be set from hash commands.
            entitlements: None, // Will be set from entitlements command.
        },
        executable_path,
    )
}

/// Checks signature validity using `codesign -vv`.
///
/// Returns a tuple of `(is_valid, notarization_source)`.
fn check_signature_validity(path: &str) -> io::Result<(bool, String)> {
    let output = Command::new("codesign").args(["-vv", path]).output()?;

    // `codesign -vv` returns exit code 0 if signature is valid.
    let is_valid = output.status.success();

    // Check for notarization in the output.
    let stderr = String::from_utf8_lossy(&output.stderr);
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

    Ok((is_valid, source))
}

fn format_output_human(info: &SignatureInfo, color: ColorConfig) -> String {
    let mut output = String::new();
    let style = ColorConfig::style();

    // Status indicator - colored.
    let status_text = if info.is_valid && info.is_notarized {
        "Valid & Notarized"
    } else if info.is_valid {
        "Valid"
    } else {
        "Invalid"
    };

    let status_display = if !color.enabled {
        status_text.to_string()
    } else {
        let status_color = if info.is_valid {
            style.green()
        } else {
            style.red()
        };
        status_text.style(status_color).to_string()
    };

    // Modernized `codesign --display --verbose=4` format.
    // Header section.
    let _ = writeln!(output, "{}", info.path);
    if let Some(ref resolved) = info.resolved_path {
        let _ = writeln!(output, "Resolved to:      {}", resolved);
    }
    let _ = writeln!(output, "Identifier:       {}", info.identifier);
    let _ = writeln!(output, "Format:           {}", info.format);
    let _ = writeln!(
        output,
        "CodeDirectory:    {}",
        if let Some(ref hashes) = info.hashes {
            &hashes.code_directory
        } else {
            "N/A"
        }
    );

    // Status line.
    let _ = writeln!(output, "Status:           {}", status_display);

    // Authority section (mimics `codesign`'s `Authority=` lines).
    if !info.authorities.is_empty() {
        for auth in &info.authorities {
            let _ = writeln!(output, "Authority:        {}", auth);
        }
    }

    // Notarization status.
    if info.is_notarized {
        let _ = writeln!(output, "Notarization:     Stapled");
    }

    output.push('\n');

    // Hashes section.
    if let Some(ref hashes) = info.hashes {
        let _ = writeln!(output, "Hashes:");
        let _ = writeln!(output, "  MD5:    {}", hashes.md5);
        let _ = writeln!(output, "  SHA1:   {}", hashes.sha1);
        let _ = writeln!(output, "  SHA256: {}", hashes.sha256);
        let _ = writeln!(output, "  SHA512: {}", hashes.sha512);
        output.push('\n');
    }

    // Entitlements section.
    if let Some(ref entitlements) = info.entitlements {
        let _ = writeln!(output, "Entitlements:");
        output.push_str(entitlements);
        output.push('\n');
    }

    output
}

fn format_output_plain(info: &SignatureInfo) -> String {
    let mut output = String::new();

    // One record per line format for easy parsing.
    let status = if info.is_valid && info.is_notarized {
        "validly signed & notarized"
    } else if info.is_valid {
        "validly signed"
    } else {
        "not validly signed"
    };

    let _ = writeln!(output, "status\t{status}");
    let _ = writeln!(output, "name\t{name}", name = info.name);
    let _ = writeln!(output, "path\t{path}", path = info.path);
    if let Some(ref resolved) = info.resolved_path {
        let _ = writeln!(output, "resolved_path\t{resolved}", resolved = resolved);
    }
    let _ = writeln!(output, "type\t{format}", format = info.format);
    let _ = writeln!(
        output,
        "signer_type\t{signer_type}",
        signer_type = info.signer_type
    );
    let _ = writeln!(output, "is_valid\t{is_valid}", is_valid = info.is_valid);
    let _ = writeln!(
        output,
        "is_notarized\t{is_notarized}",
        is_notarized = info.is_notarized
    );

    if let Some(ref hashes) = info.hashes {
        let _ = writeln!(output, "md5\t{md5}", md5 = hashes.md5);
        let _ = writeln!(output, "sha1\t{sha1}", sha1 = hashes.sha1);
        let _ = writeln!(output, "sha256\t{sha256}", sha256 = hashes.sha256);
        let _ = writeln!(output, "sha512\t{sha512}", sha512 = hashes.sha512);
        let _ = writeln!(
            output,
            "code_directory_hash\t{code_directory}",
            code_directory = hashes.code_directory
        );
    }

    for (i, auth) in info.authorities.iter().enumerate() {
        let _ = writeln!(output, "authority_{i}\t{auth}");
    }

    if let Some(ref entitlements) = info.entitlements {
        // For plain format, output entitlements as a single line.
        let entitlements_clean = entitlements.replace(['\n', '\t'], " ");
        let _ = writeln!(output, "entitlements\t{entitlements_clean}");
    }

    output
}

fn format_output_json(info: &SignatureInfo) -> String {
    let json_info = info.to_json();
    serde_json::to_string_pretty(&json_info).unwrap_or_else(|_| "{}".to_string())
}

fn output_with_pager(content: &str) -> io::Result<()> {
    // Only use pager if stdout is a TTY.
    if !atty::is(atty::Stream::Stdout) {
        print!("{content}");
        io::stdout().flush()?;
        return Ok(());
    }

    // Check if less is available
    if which("less").is_ok() {
        let mut less = Command::new("less")
            .args(["-FIRX"])
            .stdin(Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = less.stdin.take() {
            stdin.write_all(content.as_bytes())?;
        }

        less.wait()?;
    } else {
        // Fallback to direct output.
        print!("{content}");
        io::stdout().flush()?;
    }

    Ok(())
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
        code_directory: String::new(), // Will be set from `codesign` output.
    })
}

fn get_entitlements(path: &str) -> io::Result<Option<String>> {
    let entitlements_out = Command::new("codesign")
        .args(["-d", "--entitlements", ":-", path])
        .output()?;

    if !entitlements_out.status.success() {
        return Ok(None);
    }

    let entitlements_str = String::from_utf8_lossy(&entitlements_out.stdout);

    // Filter out non-XML lines (like "Executable=..." warnings)
    let plist_content: String = entitlements_str
        .lines()
        .filter(|line| line.trim().starts_with('<') || line.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    if plist_content.trim().is_empty() || !plist_content.contains("<dict>") {
        return Ok(None);
    }

    // Convert plist XML to a more readable format
    Ok(Some(format_entitlements(&plist_content)))
}

fn format_entitlements(plist: &str) -> String {
    // Simple formatting - convert plist to a more readable format.
    // This is a basic implementation; could be improved with proper plist parsing.
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

    // If formatting didn't work well, try to extract key-value pairs.
    if formatted.trim().is_empty() || formatted.lines().count() < 3 {
        return format_entitlements_simple(plist);
    }

    formatted
}

/// Finds the first value type and its position after a given start position.
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
    // Extract key-value pairs from plist format and format as JSON-like structure.
    // Handle both multi-line and single-line plists.
    let mut result = String::new();
    result.push_str("{\n");

    let mut entries = Vec::new();

    // Process the entire plist string, not just line by line.
    // This handles cases where `codesign` outputs everything on one line.
    let plist_content = plist.trim();

    // Find all key-value pairs by searching sequentially.
    // In plist format, keys and values appear in pairs: `<key>...</key><value>...</value>`.
    let mut pos = 0;

    while pos < plist_content.len() {
        // Look for `<key>` tags.
        let Some(key_start) = plist_content[pos..].find("<key>") else {
            break;
        };
        let key_start = pos + key_start;
        let Some(key_end) = plist_content[key_start..].find("</key>") else {
            break;
        };
        let key_end = key_start + key_end;
        let key = plist_content[key_start + 5..key_end].to_string();

        // Now look for the value immediately after this key.
        // Start searching right after `</key>`.
        let value_search_start = key_end + 6;

        // Find which value type appears first after the key.
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
            // No value found, move past this key and continue.
            pos = key_end + 6;
        }
    }

    // Format entries.
    if !entries.is_empty() {
        for (key, value) in &entries {
            let _ = writeln!(result, "  \"{key}\": {value},");
        }
        // Remove trailing comma from last entry.
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
/// * `format` - Output format to use
/// * `color` - Color configuration
/// * `quiet` - Whether to suppress non-essential output
/// * `debug` - Whether to show debug information
///
/// # Errors
///
/// Returns an `io::Error` if any of the external tooling invocations fail.
fn inspect_signature(
    path: &str,
    format: OutputFormat,
    color: ColorConfig,
    quiet: bool,
    debug: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !quiet {
        eprintln!("Inspecting signature...");
    }

    // Resolve symlinks to get the actual file path.
    let path_obj = Path::new(path);
    let is_symlink = path_obj.is_symlink();
    let resolved_path = resolve_symlink(path_obj)?;
    let actual_path = resolved_path.to_string_lossy().to_string();

    // Use the resolved path for all signature checks.
    let check_path = &actual_path;

    // `codesign -dvvv --verbose=4 <path>`.
    let codesign_out = Command::new("codesign")
        .args(["-dvvv", "--verbose=4", check_path])
        .output()
        .inspect_err(|e| {
            print_command_error("codesign", e, check_path, color, debug);
        })?;

    // Check if `codesign` actually succeeded.
    if codesign_out.status.success() {
        // Continue with processing below.
    } else {
        let stderr = String::from_utf8_lossy(&codesign_out.stderr);
        let error_msg = if stderr.contains("not signed") {
            format!(
                "The file '{path}' is not code signed. This tool only works with signed macOS applications and executables."
            )
        } else if stderr.contains("No such file") {
            format!("The file '{path}' doesn't exist or can't be accessed.")
        } else {
            format!(
                "codesign failed: {}",
                stderr.lines().next().unwrap_or("Unknown error")
            )
        };

        print_error_header(color);
        eprintln!();
        print_error_message(&error_msg, color);
        eprintln!();
        print_suggestion(
            "Make sure the file is a signed macOS application (.app) or executable binary.",
            color,
        );
        eprintln!();
        if !color.enabled {
            eprintln!(
                "Most important: The file must be a signed macOS binary to inspect its signature."
            );
        } else {
            eprintln!(
                "{}",
                "Most important: The file must be a signed macOS binary to inspect its signature."
                    .red()
                    .bold()
            );
        }
        return Err(Box::new(io::Error::other("codesign failed")));
    }

    let codesign_stderr = String::from_utf8_lossy(&codesign_out.stderr);
    let (mut info, executable_path) = parse_codesign_output(&codesign_stderr);
    info.path = path.to_string();
    if is_symlink {
        info.resolved_path = Some(actual_path.clone());
    }

    // Check signature validity using `codesign -vv`.
    let (is_valid, source) = check_signature_validity(check_path).inspect_err(|e| {
        print_command_error("codesign", e, check_path, color, debug);
    })?;
    info.is_valid = is_valid;

    // Check for notarization - if checking an executable, also check the app bundle.
    if source.contains("Notarized") {
        info.is_notarized = true;
    }

    // Check `codesign` output for notarization ticket (this is the most reliable).
    if codesign_stderr.contains("Notarization Ticket=") {
        info.is_notarized = codesign_stderr.contains("stapled");
    }

    // If checking an executable inside an app bundle, check the app bundle's `codesign` output.
    if let Some(ref app_bundle_path) = find_app_bundle(check_path) {
        let app_codesign_out = Command::new("codesign")
            .args(["-dvvv", app_bundle_path])
            .output();

        if let Ok(app_out) = app_codesign_out {
            let app_stderr = String::from_utf8_lossy(&app_out.stderr);
            if app_stderr.contains("Notarization Ticket=") {
                info.is_notarized = app_stderr.contains("stapled");
            }
        }
    }

    // Get file hashes - use executable path for app bundles, otherwise use the resolved path.
    let hash_path: &str = executable_path
        .as_ref()
        .map_or(check_path, |exec_path| exec_path.as_str());
    if let Ok(mut hash_info) = get_file_hashes(hash_path) {
        // Extract code directory hash from `codesign` output.
        for line in codesign_stderr.lines() {
            if line.starts_with("CandidateCDHashFull sha256=") {
                hash_info.code_directory = line.split('=').nth(1).unwrap_or("").to_uppercase();
                break;
            }
            if line.starts_with("CDHash=") && hash_info.code_directory.is_empty() {
                // Fallback to short CDHash if full is not available.
                hash_info.code_directory = line.split('=').nth(1).unwrap_or("").to_uppercase();
            }
        }
        info.hashes = Some(hash_info);
    }

    // Get entitlements - this is optional, so we don't fail if it errors.
    info.entitlements = get_entitlements(check_path).unwrap_or(None);

    // Format and output based on format.
    let output = match format {
        OutputFormat::Human => format_output_human(&info, color),
        OutputFormat::Plain => format_output_plain(&info),
        OutputFormat::Json => format_output_json(&info),
    };

    // Use pager for human-readable output if it's long and we're in a TTY.
    if !matches!(format, OutputFormat::Human) || !atty::is(atty::Stream::Stdout) {
        print!("{output}");
        io::stdout()
            .flush()
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        return Ok(());
    }

    output_with_pager(&output).map_err(|e| {
        if debug {
            eprintln!("Debug: Pager error: {e}");
        }
        // Fallback to direct output.
        print!("{output}");
        io::stdout().flush().unwrap_or(());
        Box::new(e) as Box<dyn std::error::Error>
    })?;

    Ok(())
}

/// Finds the app bundle path if the given path is inside an app bundle.
fn find_app_bundle(path: &str) -> Option<String> {
    let path_obj = Path::new(path);
    let mut current = path_obj;

    // Walk up the directory tree to find `.app` bundle.
    while let Some(parent) = current.parent() {
        if parent
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.ends_with(".app"))
            .unwrap_or(false)
        {
            return Some(parent.to_string_lossy().to_string());
        }
        current = parent;
        if current == Path::new("/") {
            break;
        }
    }
    None
}

/// Resolves symlinks to get the actual target file path.
///
/// Follows symlinks recursively until a non-symlink is found.
fn resolve_symlink(path: &Path) -> io::Result<PathBuf> {
    let mut current = path.to_path_buf();

    // Follow symlinks up to a reasonable limit (to avoid infinite loops).
    for _ in 0..256 {
        if !current.is_symlink() {
            break;
        }
        current = current.read_link()?;
        // If the symlink is relative, resolve it relative to the parent.
        if current.is_relative() {
            if let Some(parent) = path.parent() {
                current = parent.join(&current);
            }
        }
    }

    // Canonicalize to get absolute path.
    std::fs::canonicalize(&current)
}

fn check_dependencies() -> Result<(), Vec<String>> {
    let mut missing = Vec::new();

    if which("codesign").is_err() {
        missing.push("codesign".to_string());
    }

    if which("spctl").is_err() {
        missing.push("spctl".to_string());
    }

    if !missing.is_empty() {
        return Err(missing);
    }

    Ok(())
}

fn print_error_header(color: ColorConfig) {
    let header = if !color.enabled {
        "Error".to_string()
    } else {
        "Error".red().bold().to_string()
    };
    eprintln!("{header}");
}

fn print_error_message(message: &str, color: ColorConfig) {
    if !color.enabled {
        eprintln!("{message}");
        return;
    }
    eprintln!("{}", message.red());
}

fn print_suggestion(suggestion: &str, color: ColorConfig) {
    if !color.enabled {
        eprintln!("{suggestion}");
        return;
    }
    eprintln!("{}", suggestion.bright_blue());
}

fn print_path_error(path: &str, color: ColorConfig) {
    print_error_header(color);
    eprintln!();
    eprintln!("Can't find the file or directory:");
    eprintln!("  {path}");
    eprintln!();

    // Check if it's a permission issue.
    let path_obj = Path::new(path);
    let Some(parent) = path_obj.parent() else {
        print_suggestion("Make sure the path is correct and the file exists.", color);
        eprintln!();
        if !color.enabled {
            eprintln!("Most important: Check that the path is correct and the file exists.");
        } else {
            eprintln!(
                "{}",
                "Most important: Check that the path is correct and the file exists."
                    .red()
                    .bold()
            );
        }
        return;
    };

    if !parent.exists() {
        print_suggestion(
            &format!(
                "The directory '{}' doesn't exist. Make sure the path is correct.",
                parent.display()
            ),
            color,
        );
        eprintln!();
        if !color.enabled {
            eprintln!("Most important: Check that the path is correct and the file exists.");
        } else {
            eprintln!(
                "{}",
                "Most important: Check that the path is correct and the file exists."
                    .red()
                    .bold()
            );
        }
        return;
    }

    print_suggestion(
        &format!(
            "The path exists but the file '{}' doesn't. Check the spelling and try again.",
            path_obj.file_name().and_then(|n| n.to_str()).unwrap_or("")
        ),
        color,
    );

    eprintln!();
    if color.enabled {
        eprintln!(
            "{}",
            "Most important: Check that the path is correct and the file exists."
                .red()
                .bold()
        );
    } else {
        eprintln!("Most important: Check that the path is correct and the file exists.");
    }
}

fn print_dependency_error(missing: &[String], color: ColorConfig) {
    print_error_header(color);
    eprintln!();
    eprintln!("Missing required tools:");
    for tool in missing {
        eprintln!("  • {tool}");
    }
    eprintln!();

    print_suggestion("Install Xcode Command Line Tools by running:", color);
    eprintln!("  xcode-select --install");
    eprintln!();

    if !color.enabled {
        eprintln!("Most important: Install Xcode Command Line Tools to use this tool.");
        return;
    }
    eprintln!(
        "{}",
        "Most important: Install Xcode Command Line Tools to use this tool."
            .red()
            .bold()
    );
}

fn print_command_error(
    command: &str,
    error: &io::Error,
    path: &str,
    color: ColorConfig,
    debug: bool,
) {
    print_error_header(color);
    eprintln!();
    eprintln!("Failed to run '{command}' on:");
    eprintln!("  {path}");
    eprintln!();

    // Try to provide helpful context based on error kind.
    let error_msg = match error.kind() {
        io::ErrorKind::NotFound => {
            format!("The '{command}' command was not found.")
        }
        io::ErrorKind::PermissionDenied => {
            format!(
                "Permission denied. You might need to make the file readable by running:\n  chmod +r \"{path}\""
            )
        }
        _ => {
            format!("Error: {error}")
        }
    };

    print_error_message(&error_msg, color);
    eprintln!();

    // Check if it's an unsigned file.
    if command == "codesign" {
        print_suggestion(
            "The file might not be a signed macOS application or executable.",
            color,
        );
        eprintln!();
    }

    if debug {
        eprintln!("Debug information:");
        eprintln!("  Command: {command}");
        eprintln!("  Path: {path}");
        eprintln!("  Error kind: {:?}", error.kind());
        eprintln!("  Error: {error}");
        eprintln!();
    }

    if !color.enabled {
        eprintln!(
            "Most important: Make sure '{command}' can access the file and it's a valid macOS binary."
        );
        return;
    }
    eprintln!(
        "{}",
        format!(
            "Most important: Make sure '{command}' can access the file and it's a valid macOS binary."
        )
        .red()
        .bold()
    );
}

fn print_unexpected_error(
    error: &dyn std::error::Error,
    context: &str,
    color: ColorConfig,
    debug: bool,
) {
    print_error_header(color);
    eprintln!();
    eprintln!("An unexpected error occurred:");
    eprintln!("  {context}");
    eprintln!();

    if !debug {
        eprintln!("Run with --debug to see detailed error information.");
    } else {
        eprintln!("Debug information:");
        eprintln!("  Error: {error}");
        let mut source = error.source();
        let mut depth = 0;
        while let Some(err) = source {
            depth += 1;
            eprintln!("  Caused by ({depth}): {err}");
            source = err.source();
        }
    }
    eprintln!();

    // Bug report information.
    eprintln!("This looks like a bug. Please report it:");
    eprintln!("  https://github.com/anhkhoakz/some-rust-scripts/issues/new");
    eprintln!();
    eprintln!("Include the following information:");
    eprintln!("  • The command you ran");
    eprintln!("  • The error message above");
    if !debug {
        eprintln!("  • Output from running with --debug flag");
    }
    eprintln!("  • Your macOS version");
    eprintln!();

    if !color.enabled {
        eprintln!("Most important: This is a bug. Please report it with the information above.");
        return;
    }
    eprintln!(
        "{}",
        "Most important: This is a bug. Please report it with the information above."
            .red()
            .bold()
    );
}

fn main() -> ExitCode {
    if !cfg!(target_os = "macos") {
        let color = ColorConfig::new();
        print_error_header(color);
        eprintln!();
        eprintln!("This tool only works on macOS.");
        eprintln!();
        print_suggestion(
            "Run this tool on a macOS system to inspect code signatures.",
            color,
        );
        eprintln!();
        if !color.enabled {
            eprintln!("Most important: This tool requires macOS to function.");
        } else {
            eprintln!(
                "{}",
                "Most important: This tool requires macOS to function."
                    .red()
                    .bold()
            );
        }
        return ExitCode::FAILURE;
    }

    let args = Args::parse();

    // Determine color configuration.
    let mut color = ColorConfig::new();
    if args.no_color {
        color.enabled = false;
    }

    let path = Path::new(&args.path);
    if !path.exists() {
        print_path_error(&args.path, color);
        return ExitCode::FAILURE;
    }

    if let Err(missing) = check_dependencies() {
        print_dependency_error(&missing, color);
        return ExitCode::FAILURE;
    }

    if let Err(e) = inspect_signature(&args.path, args.format, color, args.quiet, args.debug) {
        // Error messages are already printed by `inspect_signature` for most cases.
        // For truly unexpected errors, print additional debug info.
        let error_str = e.to_string();
        // Only print unexpected error if it's not one we've already handled.
        if !error_str.contains("codesign failed") && args.debug {
            print_unexpected_error(e.as_ref(), "while inspecting signature", color, args.debug);
        }
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
