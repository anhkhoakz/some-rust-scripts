use anyhow::Result;
use git2::{Branch, BranchType, Repository};
use std::fmt;

#[allow(dead_code)]
pub struct GitFlow {
    repo: Repository,
}

impl fmt::Debug for GitFlow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GitFlow").finish_non_exhaustive()
    }
}

#[allow(dead_code)]
impl GitFlow {
    pub fn new() -> Result<Self> {
        let repo: Repository = Repository::open(".")?;
        Ok(Self { repo })
    }

    pub fn create_branch(&self, name: &str, base: &str) -> Result<Branch> {
        let base_commit: git2::Commit<'_> = self
            .repo
            .find_branch(base, BranchType::Local)?
            .get()
            .peel_to_commit()?;
        let branch: Branch = self.repo.branch(name, &base_commit, false)?;
        Ok(branch)
    }

    pub fn delete_branch(&self, name: &str) -> Result<()> {
        let mut branch: Branch = self.repo.find_branch(name, BranchType::Local)?;
        branch.delete()?;
        Ok(())
    }

    pub fn checkout_branch(&self, name: &str) -> Result<()> {
        let branch: Branch = self.repo.find_branch(name, BranchType::Local)?;
        let commit: git2::Commit<'_> = branch.get().peel_to_commit()?;
        self.repo.checkout_tree(&commit.as_object(), None)?;
        self.repo.set_head(&format!("refs/heads/{}", name))?;
        Ok(())
    }

    pub fn merge_branch(&self, source: &str, target: &str) -> Result<()> {
        let source_branch: Branch = self.repo.find_branch(source, BranchType::Local)?;
        let _target_branch: Branch = self.repo.find_branch(target, BranchType::Local)?;

        let source_commit: git2::Commit<'_> = source_branch.get().peel_to_commit()?;
        self.checkout_branch(target)?;

        let annotated_source: git2::AnnotatedCommit<'_> =
            self.repo.find_annotated_commit(source_commit.id())?;
        self.repo.merge(&[&annotated_source], None, None)?;
        Ok(())
    }
}
