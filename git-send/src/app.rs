use crate::config::Config;
use crate::git_ops::{GitContext, GitOperations};
use anyhow::{Context, Result};
use colored::Colorize;
use log::{error, info, warn};

pub struct GitSendApp {
    config: Config,
    git_ops: GitOperations,
}

impl GitSendApp {
    pub fn new(config: Config) -> Self {
        let git_ops: GitOperations = GitOperations::new(config.dry_run);
        Self { config, git_ops }
    }

    pub fn run(&self, commit_message: &str) -> Result<()> {
        let context = self
            .git_ops
            .get_context()
            .context("Failed to get git context")?;

        info!(
            "Working on branch '{}' ({})",
            context.branch, context.remote_url
        );

        if self.config.dry_run {
            self.run_dry_mode(&context, commit_message)?;
            return Ok(());
        }

        // Stage changes
        self.git_ops.stage_all()?;

        // Check if there are staged changes to commit
        let has_staged_changes: bool = self.git_ops.has_staged_changes()?;
        if !has_staged_changes {
            warn!("No changes to commit");
            println!("{}", "No changes to commit".yellow());
        }
        if has_staged_changes {
            self.git_ops.commit(commit_message)?;
            println!("{}", "Changes committed".green());
        }

        // Pull with rebase
        let should_pull: bool = !self.config.no_pull;
        if !should_pull {
            info!("Skipping pull (no_pull enabled)");
        }
        if should_pull {
            match self.git_ops.pull_rebase(&context.branch) {
                Ok(_) => println!("{}", "Pulled latest changes".green()),
                Err(e) => {
                    error!("Pull failed: {}", e);
                    return Err(e);
                }
            }
        }

        // Push changes
        let should_push: bool = !self.config.no_push;
        if !should_push {
            info!("Skipping push (no_push enabled)");
        }
        if should_push {
            match self.git_ops.push(&context.branch) {
                Ok(_) => println!("{}", "Pushed changes".green()),
                Err(e) => {
                    error!("Push failed: {}", e);
                    return Err(e);
                }
            }
        }

        println!(
            "{}",
            "\nAll operations completed successfully".green().bold()
        );
        Ok(())
    }

    fn run_dry_mode(&self, context: &GitContext, commit_message: &str) -> Result<()> {
        println!("{}", "=== DRY RUN MODE ===".yellow().bold());
        println!("Branch: {}", context.branch.cyan());
        println!("Remote: {}", context.remote_url.cyan());
        println!();
        println!("{}", "Would execute:".yellow());
        println!("  1. git add -A");
        println!("  2. git commit -m '{}'", commit_message);
        if !self.config.no_pull {
            println!("  3. git pull --rebase origin {}", context.branch);
        }
        if !self.config.no_push {
            println!("  4. git push origin {}", context.branch);
        }
        println!();
        println!("{}", "Dry run complete (no changes made)".green());
        Ok(())
    }
}
