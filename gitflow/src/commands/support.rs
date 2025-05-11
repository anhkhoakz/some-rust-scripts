use anyhow::{Context, Result};
use clap::Subcommand;
use git2::{BranchType, Repository};

#[derive(Subcommand)]
pub enum SupportCommands {
    /// Start a new support branch
    Start {
        /// Name of the support branch
        name: String,
    },
    /// List all support branches
    List,
    /// Publish a support branch to remote
    Publish {
        /// Name of the support branch
        name: String,
    },
    /// Track a support branch from remote
    Track {
        /// Name of the support branch
        name: String,
    },
    /// Delete a support branch
    Delete {
        /// Name of the support branch
        name: String,
        /// Force delete even if not merged
        #[arg(short, long)]
        force: bool,
    },
}

pub fn handle_support(command: SupportCommands) -> Result<()> {
    let repo: Repository = Repository::open(".").context("Failed to open repository")?;

    match command {
        SupportCommands::Start { name } => start_support(&repo, &name),
        SupportCommands::List => list_supports(&repo),
        SupportCommands::Publish { name } => publish_support(&repo, &name),
        SupportCommands::Track { name } => track_support(&repo, &name),
        SupportCommands::Delete { name, force } => delete_support(&repo, &name, force),
    }
}

fn start_support(repo: &Repository, name: &str) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let main_branch: &str = config.get_str("gitflow.branch.main")?;
    let support_prefix: &str = config.get_str("gitflow.prefix.support")?;

    // Get main branch
    let main: git2::Branch = repo.find_branch(main_branch, BranchType::Local)?;
    let main_commit: git2::Commit = main.get().peel_to_commit()?;

    // Create support branch
    let support_name: String = format!("{}{}", support_prefix, name);
    repo.branch(&support_name, &main_commit, false)?;

    // Checkout support branch
    let support_ref: git2::Branch = repo.find_branch(&support_name, BranchType::Local)?;
    repo.checkout_tree(support_ref.get().peel_to_tree()?.as_object(), None)?;
    repo.set_head(support_ref.get().name().unwrap())?;

    println!("Switched to a new branch '{}'", support_name);
    Ok(())
}

fn list_supports(repo: &Repository) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let support_prefix: &str = config.get_str("gitflow.prefix.support")?;

    let branches: git2::Branches = repo.branches(Some(BranchType::Local))?;
    let mut supports: Vec<String> = Vec::new();

    for branch in branches {
        let (branch, _): (git2::Branch, git2::BranchType) = branch?;
        if let Some(name) = branch.name()? {
            if name.starts_with(support_prefix) {
                supports.push(name[support_prefix.len()..].to_string());
            }
        }
    }

    if supports.is_empty() {
        println!("No support branches found.");
    } else {
        println!("Support branches:");
        for support in supports {
            println!("  {}", support);
        }
    }

    Ok(())
}

fn publish_support(repo: &Repository, name: &str) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let support_prefix: &str = config.get_str("gitflow.prefix.support")?;

    let support_name: String = format!("{}{}", support_prefix, name);
    let support: git2::Branch = repo.find_branch(&support_name, BranchType::Local)?;

    // Push to remote
    let mut remote: git2::Remote = repo.find_remote("origin")?;
    remote.push(&[support.get().name().unwrap()], None)?;

    println!("Published support '{}' to remote", name);
    Ok(())
}

fn track_support(repo: &Repository, name: &str) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let support_prefix: &str = config.get_str("gitflow.prefix.support")?;

    let support_name: String = format!("{}{}", support_prefix, name);
    let remote_name: String = format!("origin/{}", support_name);

    // Create tracking branch
    let remote_branch: git2::Branch = repo.find_branch(&remote_name, BranchType::Remote)?;
    repo.branch(&support_name, &remote_branch.get().peel_to_commit()?, false)?;

    println!("Tracking support '{}' from remote", name);
    Ok(())
}

fn delete_support(repo: &Repository, name: &str, force: bool) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let support_prefix: &str = config.get_str("gitflow.prefix.support")?;

    let support_name: String = format!("{}{}", support_prefix, name);
    let mut support: git2::Branch = repo.find_branch(&support_name, BranchType::Local)?;

    if !force {
        // Check if branch is merged
        let main_branch: &str = config.get_str("gitflow.branch.main")?;
        let main: git2::Branch = repo.find_branch(main_branch, BranchType::Local)?;
        let support_commit: git2::Commit = support.get().peel_to_commit()?;
        let main_commit: git2::Commit = main.get().peel_to_commit()?;

        let mut revwalk: git2::Revwalk = repo.revwalk()?;
        revwalk.push(main_commit.id())?;
        let mut found: bool = false;
        for oid in revwalk {
            if oid? == support_commit.id() {
                found = true;
                break;
            }
        }
        if !found {
            anyhow::bail!(
                "Branch '{}' is not fully merged. Use -f to force delete.",
                support_name
            );
        }
    }

    support.delete()?;
    println!("Deleted support branch '{}'", support_name);
    Ok(())
}
