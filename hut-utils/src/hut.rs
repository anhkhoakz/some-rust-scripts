use crate::utils::{AppError, Colorize, HUT_COMMAND, PASTE_COMMAND, execute_hut_command};
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::process::{Child, Command, ExitStatus, Output, Stdio};

pub const DEFAULT_VISIBILITY: &str = "unlisted";

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum Visibility {
    Public,
    Unlisted,
    Private,
}

impl Visibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            Visibility::Public => "public",
            Visibility::Unlisted => "unlisted",
            Visibility::Private => "private",
        }
    }
}

pub fn find_paste_id(source_file: &str) -> Result<String, AppError> {
    let stdout: String = execute_hut_command(&[PASTE_COMMAND, "list"])?;

    stdout
        .lines()
        .filter(|line: &&str| !line.trim().is_empty())
        .fold(None, |current_id: Option<String>, line: &str| {
            let line: &str = line.trim();

            if current_id.is_some() {
                return current_id;
            }

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

pub fn delete_paste(paste_id: &str) -> Result<(), AppError> {
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

pub fn create_paste(source_file: &str, visibility: Visibility) -> Result<(), AppError> {
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

pub fn show_paste(paste_id: &str) -> Result<String, AppError> {
    let output: String = execute_hut_command(&[PASTE_COMMAND, "show", paste_id])?;
    Ok(output)
}
