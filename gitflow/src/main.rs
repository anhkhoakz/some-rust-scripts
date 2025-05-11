use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod config;
mod git;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new git repo with support for the branching model
    Init {
        /// Accept all defaults
        #[arg(short, long)]
        defaults: bool,
    },
    /// Manage your feature branches
    Feature {
        #[command(subcommand)]
        subcommand: commands::feature::FeatureCommands,
    },
    /// Manage your bugfix branches
    Bugfix {
        #[command(subcommand)]
        subcommand: commands::bugfix::BugfixCommands,
    },
    /// Manage your release branches
    Release {
        #[command(subcommand)]
        subcommand: commands::release::ReleaseCommands,
    },
    /// Manage your hotfix branches
    Hotfix {
        #[command(subcommand)]
        subcommand: commands::hotfix::HotfixCommands,
    },
    /// Manage your support branches
    Support {
        #[command(subcommand)]
        subcommand: commands::support::SupportCommands,
    },

    /// Manage your git-flow configuration
    Config {
        #[command(subcommand)]
        subcommand: commands::config::ConfigCommands,
    },
    /// Show log deviating from base branch
    Log {
        #[command(subcommand)]
        subcommand: commands::log::LogCommands,
    },
}

fn main() -> Result<()> {
    let cli: Cli = Cli::parse();

    match cli.command {
        Commands::Init { defaults } => {
            commands::init::init(defaults)?;
        }
        Commands::Feature { subcommand } => {
            commands::feature::handle_feature(subcommand)?;
        }
        Commands::Bugfix { subcommand } => {
            commands::bugfix::handle_bugfix(subcommand)?;
        }
        Commands::Release { subcommand } => {
            commands::release::handle_release(subcommand)?;
        }
        Commands::Hotfix { subcommand } => {
            commands::hotfix::handle_hotfix(subcommand)?;
        }
        Commands::Support { subcommand } => {
            commands::support::handle_support(subcommand)?;
        }
        Commands::Config { subcommand } => {
            commands::config::handle_config(subcommand)?;
        }
        Commands::Log { subcommand } => {
            commands::log::handle_log(subcommand)?;
        }
    }

    Ok(())
}
