use clap::{Parser, Subcommand, ValueEnum, command};
use colored::*;
use std::fs::{self, File};
use std::io::{BufReader, Error as IoError, Read, Write};
use std::process::{Child, Command, ExitStatus, Output, Stdio};

const DEFAULT_VISIBILITY: &str = "unlisted";
const HUT_COMMAND: &str = "hut";
const PASTE_COMMAND: &str = "paste";

/// Custom error type for the application
#[derive(Debug)]
enum AppError {
    IoError(IoError),
    CommandError(String),
    ValidationError(String),
    PasteNotFound(String),
}

impl std::fmt::Display for AppError {
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

/// Main CLI structure
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Available commands
#[derive(Subcommand)]
enum Commands {
    /// Manage pastes
    Paste {
        #[command(subcommand)]
        action: PasteCommands,
    },
}

/// Paste-related commands
#[derive(Subcommand)]
enum PasteCommands {
    /// Update an existing paste
    Update {
        /// Source file to update as a paste
        #[arg(short = 's', long)]
        source_file: String,

        /// Visibility of the paste: Public, Unlisted, Private
        #[arg(short = 'v', long, default_value = DEFAULT_VISIBILITY, value_enum)]
        visibility: Visibility,
    },

    /// Rename an existing paste
    Rename {
        /// Paste ID to rename
        #[arg(short = 'i', long)]
        paste_id: String,

        /// New name for the paste
        #[arg(short = 'n', long)]
        new_name: String,
    },
}

/// Enum for visibility
#[derive(Clone, Debug, ValueEnum)]
enum Visibility {
    Public,
    Unlisted,
    Private,
}

impl Visibility {
    fn as_str(&self) -> &'static str {
        match self {
            Visibility::Public => "public",
            Visibility::Unlisted => "unlisted",
            Visibility::Private => "private",
        }
    }
}

fn main() {
    let cli: Cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("{} {}", "[ERROR]".red().bold(), e);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), AppError> {
    match cli.command {
        Commands::Paste { action } => match action {
            PasteCommands::Update {
                source_file,
                visibility,
            } => {
                validate_environment(&source_file)?;

                println!(
                    "{} Finding ID for {}...",
                    "[INFO]".blue().bold(),
                    source_file.cyan()
                );
                let paste_id: String = find_paste_id(&source_file)?;

                println!(
                    "{} Paste ID for {}: {}",
                    "[INFO]".blue().bold(),
                    source_file.cyan(),
                    paste_id.cyan()
                );

                println!("{} Deleting paste...", "[INFO]".blue().bold());
                delete_paste(&paste_id)?;

                println!("{} Creating new paste...", "[INFO]".blue().bold());
                create_paste(&source_file, visibility)?;

                println!(
                    "{} Paste update completed successfully",
                    "[SUCCESS]".green().bold()
                );
                Ok(())
            }
            PasteCommands::Rename { paste_id, new_name } => {
                println!(
                    "{} Renaming paste {} to {}",
                    "[INFO]".blue().bold(),
                    paste_id.cyan(),
                    new_name.cyan()
                );
                rename_paste(&paste_id, &new_name)?;

                println!(
                    "{} Paste rename completed successfully",
                    "[SUCCESS]".green().bold()
                );
                Ok(())
            }
        },
    }
}

fn validate_environment(source_file: &str) -> Result<(), AppError> {
    if which::which(HUT_COMMAND).is_err() {
        return Err(AppError::ValidationError(
            "sourcehut CLI tool (hut) is not installed or not in PATH".to_string(),
        ));
    }

    let metadata: fs::Metadata = fs::metadata(source_file)?;
    if !metadata.is_file() {
        return Err(AppError::ValidationError(format!(
            "Source file '{}' is not a file",
            source_file
        )));
    }
    Ok(())
}

fn execute_hut_command(args: &[&str]) -> Result<String, AppError> {
    let mut cmd: Command = Command::new(HUT_COMMAND);
    cmd.args(args);

    let output: Output = cmd.output()?;

    if !output.status.success() {
        return Err(AppError::CommandError(format!(
            "Failed to execute hut command: {:?}",
            args
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn find_paste_id(source_file: &str) -> Result<String, AppError> {
    let stdout: String = execute_hut_command(&[PASTE_COMMAND, "list"])?;

    stdout
        .lines()
        .filter(|line: &&str| !line.trim().is_empty())
        .fold(None, |current_id: Option<String>, line: &str| {
            let line: &str = line.trim();

            // If we already found the paste, return it
            if current_id.is_some() {
                return current_id;
            }

            // Check if this line contains an ID
            if let Some(id) = line.split_whitespace().next() {
                if id.chars().all(|c: char| c.is_ascii_hexdigit()) {
                    // If this line contains our source file, return the ID
                    if line.contains(source_file) {
                        return Some(id.to_string());
                    }
                    // Otherwise, remember this ID for the next line
                    return Some(id.to_string());
                }
            }

            None
        })
        .ok_or_else(|| AppError::PasteNotFound(format!("No paste ID found for {}", source_file)))
}

fn delete_paste(paste_id: &str) -> Result<(), AppError> {
    let status: ExitStatus = Command::new(HUT_COMMAND)
        .args([PASTE_COMMAND, "delete", paste_id])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if !status.success() {
        return Err(AppError::CommandError(format!(
            "Failed to delete paste with ID: {}",
            paste_id
        )));
    }

    println!(
        "{} Successfully deleted paste with ID: {}",
        "[SUCCESS]".green().bold(),
        paste_id.cyan()
    );

    Ok(())
}

fn create_paste(source_file: &str, visibility: Visibility) -> Result<(), AppError> {
    println!(
        "{} Creating paste for {} with visibility: {}",
        "[INFO]".blue().bold(),
        source_file.cyan(),
        visibility.as_str().cyan()
    );

    let mut child: Child = Command::new(HUT_COMMAND)
        .args([
            PASTE_COMMAND,
            "create",
            "--name",
            source_file,
            "--visibility",
            visibility.as_str(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        let file: File = File::open(source_file)?;
        let mut reader: BufReader<File> = BufReader::new(file);
        let mut buffer: Vec<u8> = Vec::new();
        reader.read_to_end(&mut buffer)?;
        stdin.write_all(&buffer)?;
    }

    let output: Output = child.wait_with_output()?;

    if !output.status.success() {
        return Err(AppError::CommandError(format!(
            "Failed to create paste for file '{}'",
            source_file
        )));
    }

    let url: String = String::from_utf8_lossy(&output.stdout).trim().to_string();

    println!(
        "{} Successfully created new paste for {}: {}",
        "[SUCCESS]".green().bold(),
        source_file.cyan(),
        url.cyan()
    );

    Ok(())
}

fn show_paste(paste_id: &str) -> Result<String, AppError> {
    let output: String = execute_hut_command(&[PASTE_COMMAND, "show", paste_id])?;
    Ok(output)
}

fn rename_paste(paste_id: &str, new_name: &str) -> Result<(), AppError> {
    println!(
        "{} Getting content of paste {}...",
        "[INFO]".blue().bold(),
        paste_id.cyan()
    );

    let content: String = show_paste(paste_id)?;
    let content: String = content.lines().skip(2).collect::<Vec<&str>>().join("\n");

    println!("{} Deleting old paste...", "[INFO]".blue().bold());
    delete_paste(paste_id)?;

    println!(
        "{} Creating new paste with name {}...",
        "[INFO]".blue().bold(),
        new_name.cyan()
    );

    // Create new paste with the filtered content
    let mut child: Child = Command::new(HUT_COMMAND)
        .args([
            PASTE_COMMAND,
            "create",
            "--name",
            new_name,
            "--visibility",
            Visibility::Unlisted.as_str(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(content.as_bytes())?;
    }

    let output: Output = child.wait_with_output()?;

    if !output.status.success() {
        return Err(AppError::CommandError(format!(
            "Failed to create paste with name '{}'",
            new_name
        )));
    }

    let url: String = String::from_utf8_lossy(&output.stdout).trim().to_string();

    println!(
        "{} Successfully renamed paste to {}: {}",
        "[SUCCESS]".green().bold(),
        new_name.cyan(),
        url.cyan()
    );

    Ok(())
}
