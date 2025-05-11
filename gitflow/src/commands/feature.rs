use anyhow::{Context, Result};
use clap::Subcommand;
use git2::{BranchType, Repository};

#[derive(Subcommand)]
pub enum FeatureCommands {
    /// Start a new feature branch
    Start {
        /// Name of the feature branch
        name: String,
    },
    /// Finish a feature branch
    Finish {
        /// Name of the feature branch
        name: String,
        /// Keep the feature branch after finishing
        #[arg(short, long)]
        keep: bool,
    },
    /// List all feature branches
    List,
    /// Publish a feature branch to remote
    Publish {
        /// Name of the feature branch
        name: String,
    },
    /// Track a feature branch from remote
    Track {
        /// Name of the feature branch
        name: String,
    },
    /// Delete a feature branch
    Delete {
        /// Name of the feature branch
        name: String,
        /// Force delete even if not merged
        #[arg(short, long)]
        force: bool,
    },
}

pub fn handle_feature(command: FeatureCommands) -> Result<()> {
    let repo: Repository = Repository::open(".").context("Failed to open repository")?;

    match command {
        FeatureCommands::Start { name } => start_feature(&repo, &name),
        FeatureCommands::Finish { name, keep } => finish_feature(&repo, &name, keep),
        FeatureCommands::List => list_features(&repo),
        FeatureCommands::Publish { name } => publish_feature(&repo, &name),
        FeatureCommands::Track { name } => track_feature(&repo, &name),
        FeatureCommands::Delete { name, force } => delete_feature(&repo, &name, force),
    }
}

fn start_feature(repo: &Repository, name: &str) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let develop_branch: &str = config.get_str("gitflow.branch.develop")?;
    let feature_prefix: &str = config.get_str("gitflow.prefix.feature")?;

    // Get develop branch
    let develop: git2::Branch = repo.find_branch(develop_branch, BranchType::Local)?;
    let develop_commit: git2::Commit = develop.get().peel_to_commit()?;

    // Create feature branch
    let feature_name: String = format!("{}{}", feature_prefix, name);
    repo.branch(&feature_name, &develop_commit, false)?;

    // Checkout feature branch
    let feature_ref: git2::Branch = repo.find_branch(&feature_name, BranchType::Local)?;
    repo.checkout_tree(feature_ref.get().peel_to_tree()?.as_object(), None)?;
    repo.set_head(feature_ref.get().name().unwrap())?;

    println!("Switched to a new branch '{}'", feature_name);
    Ok(())
}

fn finish_feature(repo: &Repository, name: &str, keep: bool) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let develop_branch: &str = config.get_str("gitflow.branch.develop")?;
    let feature_prefix: &str = config.get_str("gitflow.prefix.feature")?;

    let feature_name: String = format!("{}{}", feature_prefix, name);
    let mut feature: git2::Branch = repo.find_branch(&feature_name, BranchType::Local)?;

    // Get develop branch
    let develop: git2::Branch = repo.find_branch(develop_branch, BranchType::Local)?;

    // Merge feature into develop
    let feature_commit: git2::Commit = feature.get().peel_to_commit()?;
    let mut merge_opts: git2::MergeOptions = git2::MergeOptions::new();
    repo.merge_commits(
        &develop.get().peel_to_commit()?,
        &feature_commit,
        Some(&mut merge_opts),
    )?;

    // Checkout develop
    repo.checkout_tree(develop.get().peel_to_tree()?.as_object(), None)?;
    repo.set_head(develop.get().name().unwrap())?;

    // Delete feature branch if not keeping it
    if !keep {
        feature.delete()?;
    }

    println!(
        "Feature '{}' has been merged into '{}'",
        name, develop_branch
    );
    Ok(())
}

fn list_features(repo: &Repository) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let feature_prefix: &str = config.get_str("gitflow.prefix.feature")?;

    let branches: git2::Branches = repo.branches(Some(BranchType::Local))?;
    let mut features: Vec<String> = Vec::new();

    for branch in branches {
        let (branch, _): (git2::Branch, git2::BranchType) = branch?;
        if let Some(name) = branch.name()? {
            if name.starts_with(feature_prefix) {
                features.push(name[feature_prefix.len()..].to_string());
            }
        }
    }

    if features.is_empty() {
        println!("No feature branches found.");
    } else {
        println!("Feature branches:");
        for feature in features {
            println!("  {}", feature);
        }
    }

    Ok(())
}

fn publish_feature(repo: &Repository, name: &str) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let feature_prefix: &str = config.get_str("gitflow.prefix.feature")?;

    let feature_name: String = format!("{}{}", feature_prefix, name);
    let feature: git2::Branch = repo.find_branch(&feature_name, BranchType::Local)?;

    // Push to remote
    let mut remote: git2::Remote = repo.find_remote("origin")?;
    remote.push(&[feature.get().name().unwrap()], None)?;

    println!("Published feature '{}' to remote", name);
    Ok(())
}

fn track_feature(repo: &Repository, name: &str) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let feature_prefix: &str = config.get_str("gitflow.prefix.feature")?;

    let feature_name: String = format!("{}{}", feature_prefix, name);
    let remote_name: String = format!("origin/{}", feature_name);

    // Create tracking branch
    let remote_branch: git2::Branch = repo.find_branch(&remote_name, BranchType::Remote)?;
    repo.branch(&feature_name, &remote_branch.get().peel_to_commit()?, false)?;

    println!("Tracking feature '{}' from remote", name);
    Ok(())
}

fn delete_feature(repo: &Repository, name: &str, force: bool) -> Result<()> {
    let config: git2::Config = repo.config()?;
    let feature_prefix: &str = config.get_str("gitflow.prefix.feature")?;

    let feature_name: String = format!("{}{}", feature_prefix, name);
    let mut feature: git2::Branch = repo.find_branch(&feature_name, BranchType::Local)?;

    if !force {
        // Check if branch is merged
        let develop_branch: &str = config.get_str("gitflow.branch.develop")?;
        let develop: git2::Branch = repo.find_branch(develop_branch, BranchType::Local)?;
        let feature_commit: git2::Commit = feature.get().peel_to_commit()?;
        let develop_commit: git2::Commit = develop.get().peel_to_commit()?;

        let mut revwalk: git2::Revwalk = repo.revwalk()?;
        revwalk.push(develop_commit.id())?;
        let mut found: bool = false;
        for oid in revwalk {
            if oid? == feature_commit.id() {
                found = true;
                break;
            }
        }
        if !found {
            anyhow::bail!(
                "Branch '{}' is not fully merged. Use -f to force delete.",
                feature_name
            );
        }
    }

    feature.delete()?;
    println!("Deleted feature branch '{}'", feature_name);
    Ok(())
}
