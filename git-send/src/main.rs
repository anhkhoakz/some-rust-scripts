#![allow(missing_docs, reason = "Missing docs are allowed")]
#![allow(
    clippy::missing_docs_in_private_items,
    reason = "Missing docs are allowed"
)]

mod app;
mod cli;
mod config;
mod errors;
mod git_ops;

use crate::app::GitSendApp;
use crate::cli::build_cli;
use crate::config::{Config, resolve_config_path};
use clap::ArgMatches;
use colored::Colorize;
use log::error;
use std::path::PathBuf;
use std::process::exit;

pub const APP_NAME: &str = "git-send";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

fn init_logging(verbose: bool) {
    let level: log::LevelFilter = if verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    env_logger::Builder::from_default_env()
        .filter_level(level)
        .format_timestamp(None)
        .format_module_path(false)
        .init();
}

fn main() {
    let matches: ArgMatches = build_cli().get_matches();

    // Initialize logging early
    let verbose: bool = matches.get_flag("verbose");
    init_logging(verbose);

    // Load and merge configuration
    let config_path: PathBuf = resolve_config_path(&matches, APP_NAME);
    let file_cfg: Config = match Config::load(&config_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to load configuration: {e:#}");
            exit(1);
        }
    };

    let config: Config = file_cfg.merge_with_cli(&matches);

    // Determine commit message
    let commit_message: String = matches
        .get_one::<String>("message")
        .cloned()
        .or_else(|| matches.get_one::<String>("pos_msg").cloned())
        .unwrap_or_else(|| config.default_msg.clone());

    if commit_message.is_empty() {
        error!("Commit message cannot be empty");
        exit(1);
    }

    // Run the application
    let app: GitSendApp = GitSendApp::new(config);
    if let Err(e) = app.run(&commit_message) {
        error!("{}", format!("Operation failed: {:#}", e).red());
        exit(1);
    }
}
