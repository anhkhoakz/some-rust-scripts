use anyhow::{Context, Result};
use clap::Subcommand;
use git2::{BranchType, Repository};

#[derive(Subcommand)]
pub enum HotfixCommands {
    /// Start a new hotfix branch
    Start {
        /// Name of the hotfix branch
        name: String,
    },
    /// Finish a hotfix branch
    Finish {
        /// Name of the hotfix branch
        name: String,
        /// Keep the hotfix branch after finishing
        #[arg(short, long)]
        keep: bool,
        /// Don't tag the hotfix
        #[arg(short, long)]
        no_tag: bool,
    },
    /// List all hotfix branches
    List,
    /// Publish a hotfix branch to remote
    Publish {
        /// Name of the hotfix branch
        name: String,
    },
    /// Track a hotfix branch from remote
    Track {
        /// Name of the hotfix branch
        name: String,
    },
    /// Delete a hotfix branch
    Delete {
        /// Name of the hotfix branch
        name: String,
        /// Force delete even if not merged
        #[arg(short, long)]
        force: bool,
    },
}

pub fn handle_hotfix(command: HotfixCommands) -> Result<()> {
    let repo: Repository = Repository::open(".").context("Failed to open repository")?;

    match command {
        HotfixCommands::Start { name } => start_hotfix(&repo, &name),
        HotfixCommands::Finish { name, keep, no_tag } => finish_hotfix(&repo, &name, keep, no_tag),
        HotfixCommands::List => list_hotfixes(&repo),
        HotfixCommands::Publish { name } => publish_hotfix(&repo, &name),
        HotfixCommands::Track { name } => track_hotfix(&repo, &name),
        HotfixCommands::Delete { name, force } => delete_hotfix(&repo, &name, force),
    }
}

fn start_hotfix(repo: &Repository, name: &str) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let main_branch: &str = config.get_str("gitflow.branch.main")?;
    let hotfix_prefix: &str = config.get_str("gitflow.prefix.hotfix")?;

    // Get main branch
    let main: git2::Branch = repo.find_branch(main_branch, BranchType::Local)?;
    let main_commit: git2::Commit = main.get().peel_to_commit()?;

    // Create hotfix branch
    let hotfix_name: String = format!("{}{}", hotfix_prefix, name);
    repo.branch(&hotfix_name, &main_commit, false)?;

    // Checkout hotfix branch
    let hotfix_ref: git2::Branch = repo.find_branch(&hotfix_name, BranchType::Local)?;
    repo.checkout_tree(hotfix_ref.get().peel_to_tree()?.as_object(), None)?;
    repo.set_head(hotfix_ref.get().name().unwrap())?;

    println!("Switched to a new branch '{}'", hotfix_name);
    Ok(())
}

fn finish_hotfix(repo: &Repository, name: &str, keep: bool, no_tag: bool) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let develop_branch: &str = config.get_str("gitflow.branch.develop")?;
    let main_branch: &str = config.get_str("gitflow.branch.main")?;
    let hotfix_prefix: &str = config.get_str("gitflow.prefix.hotfix")?;

    let hotfix_name: String = format!("{}{}", hotfix_prefix, name);
    let mut hotfix: git2::Branch = repo.find_branch(&hotfix_name, BranchType::Local)?;

    // Get develop and main branches
    let develop: git2::Branch = repo.find_branch(develop_branch, BranchType::Local)?;
    let main: git2::Branch = repo.find_branch(main_branch, BranchType::Local)?;

    // Merge hotfix into main
    let hotfix_commit: git2::Commit = hotfix.get().peel_to_commit()?;
    let mut merge_opts: git2::MergeOptions = git2::MergeOptions::new();
    repo.merge_commits(
        &main.get().peel_to_commit()?,
        &hotfix_commit,
        Some(&mut merge_opts),
    )?;

    // Create tag if requested
    if !no_tag {
        let tag_name: String = format!("v{}", name);
        let tag_message: String = format!("Hotfix {}", name);
        repo.tag(
            &tag_name,
            &hotfix_commit.as_object(),
            &hotfix_commit.author(),
            &tag_message,
            false,
        )?;
    }

    // Merge hotfix into develop
    repo.merge_commits(
        &develop.get().peel_to_commit()?,
        &hotfix_commit,
        Some(&mut merge_opts),
    )?;

    // Checkout develop
    repo.checkout_tree(develop.get().peel_to_tree()?.as_object(), None)?;
    repo.set_head(develop.get().name().unwrap())?;

    // Delete hotfix branch if not keeping it
    if !keep {
        hotfix.delete()?;
    }

    println!(
        "Hotfix '{}' has been merged into '{}' and '{}'",
        name, main_branch, develop_branch
    );
    Ok(())
}

fn list_hotfixes(repo: &Repository) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let hotfix_prefix: &str = config.get_str("gitflow.prefix.hotfix")?;

    let branches: git2::Branches = repo.branches(Some(BranchType::Local))?;
    let mut hotfixes: Vec<String> = Vec::new();

    for branch in branches {
        let (branch, _): (git2::Branch, git2::BranchType) = branch?;
        if let Some(name) = branch.name()? {
            if name.starts_with(hotfix_prefix) {
                hotfixes.push(name[hotfix_prefix.len()..].to_string());
            }
        }
    }

    if hotfixes.is_empty() {
        println!("No hotfix branches found.");
    } else {
        println!("Hotfix branches:");
        for hotfix in hotfixes {
            println!("  {}", hotfix);
        }
    }

    Ok(())
}

fn publish_hotfix(repo: &Repository, name: &str) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let hotfix_prefix: &str = config.get_str("gitflow.prefix.hotfix")?;

    let hotfix_name: String = format!("{}{}", hotfix_prefix, name);
    let hotfix: git2::Branch = repo.find_branch(&hotfix_name, BranchType::Local)?;

    // Push to remote
    let mut remote: git2::Remote = repo.find_remote("origin")?;
    remote.push(&[hotfix.get().name().unwrap()], None)?;

    println!("Published hotfix '{}' to remote", name);
    Ok(())
}

fn track_hotfix(repo: &Repository, name: &str) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let hotfix_prefix: &str = config.get_str("gitflow.prefix.hotfix")?;

    let hotfix_name: String = format!("{}{}", hotfix_prefix, name);
    let remote_name: String = format!("origin/{}", hotfix_name);

    // Create tracking branch
    let remote_branch: git2::Branch = repo.find_branch(&remote_name, BranchType::Remote)?;
    repo.branch(&hotfix_name, &remote_branch.get().peel_to_commit()?, false)?;

    println!("Tracking hotfix '{}' from remote", name);
    Ok(())
}

fn delete_hotfix(repo: &Repository, name: &str, force: bool) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let hotfix_prefix: &str = config.get_str("gitflow.prefix.hotfix")?;

    let hotfix_name: String = format!("{}{}", hotfix_prefix, name);
    let mut hotfix: git2::Branch = repo.find_branch(&hotfix_name, BranchType::Local)?;

    if !force {
        // Check if branch is merged
        let develop_branch: &str = config.get_str("gitflow.branch.develop")?;
        let develop: git2::Branch = repo.find_branch(develop_branch, BranchType::Local)?;
        let hotfix_commit: git2::Commit = hotfix.get().peel_to_commit()?;
        let develop_commit: git2::Commit = develop.get().peel_to_commit()?;

        let mut revwalk: git2::Revwalk = repo.revwalk()?;
        revwalk.push(develop_commit.id())?;
        let mut found: bool = false;
        for oid in revwalk {
            if oid? == hotfix_commit.id() {
                found = true;
                break;
            }
        }
        if !found {
            anyhow::bail!(
                "Branch '{}' is not fully merged. Use -f to force delete.",
                hotfix_name
            );
        }
    }

    hotfix.delete()?;
    println!("Deleted hotfix branch '{}'", hotfix_name);
    Ok(())
}
