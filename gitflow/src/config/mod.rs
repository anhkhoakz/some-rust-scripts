use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct GitFlowConfig {
    pub main_branch: String,
    pub develop_branch: String,
    pub feature_prefix: String,
    pub release_prefix: String,
    pub hotfix_prefix: String,
    pub support_prefix: String,
    pub version_tag_prefix: String,
}

impl Default for GitFlowConfig {
    fn default() -> Self {
        Self {
            main_branch: "main".to_string(),
            develop_branch: "develop".to_string(),
            feature_prefix: "feature/".to_string(),
            release_prefix: "release/".to_string(),
            hotfix_prefix: "hotfix/".to_string(),
            support_prefix: "support/".to_string(),
            version_tag_prefix: "v".to_string(),
        }
    }
}

#[allow(dead_code)]
pub fn get_config_path() -> PathBuf {
    PathBuf::from(".git/gitflow-config.json")
}

#[allow(dead_code)]
pub fn load_config() -> anyhow::Result<GitFlowConfig> {
    Ok(GitFlowConfig::default())
}

#[allow(dead_code)]
pub fn save_config(_config: &GitFlowConfig) -> anyhow::Result<()> {
    Ok(())
}
