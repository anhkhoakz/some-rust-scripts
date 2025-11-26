use anyhow::{Context, Result};
use clap::{Arg, ArgAction, ArgMatches, Command};
use colored::*;
use dirs::config_dir;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, Output, Stdio};
use thiserror::Error;

const APP_NAME: &str = "git-send";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_COMMIT_MSG: &str = "Update: automated commit";

#[derive(Debug, Error)]
pub enum GitSendError {
    #[error("Not a git repository")]
    NotGitRepository,

    #[error("Git command failed: {0}")]
    GitCommandFailed(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid configuration value for key '{key}': {value}")]
    InvalidConfigValue { key: String, value: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    pub default_msg: String,
    pub dry_run: bool,
    pub no_pull: bool,
    pub no_push: bool,
    pub auto_stash: bool,
    pub verbose: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_msg: DEFAULT_COMMIT_MSG.to_string(),
            dry_run: false,
            no_pull: false,
            no_push: false,
            auto_stash: false,
            verbose: false,
        }
    }
}

impl Config {
    /// Load configuration from a file with robust error handling
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            debug!("Config file not found at {:?}, using defaults", path);
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;

        Self::parse_config(&content)
    }

    /// Parse configuration content with validation
    fn parse_config(content: &str) -> Result<Self> {
        let mut cfg = Self::default();

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, val)) = line.split_once('=') {
                let key = key.trim();
                let val = val.trim();

                match key {
                    "default_msg" => cfg.default_msg = val.to_string(),
                    "dry_run" => {
                        cfg.dry_run = Self::parse_bool(key, val, line_num)?;
                    }
                    "no_pull" => {
                        cfg.no_pull = Self::parse_bool(key, val, line_num)?;
                    }
                    "no_push" => {
                        cfg.no_push = Self::parse_bool(key, val, line_num)?;
                    }
                    "auto_stash" => {
                        cfg.auto_stash = Self::parse_bool(key, val, line_num)?;
                    }
                    "verbose" => {
                        cfg.verbose = Self::parse_bool(key, val, line_num)?;
                    }
                    _ => {
                        warn!(
                            "Unknown configuration key '{}' on line {}",
                            key,
                            line_num + 1
                        );
                    }
                }
            } else {
                warn!("Invalid configuration line {}: {}", line_num + 1, line);
            }
        }

        Ok(cfg)
    }

    fn parse_bool(key: &str, val: &str, line_num: usize) -> Result<bool> {
        match val {
            "1" | "true" | "yes" | "on" => Ok(true),
            "0" | "false" | "no" | "off" => Ok(false),
            _ => Err(GitSendError::InvalidConfigValue {
                key: key.to_string(),
                value: val.to_string(),
            }
            .into()),
        }
    }

    /// Save configuration to file
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        let content = format!(
            "# git-send configuration file
# Boolean values: 1/true/yes/on or 0/false/no/off

default_msg={}
dry_run={}
no_pull={}
no_push={}
auto_stash={}
verbose={}
",
            self.default_msg,
            self.dry_run,
            self.no_pull,
            self.no_push,
            self.auto_stash,
            self.verbose
        );

        fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct GitContext {
    pub branch: String,
    pub remote_url: String,
    pub has_changes: bool,
}

impl fmt::Display for GitContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Branch: {}, Remote: {}, Changes: {}",
            self.branch, self.remote_url, self.has_changes
        )
    }
}

pub struct GitOperations {
    dry_run: bool,
}

impl GitOperations {
    pub fn new(dry_run: bool) -> Self {
        Self { dry_run }
    }

    /// Execute a git command with proper error handling
    fn execute_git(&self, args: &[&str]) -> Result<Output> {
        debug!("Executing: git {}", args.join(" "));

        if self.dry_run {
            info!("[DRY RUN] Would execute: git {}", args.join(" "));
            return Ok(Output {
                status: std::process::ExitStatus::default(),
                stdout: Vec::new(),
                stderr: Vec::new(),
            });
        }

        let output = ProcessCommand::new("git")
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context("Failed to execute git command")?;

        // Log output for debugging
        if !output.stdout.is_empty() {
            debug!("Git stdout: {}", String::from_utf8_lossy(&output.stdout));
        }
        if !output.stderr.is_empty() {
            debug!("Git stderr: {}", String::from_utf8_lossy(&output.stderr));
        }

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(GitSendError::GitCommandFailed(error_msg.to_string()).into());
        }

        Ok(output)
    }

    /// Get current git context
    pub fn get_context(&self) -> Result<GitContext> {
        let branch_output = self
            .execute_git(&["rev-parse", "--abbrev-ref", "HEAD"])
            .context("Failed to get current branch")?;
        let branch = String::from_utf8_lossy(&branch_output.stdout)
            .trim()
            .to_string();

        if branch.is_empty() {
            return Err(GitSendError::NotGitRepository.into());
        }

        let remote_output = self
            .execute_git(&["config", "remote.origin.url"])
            .unwrap_or_else(|_| Output {
                stdout: b"<no remote>".to_vec(),
                stderr: Vec::new(),
                status: std::process::ExitStatus::default(),
            });
        let remote_url = String::from_utf8_lossy(&remote_output.stdout)
            .trim()
            .to_string();

        let has_changes = self.has_uncommitted_changes()?;

        Ok(GitContext {
            branch,
            remote_url,
            has_changes,
        })
    }

    pub fn has_uncommitted_changes(&self) -> Result<bool> {
        let status_output = self.execute_git(&["status", "--porcelain"])?;
        Ok(!status_output.stdout.is_empty())
    }

    pub fn has_staged_changes(&self) -> Result<bool> {
        let diff_result = self.execute_git(&["diff", "--cached", "--quiet"]);
        Ok(diff_result.is_err() || !self.dry_run)
    }

    pub fn stage_all(&self) -> Result<()> {
        info!("Staging all changes...");
        self.execute_git(&["add", "-A"])
            .context("Failed to stage changes")?;
        Ok(())
    }

    pub fn commit(&self, message: &str) -> Result<()> {
        info!("Committing with message: {}", message);
        self.execute_git(&["commit", "-m", message])
            .context("Failed to commit changes")?;
        Ok(())
    }

    pub fn pull_rebase(&self, branch: &str) -> Result<()> {
        info!("Pulling with rebase from origin/{}", branch);
        self.execute_git(&["pull", "--rebase", "origin", branch])
            .context("Failed to pull with rebase")?;
        Ok(())
    }

    pub fn push(&self, branch: &str) -> Result<()> {
        info!("Pushing to origin/{}", branch);
        self.execute_git(&["push", "origin", branch])
            .context("Failed to push changes")?;
        Ok(())
    }

    pub fn stash(&self) -> Result<()> {
        info!("Stashing changes...");
        self.execute_git(&["stash", "push", "-u"])
            .context("Failed to stash changes")?;
        Ok(())
    }

    pub fn stash_pop(&self) -> Result<()> {
        info!("Restoring stashed changes...");
        self.execute_git(&["stash", "pop"])
            .context("Failed to restore stashed changes")?;
        Ok(())
    }
}

pub struct GitSendApp {
    config: Config,
    git_ops: GitOperations,
}

impl GitSendApp {
    pub fn new(config: Config) -> Self {
        let git_ops = GitOperations::new(config.dry_run);
        Self { config, git_ops }
    }

    pub fn run(&self, commit_message: &str) -> Result<()> {
        let context = self
            .git_ops
            .get_context()
            .context("Failed to get git context")?;

        info!(
            "Working on branch '{}' ({})",
            context.branch, context.remote_url
        );

        if self.config.dry_run {
            self.run_dry_mode(&context, commit_message)?;
            return Ok(());
        }

        // Stage changes
        self.git_ops.stage_all()?;

        // Check if there are staged changes to commit
        if self.git_ops.has_staged_changes()? {
            self.git_ops.commit(commit_message)?;
            println!("{}", "✓ Changes committed".green());
        } else {
            warn!("No changes to commit");
            println!("{}", "⚠ No changes to commit".yellow());
        }

        // Pull with rebase
        if !self.config.no_pull {
            match self.git_ops.pull_rebase(&context.branch) {
                Ok(_) => println!("{}", "✓ Pulled latest changes".green()),
                Err(e) => {
                    error!("Pull failed: {}", e);
                    return Err(e);
                }
            }
        } else {
            info!("Skipping pull (no_pull enabled)");
        }

        // Push changes
        if !self.config.no_push {
            match self.git_ops.push(&context.branch) {
                Ok(_) => println!("{}", "✓ Pushed changes".green()),
                Err(e) => {
                    error!("Push failed: {}", e);
                    return Err(e);
                }
            }
        } else {
            info!("Skipping push (no_push enabled)");
        }

        println!(
            "{}",
            "\n✓ All operations completed successfully".green().bold()
        );
        Ok(())
    }

    fn run_dry_mode(&self, context: &GitContext, commit_message: &str) -> Result<()> {
        println!("{}", "=== DRY RUN MODE ===".yellow().bold());
        println!("Branch: {}", context.branch.cyan());
        println!("Remote: {}", context.remote_url.cyan());
        println!();
        println!("{}", "Would execute:".yellow());
        println!("  1. git add -A");
        println!("  2. git commit -m '{}'", commit_message);
        if !self.config.no_pull {
            println!("  3. git pull --rebase origin {}", context.branch);
        }
        if !self.config.no_push {
            println!("  4. git push origin {}", context.branch);
        }
        println!();
        println!("{}", "✓ Dry run complete (no changes made)".green());
        Ok(())
    }
}

fn get_config_path(matches: &ArgMatches) -> PathBuf {
    if let Some(path) = matches.get_one::<String>("config") {
        return PathBuf::from(path);
    }

    let mut path = config_dir().unwrap_or_else(|| {
        warn!("Could not determine config directory, using current directory");
        PathBuf::from(".")
    });
    path.push(APP_NAME);
    path.push("config");
    path
}

fn merge_config(file_cfg: Config, matches: &ArgMatches) -> Config {
    Config {
        default_msg: env::var("GIT_SEND_DEFAULT_MSG")
            .ok()
            .unwrap_or(file_cfg.default_msg),
        dry_run: matches.get_flag("dry_run")
            || env::var("GIT_SEND_DRY_RUN")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false)
            || file_cfg.dry_run,
        no_pull: matches.get_flag("no_pull")
            || env::var("GIT_SEND_NO_PULL")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false)
            || file_cfg.no_pull,
        no_push: matches.get_flag("no_push")
            || env::var("GIT_SEND_NO_PUSH")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false)
            || file_cfg.no_push,
        auto_stash: file_cfg.auto_stash,
        verbose: matches.get_flag("verbose") || file_cfg.verbose,
    }
}

fn init_logging(verbose: bool) {
    let level = if verbose {
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

fn build_cli() -> Command {
    Command::new(APP_NAME)
        .about("Enterprise-grade git workflow automation tool")
        .long_about(
            "Stage, commit, pull, and push changes in a single command with robust error handling",
        )
        .version(VERSION)
        .arg(
            Arg::new("message")
                .short('m')
                .long("message")
                .value_name("MSG")
                .help("Commit message")
                .num_args(1),
        )
        .arg(
            Arg::new("dry_run")
                .long("dry-run")
                .help("Show operations without executing")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no_pull")
                .long("no-pull")
                .help("Skip git pull")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no_push")
                .long("no-push")
                .help("Skip git push")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose logging")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("pos_msg")
                .help("Commit message (positional)")
                .num_args(1),
        )
        .arg(
            Arg::new("config")
                .long("config")
                .value_name("PATH")
                .num_args(1)
                .help("Use custom config file"),
        )
}

fn main() {
    let matches: ArgMatches = build_cli().get_matches();

    // Initialize logging early
    let verbose: bool = matches.get_flag("verbose");
    init_logging(verbose);

    // Load and merge configuration
    let config_path: PathBuf = get_config_path(&matches);
    let file_cfg: Config = match Config::load(&config_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    let config: Config = merge_config(file_cfg, &matches);

    // Determine commit message
    let commit_message: String = matches
        .get_one::<String>("message")
        .cloned()
        .or_else(|| matches.get_one::<String>("pos_msg").cloned())
        .unwrap_or_else(|| config.default_msg.clone());

    if commit_message.is_empty() {
        error!("Commit message cannot be empty");
        std::process::exit(1);
    }

    // Run the application
    let app: GitSendApp = GitSendApp::new(config);
    if let Err(e) = app.run(&commit_message) {
        error!("{}", format!("Operation failed: {:#}", e).red());
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let cfg = Config::default();
        assert_eq!(cfg.default_msg, DEFAULT_COMMIT_MSG);
        assert!(!cfg.dry_run);
        assert!(!cfg.no_pull);
        assert!(!cfg.no_push);
    }

    #[test]
    fn test_config_parse_bool() {
        assert!(Config::parse_bool("test", "1", 0).unwrap());
        assert!(Config::parse_bool("test", "true", 0).unwrap());
        assert!(Config::parse_bool("test", "yes", 0).unwrap());
        assert!(!Config::parse_bool("test", "0", 0).unwrap());
        assert!(!Config::parse_bool("test", "false", 0).unwrap());
        assert!(Config::parse_bool("test", "invalid", 0).is_err());
    }

    #[test]
    fn test_config_parse() {
        let content: &str = r#"
            # Comment
            default_msg=Test message
            dry_run=true
            no_pull=1
            no_push=false
        "#;

        let cfg = Config::parse_config(content).unwrap();
        assert_eq!(cfg.default_msg, "Test message");
        assert!(cfg.dry_run);
        assert!(cfg.no_pull);
        assert!(!cfg.no_push);
    }

    #[test]
    fn test_git_context_display() {
        let ctx: GitContext = GitContext {
            branch: "main".to_string(),
            remote_url: "git@github.com:user/repo.git".to_string(),
            has_changes: true,
        };
        let display: String = format!("{}", ctx);
        assert!(display.contains("main"));
        assert!(display.contains("git@github.com"));
    }
}
