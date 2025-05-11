use anyhow::{Context, Result};
use clap::Subcommand;
use git2::Repository;

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// List all gitflow configuration
    List,
    /// Set a gitflow configuration value
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },
    /// Get a gitflow configuration value
    Get {
        /// Configuration key
        key: String,
    },
}

pub fn handle_config(command: ConfigCommands) -> Result<()> {
    let repo: Repository = Repository::open(".").context("Failed to open repository")?;

    match command {
        ConfigCommands::List => list_config(&repo),
        ConfigCommands::Set { key, value } => set_config(&repo, &key, &value),
        ConfigCommands::Get { key } => get_config(&repo, &key),
    }
}

fn list_config(repo: &Repository) -> Result<()> {
    let config: git2::Config = repo.config()?;

    // List all gitflow configuration
    let mut entries: git2::ConfigEntries = config.entries(Some("gitflow.*"))?;
    let mut configs: Vec<(String, String)> = Vec::new();

    while let Some(entry) = entries.next() {
        let entry = entry?;
        if let Some(name) = entry.name() {
            configs.push((name.to_string(), entry.value().unwrap_or("").to_string()));
        }
    }

    if configs.is_empty() {
        println!("No gitflow configuration found.");
    } else {
        println!("Gitflow configuration:");
        for (key, value) in configs {
            println!("  {} = {}", key, value);
        }
    }

    Ok(())
}

fn set_config(repo: &Repository, key: &str, value: &str) -> Result<()> {
    let mut config: git2::Config = repo.config()?;

    // Set configuration value
    config.set_str(&format!("gitflow.{}", key), value)?;

    println!("Set gitflow.{} = {}", key, value);
    Ok(())
}

fn get_config(repo: &Repository, key: &str) -> Result<()> {
    let config: git2::Config = repo.config()?;

    // Get configuration value
    match config.get_str(&format!("gitflow.{}", key)) {
        Ok(value) => println!("{}", value),
        Err(_) => println!("Configuration 'gitflow.{}' not found", key),
    }

    Ok(())
}
