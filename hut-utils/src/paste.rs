use crate::hut::{
    DEFAULT_VISIBILITY, Visibility, create_paste, delete_paste, find_paste_id, show_paste,
};
use crate::utils::{AppError, Colorize, HUT_COMMAND, PASTE_COMMAND};
use clap::Subcommand;
use std::io::Write;
use std::process::{Child, Command, Output, Stdio};

/// Paste related commands
#[derive(Subcommand)]
pub enum PasteCommands {
    /// Update an existing paste (delete then create)
    Update {
        /// Source file to update as a paste
        #[arg(short = 's', long)]
        source_file: String,

        /// Remote file name for the paste (defaults to source file name)
        #[arg(short = 'r', long)]
        remote_file: Option<String>,

        /// Visibility of the paste: Public, Unlisted, Private
        #[arg(short = 'v', long, default_value = DEFAULT_VISIBILITY, value_enum)]
        visibility: Visibility,
    },

    /// Rename an existing paste
    Rename {
        /// Current name of the paste to rename
        #[arg(short = 'n', long)]
        current_name: String,

        /// New name for the paste
        #[arg(short = 't', long)]
        new_name: String,
    },
}

pub fn handle_paste_command(action: PasteCommands) -> Result<(), AppError> {
    match action {
        PasteCommands::Update {
            source_file,
            remote_file,
            visibility,
        } => {
            let remote_name: String = remote_file.clone().unwrap_or_else(|| source_file.clone());
            let paste_id: String = find_paste_id(&remote_name)?;

            println!(
                "{} Found existing paste for {} with ID: {}",
                "[INFO]".blue().bold(),
                remote_name.cyan(),
                paste_id.cyan()
            );

            // Step 1: Delete the existing paste
            println!("{} Deleting existing paste...", "[INFO]".blue().bold());
            delete_paste(&paste_id)?;

            // Step 2: Create a new paste
            println!("{} Creating new paste...", "[INFO]".blue().bold());
            create_paste(&source_file, visibility)?;

            println!(
                "{} Paste update completed successfully (delete then create)",
                "[SUCCESS]".green().bold()
            );
            Ok(())
        }
        PasteCommands::Rename {
            current_name,
            new_name,
        } => {
            let paste_id: String = find_paste_id(&current_name)?;

            println!(
                "{} Renaming paste {} to {}",
                "[INFO]".blue().bold(),
                current_name.cyan(),
                new_name.cyan()
            );
            rename_paste(&paste_id, &new_name)?;

            println!(
                "{} Paste rename completed successfully",
                "[SUCCESS]".green().bold()
            );
            Ok(())
        }
    }
}

pub fn rename_paste(paste_id: &str, new_name: &str) -> Result<(), AppError> {
    println!(
        "{} Getting content of paste {}...",
        "[INFO]".blue().bold(),
        paste_id.cyan()
    );

    let content: String = show_paste(paste_id)?;
    let content: String = content.lines().skip(3).collect::<Vec<&str>>().join("\n");

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
