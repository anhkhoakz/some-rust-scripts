use clap::{Parser, Subcommand};
use paste::{PasteCommands, handle_paste_command};
use utils::validate_environment;

mod hut;
mod paste;
mod utils;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Paste related commands
    Paste {
        #[command(subcommand)]
        action: PasteCommands,
    },
}

fn main() {
    validate_environment().unwrap();
    let cli: Cli = Cli::parse();

    match cli.command {
        Commands::Paste { action } => handle_paste_command(action).unwrap(),
    }
}
