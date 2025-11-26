use clap::{Arg, ArgAction, ArgMatches, Command};
use colored::*;
use dirs::config_dir;
use duct::cmd;
use serde::Deserialize;
use std::borrow::Cow;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Output;

#[derive(Debug, Deserialize)]
struct Config {
    default_msg: Option<String>,
    dry_run: Option<u8>,
    no_pull: Option<u8>,
    no_push: Option<u8>,
}

impl Config {
    fn load(path: &Path) -> Self {
        if !path.exists() {
            return Self {
                default_msg: None,
                dry_run: None,
                no_pull: None,
                no_push: None,
            };
        }

        let content: String = fs::read_to_string(path).unwrap_or_default();
        let mut cfg = Config {
            default_msg: None,
            dry_run: None,
            no_pull: None,
            no_push: None,
        };

        'label: for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue 'label;
            }
            if let Some((key, val)) = line.split_once('=') {
                match key.trim() {
                    "default_msg" => cfg.default_msg = Some(val.trim().to_string()),
                    "dry_run" => cfg.dry_run = val.trim().parse().ok(),
                    "no_pull" => cfg.no_pull = val.trim().parse().ok(),
                    "no_push" => cfg.no_push = val.trim().parse().ok(),
                    _ => eprintln!("{}", format!("Unknown key: {}", key).yellow()),
                }
            }
        }

        cfg
    }
}

fn run_git(args: &[&str]) -> Result<(), String> {
    let out: Output = cmd("git", args)
        .stderr_to_stdout()
        .stdout_capture()
        .run()
        .map_err(|e| e.to_string())?;

    let message: Cow<str> = String::from_utf8_lossy(&out.stdout);
    if !message.trim().is_empty() {
        let message_count: usize = message.lines().count();
        eprintln!(
            "{} {}",
            "Git returned".blue(),
            format!("{} message(s)", message_count).blue()
        );
        eprint!("{}", message);
    }

    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stdout).to_string());
    }

    Ok(())
}

fn main() {
    let matches: ArgMatches = Command::new("git-send")
        .about("Stage, commit, pull, push in one shot")
        .version("2.0.0")
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
            Arg::new("pos_msg")
                .help("Commit message (positional)")
                .num_args(1),
        )
        .arg(
            Arg::new("config")
                .long("config")
                .num_args(1)
                .help("Use custom config file"),
        )
        .get_matches();

    let config_path: PathBuf = if let Some(path) = matches.get_one::<String>("config") {
        PathBuf::from(path)
    } else {
        let mut p: PathBuf = config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
        p.push("git-send/config");
        p
    };

    let file_cfg: Config = Config::load(&config_path);

    let cli_message: Option<String> = matches.get_one::<String>("message").cloned();
    let positional: Option<String> = matches.get_one::<String>("pos_msg").cloned();

    let default_msg: String = env::var("GIT_SEND_DEFAULT_MSG")
        .ok()
        .or(file_cfg.default_msg)
        .unwrap_or_else(|| "I'm too lazy to write a commit message.".to_string());

    let commit_message: String = cli_message.or(positional).unwrap_or(default_msg);

    let dry_run: bool = matches.get_flag("dry_run")
        || env::var("GIT_SEND_DRY_RUN")
            .map(|v| v == "1")
            .unwrap_or(false)
        || file_cfg.dry_run == Some(1);

    let no_pull: bool = matches.get_flag("no_pull")
        || env::var("GIT_SEND_NO_PULL")
            .map(|v| v == "1")
            .unwrap_or(false)
        || file_cfg.no_pull == Some(1);

    let no_push: bool = matches.get_flag("no_push")
        || env::var("GIT_SEND_NO_PUSH")
            .map(|v| v == "1")
            .unwrap_or(false)
        || file_cfg.no_push == Some(1);

    let branch: String = cmd("git", &["rev-parse", "--abbrev-ref", "HEAD"])
        .read()
        .unwrap_or_else(|_| {
            eprintln!("{}", "Not a git repo".red());
            std::process::exit(1);
        });

    let branch: &str = branch.trim();

    let remote_url: String = cmd("git", &["config", "remote.origin.url"])
        .read()
        .unwrap_or_else(|_| "unknown".to_string());

    eprintln!(
        "{} {} ({})",
        "Info: Working on branch".blue(),
        branch,
        remote_url
    );

    if dry_run {
        eprintln!("{}", "Dry-run mode enabled".yellow());
        eprintln!("Would: git add -A");
        eprintln!("Would: git commit -m '{}'", commit_message);
        if !no_pull {
            eprintln!("Would: git pull --rebase origin {}", branch);
        }
        if !no_push {
            eprintln!("Would: git push origin {}", branch);
        }
        println!("{}", "Dry-run complete".green());
        return;
    }

    if let Err(e) = run_git(&["add", "-A"]) {
        eprintln!("{}", format!("Failed staging: {}", e).red());
        std::process::exit(1);
    }

    let diff = cmd("git", &["diff", "--cached", "--quiet"]).run();
    if diff.is_err() {
        if let Err(e) = run_git(&["commit", "-m", &commit_message]) {
            eprintln!("{}", format!("Commit failed: {}", e).red());
            std::process::exit(1);
        }
    } else {
        eprintln!("{}", "No changes to commit".yellow());
    }

    if !no_pull {
        if let Err(e) = run_git(&["pull", "--rebase", "origin", branch]) {
            eprintln!("{}", format!("Pull failed: {}", e).red());
            std::process::exit(1);
        }
    }

    if !no_push {
        if let Err(e) = run_git(&["push", "origin", branch]) {
            eprintln!("{}", format!("Push failed: {}", e).red());
            std::process::exit(1);
        }
    }

    println!("{}", "Done".green());
}
