use anyhow::{Context, Result};
use clap::Subcommand;
use git2::{BranchType, Repository};

#[derive(Subcommand)]
pub enum LogCommands {
    /// Show log of commits that deviate from base branch
    Show {
        /// Branch to show log for
        branch: String,
        /// Base branch to compare against
        #[arg(short, long)]
        base: Option<String>,
    },
}

pub fn handle_log(command: LogCommands) -> Result<()> {
    let repo: Repository = Repository::open(".").context("Failed to open repository")?;

    match command {
        LogCommands::Show { branch, base } => show_log(&repo, &branch, base.as_deref()),
    }
}

fn show_log(repo: &Repository, branch: &str, base: Option<&str>) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let develop_branch: &str = config.get_str("gitflow.branch.develop")?;

    // Get the branch to show log for
    let branch_ref: git2::Branch = repo.find_branch(branch, BranchType::Local)?;
    let branch_commit: git2::Commit = branch_ref.get().peel_to_commit()?;

    // Get the base branch to compare against
    let base_branch: &str = base.unwrap_or(develop_branch);
    let base_ref: git2::Branch = repo.find_branch(base_branch, BranchType::Local)?;
    let base_commit: git2::Commit = base_ref.get().peel_to_commit()?;

    // Create revwalk to find commits
    let mut revwalk: git2::Revwalk = repo.revwalk()?;
    revwalk.push(branch_commit.id())?;
    revwalk.hide(base_commit.id())?;

    // Print commits
    println!(
        "Commits in '{}' that deviate from '{}':",
        branch, base_branch
    );
    for oid in revwalk {
        let oid: git2::Oid = oid?;
        let commit: git2::Commit = repo.find_commit(oid)?;
        println!("  {} {}", oid, commit.summary().unwrap_or(""));
    }

    Ok(())
}
