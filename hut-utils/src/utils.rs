use std::fmt::Display;
use std::io::Error as IoError;
use std::process::Command;

use which::which;

pub const HUT_COMMAND: &str = "hut";
pub const PASTE_COMMAND: &str = "paste";

// ANSI color codes
const GREEN: &str = "\x1b[32m";
const BLUE: &str = "\x1b[34m";
const CYAN: &str = "\x1b[36m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

pub trait Colorize {
    fn green(&self) -> String;
    fn blue(&self) -> String;
    fn cyan(&self) -> String;
    fn bold(&self) -> String;
}

impl Colorize for &str {
    fn green(&self) -> String {
        format!("{}{}{}{}", GREEN, BOLD, self, RESET)
    }

    fn blue(&self) -> String {
        format!("{}{}{}{}", BLUE, BOLD, self, RESET)
    }

    fn cyan(&self) -> String {
        format!("{}{}{}{}", CYAN, BOLD, self, RESET)
    }

    fn bold(&self) -> String {
        format!("{}{}{}", BOLD, self, RESET)
    }
}

impl Colorize for String {
    fn green(&self) -> String {
        self.as_str().green()
    }

    fn blue(&self) -> String {
        self.as_str().blue()
    }

    fn cyan(&self) -> String {
        self.as_str().cyan()
    }

    fn bold(&self) -> String {
        self.as_str().bold()
    }
}

/// Custom error type for the application
#[derive(Debug)]
pub enum AppError {
    IoError(IoError),
    CommandError(String),
    ValidationError(String),
    PasteNotFound(String),
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::IoError(e) => write!(f, "IO error: {}", e),
            AppError::CommandError(e) => write!(f, "Command error: {}", e),
            AppError::ValidationError(e) => write!(f, "Validation error: {}", e),
            AppError::PasteNotFound(e) => write!(f, "Paste not found: {}", e),
        }
    }
}

impl From<IoError> for AppError {
    fn from(error: IoError) -> Self {
        AppError::IoError(error)
    }
}

pub fn validate_environment() -> Result<(), AppError> {
    if which(HUT_COMMAND).is_err() {
        return Err(AppError::ValidationError(
            "SourceHut CLI tool (hut) is not installed or not in PATH".to_string(),
        ));
    }

    Ok(())
}

pub fn execute_hut_command(args: &[&str]) -> Result<String, AppError> {
    let mut cmd = Command::new(HUT_COMMAND);
    cmd.args(args);

    let output = cmd.output()?;

    if !output.status.success() {
        return Err(AppError::CommandError(format!(
            "Failed to execute hut command: {:?}",
            args
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
