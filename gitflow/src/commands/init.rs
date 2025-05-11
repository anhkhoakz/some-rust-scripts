use anyhow::{Context, Result};
use git2::{BranchType, Repository};
use std::io::{self, Write};

pub fn init(defaults: bool) -> Result<()> {
    let repo: Repository = Repository::open(".").context("Failed to open repository")?;

    if is_initialized(&repo)? && !defaults {
        println!("Already initialized for gitflow.");
        println!("To force reinitialization, use: git flow init -f");
        return Ok(());
    }

    let main_branch: String = get_or_create_main_branch(&repo, defaults)?;

    let develop_branch: String = get_or_create_develop_branch(&repo, &main_branch, defaults)?;

    setup_gitflow_config(&repo, &main_branch, &develop_branch)?;

    println!("Gitflow initialized successfully!");
    Ok(())
}

fn is_initialized(repo: &Repository) -> Result<bool> {
    let config: git2::Config = repo.config()?;
    Ok(config.get_str("gitflow.branch.main").is_ok())
}

fn get_or_create_main_branch(repo: &Repository, defaults: bool) -> Result<String> {
    let branches: git2::Branches<'_> = repo.branches(Some(BranchType::Local))?;
    let branch_count: usize = branches.count();

    let main_branch: String = if branch_count == 0 {
        let main: git2::Branch<'_> = repo.branch("main", &repo.head()?.peel_to_commit()?, false)?;
        main.name()?.unwrap_or("main").to_string()
    } else {
        if defaults {
            "main".to_string()
        } else {
            print!("Which branch should be used for bringing forth production releases? [main] ");
            io::stdout().flush()?;
            let mut input: String = String::new();
            io::stdin().read_line(&mut input)?;
            let branch_name: &str = input.trim();
            if branch_name.is_empty() {
                "main".to_string()
            } else {
                branch_name.to_string()
            }
        }
    };

    Ok(main_branch)
}

fn get_or_create_develop_branch(
    repo: &Repository,
    main_branch: &str,
    defaults: bool,
) -> Result<String> {
    let branches: git2::Branches<'_> = repo.branches(Some(BranchType::Local))?;
    let branch_count: usize = branches.count();

    let develop_branch: String = if branch_count <= 1 {
        let main_ref: git2::Branch<'_> = repo.find_branch(main_branch, BranchType::Local)?;
        let develop: git2::Branch<'_> =
            repo.branch("develop", &main_ref.get().peel_to_commit()?, false)?;
        develop.name()?.unwrap_or("develop").to_string()
    } else {
        if defaults {
            "develop".to_string()
        } else {
            print!(
                "Which branch should be used for integration of the \"next release\"? [develop] "
            );
            io::stdout().flush()?;
            let mut input: String = String::new();
            io::stdin().read_line(&mut input)?;
            let branch_name: &str = input.trim();
            if branch_name.is_empty() {
                "develop".to_string()
            } else {
                branch_name.to_string()
            }
        }
    };

    Ok(develop_branch)
}

fn setup_gitflow_config(repo: &Repository, main_branch: &str, develop_branch: &str) -> Result<()> {
    let mut config: git2::Config = repo.config()?;

    config.set_str("gitflow.branch.main", main_branch)?;
    config.set_str("gitflow.branch.develop", develop_branch)?;

    config.set_str("gitflow.prefix.feature", "feature/")?;
    config.set_str("gitflow.prefix.bugfix", "bugfix/")?;
    config.set_str("gitflow.prefix.release", "release/")?;
    config.set_str("gitflow.prefix.hotfix", "hotfix/")?;
    config.set_str("gitflow.prefix.support", "support/")?;
    config.set_str("gitflow.prefix.versiontag", "v")?;

    Ok(())
}
