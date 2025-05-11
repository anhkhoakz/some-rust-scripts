use anyhow::{Context, Result};
use clap::Subcommand;
use git2::{BranchType, Repository};

#[derive(Subcommand)]
pub enum ReleaseCommands {
    /// Start a new release branch
    Start {
        /// Name of the release branch
        name: String,
    },
    /// Finish a release branch
    Finish {
        /// Name of the release branch
        name: String,
        /// Keep the release branch after finishing
        #[arg(short, long)]
        keep: bool,
        /// Don't tag the release
        #[arg(short, long)]
        no_tag: bool,
    },
    /// List all release branches
    List,
    /// Publish a release branch to remote
    Publish {
        /// Name of the release branch
        name: String,
    },
    /// Track a release branch from remote
    Track {
        /// Name of the release branch
        name: String,
    },
    /// Delete a release branch
    Delete {
        /// Name of the release branch
        name: String,
        /// Force delete even if not merged
        #[arg(short, long)]
        force: bool,
    },
}

pub fn handle_release(command: ReleaseCommands) -> Result<()> {
    let repo: Repository = Repository::open(".").context("Failed to open repository")?;

    match command {
        ReleaseCommands::Start { name } => start_release(&repo, &name),
        ReleaseCommands::Finish { name, keep, no_tag } => {
            finish_release(&repo, &name, keep, no_tag)
        }
        ReleaseCommands::List => list_releases(&repo),
        ReleaseCommands::Publish { name } => publish_release(&repo, &name),
        ReleaseCommands::Track { name } => track_release(&repo, &name),
        ReleaseCommands::Delete { name, force } => delete_release(&repo, &name, force),
    }
}

fn start_release(repo: &Repository, name: &str) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let develop_branch: &str = config.get_str("gitflow.branch.develop")?;
    let release_prefix: &str = config.get_str("gitflow.prefix.release")?;

    // Get develop branch
    let develop: git2::Branch = repo.find_branch(develop_branch, BranchType::Local)?;
    let develop_commit: git2::Commit = develop.get().peel_to_commit()?;

    // Create release branch
    let release_name: String = format!("{}{}", release_prefix, name);
    repo.branch(&release_name, &develop_commit, false)?;

    // Checkout release branch
    let release_ref: git2::Branch = repo.find_branch(&release_name, BranchType::Local)?;
    repo.checkout_tree(release_ref.get().peel_to_tree()?.as_object(), None)?;
    repo.set_head(release_ref.get().name().unwrap())?;

    println!("Switched to a new branch '{}'", release_name);
    Ok(())
}

fn finish_release(repo: &Repository, name: &str, keep: bool, no_tag: bool) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let develop_branch: &str = config.get_str("gitflow.branch.develop")?;
    let main_branch: &str = config.get_str("gitflow.branch.main")?;
    let release_prefix: &str = config.get_str("gitflow.prefix.release")?;

    let release_name: String = format!("{}{}", release_prefix, name);
    let mut release: git2::Branch = repo.find_branch(&release_name, BranchType::Local)?;

    // Get develop and main branches
    let develop: git2::Branch = repo.find_branch(develop_branch, BranchType::Local)?;
    let main: git2::Branch = repo.find_branch(main_branch, BranchType::Local)?;

    // Merge release into main
    let release_commit: git2::Commit = release.get().peel_to_commit()?;
    let mut merge_opts: git2::MergeOptions = git2::MergeOptions::new();
    repo.merge_commits(
        &main.get().peel_to_commit()?,
        &release_commit,
        Some(&mut merge_opts),
    )?;

    // Create tag if requested
    if !no_tag {
        let tag_name: String = format!("v{}", name);
        let tag_message: String = format!("Release {}", name);
        repo.tag(
            &tag_name,
            &release_commit.as_object(),
            &release_commit.author(),
            &tag_message,
            false,
        )?;
    }

    // Merge release into develop
    repo.merge_commits(
        &develop.get().peel_to_commit()?,
        &release_commit,
        Some(&mut merge_opts),
    )?;

    // Checkout develop
    repo.checkout_tree(develop.get().peel_to_tree()?.as_object(), None)?;
    repo.set_head(develop.get().name().unwrap())?;

    // Delete release branch if not keeping it
    if !keep {
        release.delete()?;
    }

    println!(
        "Release '{}' has been merged into '{}' and '{}'",
        name, main_branch, develop_branch
    );
    Ok(())
}

fn list_releases(repo: &Repository) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let release_prefix: &str = config.get_str("gitflow.prefix.release")?;

    let branches: git2::Branches = repo.branches(Some(BranchType::Local))?;
    let mut releases: Vec<String> = Vec::new();

    for branch in branches {
        let (branch, _): (git2::Branch, git2::BranchType) = branch?;
        if let Some(name) = branch.name()? {
            if name.starts_with(release_prefix) {
                releases.push(name[release_prefix.len()..].to_string());
            }
        }
    }

    if releases.is_empty() {
        println!("No release branches found.");
    } else {
        println!("Release branches:");
        for release in releases {
            println!("  {}", release);
        }
    }

    Ok(())
}

fn publish_release(repo: &Repository, name: &str) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let release_prefix: &str = config.get_str("gitflow.prefix.release")?;

    let release_name: String = format!("{}{}", release_prefix, name);
    let release: git2::Branch = repo.find_branch(&release_name, BranchType::Local)?;

    // Push to remote
    let mut remote: git2::Remote = repo.find_remote("origin")?;
    remote.push(&[release.get().name().unwrap()], None)?;

    println!("Published release '{}' to remote", name);
    Ok(())
}

fn track_release(repo: &Repository, name: &str) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let release_prefix: &str = config.get_str("gitflow.prefix.release")?;

    let release_name: String = format!("{}{}", release_prefix, name);
    let remote_name: String = format!("origin/{}", release_name);

    // Create tracking branch
    let remote_branch: git2::Branch = repo.find_branch(&remote_name, BranchType::Remote)?;
    repo.branch(&release_name, &remote_branch.get().peel_to_commit()?, false)?;

    println!("Tracking release '{}' from remote", name);
    Ok(())
}

fn delete_release(repo: &Repository, name: &str, force: bool) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let release_prefix: &str = config.get_str("gitflow.prefix.release")?;

    let release_name: String = format!("{}{}", release_prefix, name);
    let mut release: git2::Branch = repo.find_branch(&release_name, BranchType::Local)?;

    if !force {
        // Check if branch is merged
        let develop_branch: &str = config.get_str("gitflow.branch.develop")?;
        let develop: git2::Branch = repo.find_branch(develop_branch, BranchType::Local)?;
        let release_commit: git2::Commit = release.get().peel_to_commit()?;
        let develop_commit: git2::Commit = develop.get().peel_to_commit()?;

        let mut revwalk: git2::Revwalk = repo.revwalk()?;
        revwalk.push(develop_commit.id())?;
        let mut found: bool = false;
        for oid in revwalk {
            if oid? == release_commit.id() {
                found = true;
                break;
            }
        }
        if !found {
            anyhow::bail!(
                "Branch '{}' is not fully merged. Use -f to force delete.",
                release_name
            );
        }
    }

    release.delete()?;
    println!("Deleted release branch '{}'", release_name);
    Ok(())
}
