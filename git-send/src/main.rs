use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, ExitStatus};
use std::time::SystemTime;
use std::collections::HashMap;
use uuid::Uuid;

const SCRIPT_VERSION: &str = "2.0.0";
const DEFAULT_COMMIT_MSG: &str = "I'm too lazy to write a commit message.";
const CONFIG_DIR: &str = ".config/git-send";
const CONFIG_FILE: &str = "config";

struct Config {
    dry_run: bool,
    no_pull: bool,
    no_push: bool,
    commit_message: Option<String>,
    default_commit_msg: String,
}

impl Config {
    fn new() -> Self {
        Config {
            dry_run: false,
            no_pull: false,
            no_push: false,
            commit_message: None,
            default_commit_msg: DEFAULT_COMMIT_MSG.to_string(),
        }
    }

    fn from_args_and_env(args: Vec<String>) -> Result<Self, String> {
        let mut config = Config::new();
        let mut config_file = format!("{}/{}", env::var("HOME").unwrap_or_default(), CONFIG_DIR);
        let mut i = 1;

        while i < args.len() {
            match args[i].as_str() {
                "-h" | "--help" => {
                    print_help();
                    std::process::exit(0);
                }
                "-v" | "--version" => {
                    println!("git send v{}", SCRIPT_VERSION);
                    std::process::exit(0);
                }
                "--dry-run" => config.dry_run = true,
                "--no-pull" => config.no_pull = true,
                "--no-push" => config.no_push = true,
                "--config" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("Option --config requires an argument".to_string());
                    }
                    config_file = args[i].clone();
                }
                _ if !args[i].starts_with('-') => {
                    if config.commit_message.is_some() {
                        return Err("Multiple commit messages provided. Use -h for usage information.".to_string());
                    }
                    config.commit_message = Some(args[i].clone());
                }
                _ => return Err(format!("Unknown option: {}. Use -h for usage information.", args[i])),
            }
            i += 1;
        }

        // Load configuration file
        if Path::new(&config_file).exists() {
            let contents = fs::read_to_string(&config_file)
                .map_err(|e| format!("Failed to read config file: {}", e))?;
            for line in contents.lines() {
                if line.trim().starts_with('#') || line.trim().is_empty() {
                    continue;
                }
                let parts: Vec<&str> = line.splitn(2, '=').collect();
                if parts.len() != 2 {
                    continue;
                }
                let key = parts[0].trim();
                let value = parts[1].trim();
                match key {
                    "default_msg" => config.default_commit_msg = value.to_string(),
                    "dry_run" if value == "1" => config.dry_run = true,
                    "no_pull" if value == "1" => config.no_pull = true,
                    "no_push" if value == "1" => config.no_push = true,
                    _ => eprintln!("Warning: Unknown configuration key: {}", key),
                }
            }
        }

        // Apply environment variables
        if let Ok(msg) = env::var("GIT_SEND_DEFAULT_MSG") {
            config.default_commit_msg = msg;
        }
        if env::var("GIT_SEND_DRY_RUN").map(|v| v == "1").unwrap_or(false) {
            config.dry_run = true;
        }
        if env::var("GIT_SEND_NO_PULL").map(|v| v == "1").unwrap_or(false) {
            config.no_pull = true;
        }
        if env::var("GIT_SEND_NO_PUSH").map(|v| v == "1").unwrap_or(false) {
            config.no_push = true;
        }

        Ok(config)
    }
}

fn print_help() {
    println!(r#"
git send - Commit and push changes with a single command

USAGE:
    git-send [OPTIONS] [COMMIT_MESSAGE]

ARGUMENTS:
    COMMIT_MESSAGE    Custom commit message (optional)

OPTIONS:
    -h, --help           Show this help message
    -v, --version        Show version information
    --dry-run            Show what would be done without actually doing it
    --no-pull            Skip pulling latest changes before push
    --no-push            Skip pushing changes (commit only)
    --config FILE        Use custom configuration file

EXAMPLES:
    git-send                                    # Use default commit message
    git-send "Fix bug in user authentication"   # Custom commit message
    git-send --dry-run                          # Show what would be done
    git-send "Quick fix"                        # Custom commit message
    git-send --no-pull "Local changes only"     # Skip pull step

DESCRIPTION:
    git send automates the common workflow of staging, committing, and pushing
    changes. It includes safety features like confirmation prompts and
    configuration support.

CONFIGURATION:
    The script supports configuration via:
    - Environment variables (GIT_SEND_*)
    - Configuration file (~/.config/git-send/config)
    - Command line options (highest priority)

EXIT CODES:
    0    Success
    1    Error occurred
    130  Operation cancelled (Ctrl+C)
"#);
}

fn err(msg: &str) {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| {
            let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(d.as_secs() as i64, 0).unwrap();
            datetime.format("%Y-%m-%dT%H:%M:%S%z").to_string()
        })
        .unwrap_or_default();
    eprintln!("\x1b[38;2;191;97;106m[{}]: Error: {}\x1b[0m", timestamp, msg);
}

fn info(msg: &str) {
    eprintln!("\x1b[38;2;94;129;172mInfo: {}\x1b[0m", msg);
}

fn warn(msg: &str) {
    eprintln!("\x1b[38;2;235;203;139mWarning: {}\x1b[0m", msg);
}

fn success(msg: &str) {
    eprintln!("\x1b[38;2;163;190;140m{}\x1b[0m", msg);
}

fn run_git_command(args: &[&str], dry_run: bool) -> Result<ExitStatus, String> {
    if dry_run {
        info(&format!("Would run: git {}", args.join(" ")));
        Ok(ExitStatus::default())
    } else {
        let output = Command::new("git")
            .args(args)
            .status()
            .map_err(|e| format!("Failed to execute git command: {}", e))?;
        Ok(output)
    }
}

fn get_git_output(args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute git command: {}", e))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn send(config: Config) -> Result<(), String> {
    let branch = get_git_output(&["rev-parse", "--abbrev-ref", "HEAD"])
        .map_err(|_| "Failed to get current branch. Are you in a valid git repository?".to_string())?;
    let remote_url = get_git_output(&["config", "remote.origin.url"])
        .map_err(|_| "Failed to get remote URL. Are you in a valid git repository?".to_string())?;

    let has_upstream = get_git_output(&["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"]).is_ok();
    if !has_upstream {
        info(&format!("Branch '{}' has no upstream. This will create a new remote branch.", branch));
    }

    info(&format!("Working on {} branch (remote: {})", branch, remote_url));

    if config.dry_run {
        info("DRY RUN MODE - No changes will actually be made");
        info("Would stage all changes (git add -A)");

        let diff_output = get_git_output(&["diff", "--cached"]);
        if diff_output.is_ok() && diff_output.unwrap().is_empty() {
            info("No changes to commit.");
        } else {
            let commit_msg = config.commit_message.unwrap_or_else(|| config.default_commit_msg.clone());
            info(&format!("Would commit with message: '{}'", commit_msg));
        }

        if !config.no_pull {
            info(&format!("Would pull latest changes (git pull --rebase origin {})", branch));
        }

        if !config.no_push {
            info(&format!("Would push changes (git push origin {})", branch));
        }

        success("Dry run completed");
        return Ok(());
    }

    // Stage all changes
    info("Staging all changes...");
    if !run_git_command(&["add", "-A"], config.dry_run)?.success() {
        return Err("Failed to stage changes. Check if files are readable and you have write permissions.".to_string());
    }

    // Check for changes
    let diff_output = get_git_output(&["diff", "--cached"]);
    if diff_output.is_ok() && diff_output.unwrap().is_empty() {
        info("No changes to commit.");
    } else {
        let commit_msg = config.commit_message.unwrap_or_else(|| config.default_commit_msg.clone());
        info(&format!("Committing changes with message: '{}'", commit_msg));
        if !run_git_command(&["commit", "-m", &commit_msg], config.dry_run)?.success() {
            return Err("Failed to commit changes. Check your git configuration and commit message.".to_string());
        }
        success("Changes committed successfully");
    }

    // Pull changes
    if !config.no_pull {
        info("Pulling latest changes...");
        if !run_git_command(&["pull", "--rebase", "origin", &branch], config.dry_run)?.success() {
            return Err("Failed to pull latest changes. Resolve conflicts and try again.".to_string());
        }
        success("Latest changes pulled successfully");
    }

    // Push changes
    if !config.no_push {
        info("Pushing changes...");
        if !run_git_command(&["push", "origin", &branch], config.dry_run)?.success() {
            return Err("Failed to push changes. Check your network connection and permissions.".to_string());
        }
        success("Changes pushed successfully");
    }

    success("All operations completed successfully");
    Ok(())
}

fn main() {
    ctrlc::set_handler(|| {
        eprintln!("\x1b[38;2;235;203;139mOperation cancelled.\x1b[0m");
        std::process::exit(130);
    }).expect("Error setting Ctrl-C handler");

    let args: Vec<String> = env::args().collect();
    let config = match Config::from_args_and_env(args) {
        Ok(config) => config,
        Err(e) => {
            err(&e);
            std::process::exit(1);
        }
    };

    if let Err(e) = send(config) {
        err(&e);
        std::process::exit(1);
    }
}
