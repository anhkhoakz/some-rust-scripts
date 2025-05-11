use anyhow::{Context, Result};
use clap::Subcommand;
use git2::{BranchType, Repository};

#[derive(Subcommand)]
pub enum BugfixCommands {
    /// Start a new bugfix branch
    Start {
        /// Name of the bugfix branch
        name: String,
    },
    /// Finish a bugfix branch
    Finish {
        /// Name of the bugfix branch
        name: String,
        /// Keep the bugfix branch after finishing
        #[arg(short, long)]
        keep: bool,
    },
    /// List all bugfix branches
    List,
    /// Publish a bugfix branch to remote
    Publish {
        /// Name of the bugfix branch
        name: String,
    },
    /// Track a bugfix branch from remote
    Track {
        /// Name of the bugfix branch
        name: String,
    },
    /// Delete a bugfix branch
    Delete {
        /// Name of the bugfix branch
        name: String,
        /// Force delete even if not merged
        #[arg(short, long)]
        force: bool,
    },
}

pub fn handle_bugfix(command: BugfixCommands) -> Result<()> {
    let repo: Repository = Repository::open(".").context("Failed to open repository")?;

    match command {
        BugfixCommands::Start { name } => start_bugfix(&repo, &name),
        BugfixCommands::Finish { name, keep } => finish_bugfix(&repo, &name, keep),
        BugfixCommands::List => list_bugfixes(&repo),
        BugfixCommands::Publish { name } => publish_bugfix(&repo, &name),
        BugfixCommands::Track { name } => track_bugfix(&repo, &name),
        BugfixCommands::Delete { name, force } => delete_bugfix(&repo, &name, force),
    }
}

fn start_bugfix(repo: &Repository, name: &str) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let develop_branch: &str = config.get_str("gitflow.branch.develop")?;
    let bugfix_prefix: &str = config.get_str("gitflow.prefix.bugfix")?;

    // Get develop branch
    let develop: git2::Branch = repo.find_branch(develop_branch, BranchType::Local)?;
    let develop_commit: git2::Commit = develop.get().peel_to_commit()?;

    // Create bugfix branch
    let bugfix_name: String = format!("{}{}", bugfix_prefix, name);
    repo.branch(&bugfix_name, &develop_commit, false)?;

    // Checkout bugfix branch
    let bugfix_ref: git2::Branch = repo.find_branch(&bugfix_name, BranchType::Local)?;
    repo.checkout_tree(bugfix_ref.get().peel_to_tree()?.as_object(), None)?;
    repo.set_head(bugfix_ref.get().name().unwrap())?;

    println!("Switched to a new branch '{}'", bugfix_name);
    Ok(())
}

fn finish_bugfix(repo: &Repository, name: &str, keep: bool) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let develop_branch: &str = config.get_str("gitflow.branch.develop")?;
    let bugfix_prefix: &str = config.get_str("gitflow.prefix.bugfix")?;

    let bugfix_name: String = format!("{}{}", bugfix_prefix, name);
    let mut bugfix: git2::Branch = repo.find_branch(&bugfix_name, BranchType::Local)?;

    // Get develop branch
    let develop: git2::Branch = repo.find_branch(develop_branch, BranchType::Local)?;

    // Merge bugfix into develop
    let bugfix_commit: git2::Commit = bugfix.get().peel_to_commit()?;
    let mut merge_opts: git2::MergeOptions = git2::MergeOptions::new();
    repo.merge_commits(
        &develop.get().peel_to_commit()?,
        &bugfix_commit,
        Some(&mut merge_opts),
    )?;

    // Checkout develop
    repo.checkout_tree(develop.get().peel_to_tree()?.as_object(), None)?;
    repo.set_head(develop.get().name().unwrap())?;

    // Delete bugfix branch if not keeping it
    if !keep {
        bugfix.delete()?;
    }

    println!(
        "Bugfix '{}' has been merged into '{}'",
        name, develop_branch
    );
    Ok(())
}

fn list_bugfixes(repo: &Repository) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let bugfix_prefix: &str = config.get_str("gitflow.prefix.bugfix")?;

    let branches: git2::Branches = repo.branches(Some(BranchType::Local))?;
    let mut bugfixes: Vec<String> = Vec::new();

    for branch in branches {
        let (branch, _): (git2::Branch, git2::BranchType) = branch?;
        if let Some(name) = branch.name()? {
            if name.starts_with(bugfix_prefix) {
                bugfixes.push(name[bugfix_prefix.len()..].to_string());
            }
        }
    }

    if bugfixes.is_empty() {
        println!("No bugfix branches found.");
    } else {
        println!("Bugfix branches:");
        for bugfix in bugfixes {
            println!("  {}", bugfix);
        }
    }

    Ok(())
}

fn publish_bugfix(repo: &Repository, name: &str) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let bugfix_prefix: &str = config.get_str("gitflow.prefix.bugfix")?;

    let bugfix_name: String = format!("{}{}", bugfix_prefix, name);
    let bugfix: git2::Branch = repo.find_branch(&bugfix_name, BranchType::Local)?;

    // Push to remote
    let mut remote: git2::Remote = repo.find_remote("origin")?;
    remote.push(&[bugfix.get().name().unwrap()], None)?;

    println!("Published bugfix '{}' to remote", name);
    Ok(())
}

fn track_bugfix(repo: &Repository, name: &str) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let bugfix_prefix: &str = config.get_str("gitflow.prefix.bugfix")?;

    let bugfix_name: String = format!("{}{}", bugfix_prefix, name);
    let remote_name: String = format!("origin/{}", bugfix_name);

    // Create tracking branch
    let remote_branch: git2::Branch = repo.find_branch(&remote_name, BranchType::Remote)?;
    repo.branch(&bugfix_name, &remote_branch.get().peel_to_commit()?, false)?;

    println!("Tracking bugfix '{}' from remote", name);
    Ok(())
}

fn delete_bugfix(repo: &Repository, name: &str, force: bool) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let bugfix_prefix: &str = config.get_str("gitflow.prefix.bugfix")?;

    let bugfix_name: String = format!("{}{}", bugfix_prefix, name);
    let mut bugfix: git2::Branch = repo.find_branch(&bugfix_name, BranchType::Local)?;

    if !force {
        let develop_branch: &str = config.get_str("gitflow.branch.develop")?;
        let develop: git2::Branch = repo.find_branch(develop_branch, BranchType::Local)?;
        let bugfix_commit: git2::Commit = bugfix.get().peel_to_commit()?;
        let develop_commit: git2::Commit = develop.get().peel_to_commit()?;

        let mut revwalk: git2::Revwalk = repo.revwalk()?;
        revwalk.push(develop_commit.id())?;
        let mut found: bool = false;
        for oid in revwalk {
            if oid? != bugfix_commit.id() {
                continue;
            }
            found = true;
            break;
        }
        if found {
            bugfix.delete()?;
            println!("Deleted bugfix branch '{}'", bugfix_name);
            return Ok(());
        }
        anyhow::bail!(
            "Branch '{}' is not fully merged. Use -f to force delete.",
            bugfix_name
        );
    }

    bugfix.delete()?;
    println!("Deleted bugfix branch '{}'", bugfix_name);
    Ok(())
}
