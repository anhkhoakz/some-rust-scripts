use crate::{APP_NAME, VERSION};
use clap::{Arg, ArgAction, Command};

pub fn build_cli() -> Command {
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
