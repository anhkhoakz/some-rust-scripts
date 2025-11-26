use crate::errors::GitSendError;
use anyhow::{Context, Result};
use log::{debug, info};
use std::borrow::Cow;
use std::fmt;
use std::process::{Command as ProcessCommand, Output, Stdio};

#[derive(Debug, Clone)]
pub struct GitContext {
    pub branch: String,
    pub remote_url: String,
    pub has_changes: bool,
}

impl fmt::Display for GitContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Branch: {}, Remote: {}, Changes: {}",
            self.branch, self.remote_url, self.has_changes
        )
    }
}

pub struct GitOperations {
    dry_run: bool,
}

impl GitOperations {
    pub fn new(dry_run: bool) -> Self {
        Self { dry_run }
    }

    /// Execute a git command with proper error handling
    fn execute_git(&self, args: &[&str]) -> Result<Output> {
        debug!("Executing: git {}", args.join(" "));

        if self.dry_run {
            info!("[DRY RUN] Would execute: git {}", args.join(" "));
            return Ok(Output {
                status: std::process::ExitStatus::default(),
                stdout: Vec::new(),
                stderr: Vec::new(),
            });
        }

        let output: Output = ProcessCommand::new("git")
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context("Failed to execute git command")?;

        // Log output for debugging
        if !output.stdout.is_empty() {
            debug!("Git stdout: {}", String::from_utf8_lossy(&output.stdout));
        }
        if !output.stderr.is_empty() {
            debug!("Git stderr: {}", String::from_utf8_lossy(&output.stderr));
        }

        if !output.status.success() {
            let error_msg: Cow<'_, str> = String::from_utf8_lossy(&output.stderr);
            return Err(GitSendError::GitCommandFailed(error_msg.to_string()).into());
        }

        Ok(output)
    }

    /// Get current git context
    pub fn get_context(&self) -> Result<GitContext> {
        let branch_output = self
            .execute_git(&["rev-parse", "--abbrev-ref", "HEAD"])
            .context("Failed to get current branch")?;
        let branch: String = String::from_utf8_lossy(&branch_output.stdout)
            .trim()
            .to_owned();

        if branch.is_empty() {
            return Err(GitSendError::NotGitRepository.into());
        }

        let remote_output = self
            .execute_git(&["config", "remote.origin.url"])
            .unwrap_or_else(|_| Output {
                stdout: b"<no remote>".to_vec(),
                stderr: Vec::new(),
                status: std::process::ExitStatus::default(),
            });
        let remote_url = String::from_utf8_lossy(&remote_output.stdout)
            .trim()
            .to_string();

        let has_changes = self.has_uncommitted_changes()?;

        Ok(GitContext {
            branch,
            remote_url,
            has_changes,
        })
    }

    pub fn has_uncommitted_changes(&self) -> Result<bool> {
        let status_output: Output = self.execute_git(&["status", "--porcelain"])?;
        Ok(!status_output.stdout.is_empty())
    }

    pub fn has_staged_changes(&self) -> Result<bool> {
        let diff_result: Result<Output> = self.execute_git(&["diff", "--cached", "--quiet"]);
        Ok(diff_result.is_err() || !self.dry_run)
    }

    pub fn stage_all(&self) -> Result<()> {
        info!("Staging all changes...");
        self.execute_git(&["add", "-A"])
            .context("Failed to stage changes")?;
        Ok(())
    }

    pub fn commit(&self, message: &str) -> Result<()> {
        info!("Committing with message: {}", message);
        self.execute_git(&["commit", "-m", message])
            .context("Failed to commit changes")?;
        Ok(())
    }

    pub fn pull_rebase(&self, branch: &str) -> Result<()> {
        info!("Pulling with rebase from origin/{}", branch);
        self.execute_git(&["pull", "--rebase", "origin", branch])
            .context("Failed to pull with rebase")?;
        Ok(())
    }

    pub fn push(&self, branch: &str) -> Result<()> {
        info!("Pushing to origin/{}", branch);
        self.execute_git(&["push", "origin", branch])
            .context("Failed to push changes")?;
        Ok(())
    }

    pub fn stash(&self) -> Result<()> {
        info!("Stashing changes...");
        self.execute_git(&["stash", "push", "-u"])
            .context("Failed to stash changes")?;
        Ok(())
    }

    pub fn stash_pop(&self) -> Result<()> {
        info!("Restoring stashed changes...");
        self.execute_git(&["stash", "pop"])
            .context("Failed to restore stashed changes")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_context_display() {
        let ctx: GitContext = GitContext {
            branch: "main".to_string(),
            remote_url: "git@github.com:user/repo.git".to_string(),
            has_changes: true,
        };
        let display: String = format!("{}", ctx);
        assert!(display.contains("main"));
        assert!(display.contains("git@github.com"));
    }
}
