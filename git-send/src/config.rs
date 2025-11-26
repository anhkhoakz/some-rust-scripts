use crate::errors::GitSendError;
use anyhow::{Context, Result};
use clap::ArgMatches;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_COMMIT_MSG: &str = "Update: automated commit";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    pub default_msg: String,
    pub dry_run: bool,
    pub no_pull: bool,
    pub auto_stash: bool,
    pub no_push: bool,
    pub verbose: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_msg: DEFAULT_COMMIT_MSG.to_owned(),
            dry_run: false,
            no_pull: false,
            no_push: false,
            auto_stash: false,
            verbose: false,
        }
    }
}

impl Config {
    /// Load configuration from a file with robust error handling
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            debug!("Config file not found at {path:?}, using defaults");
            return Ok(Self::default());
        }

        let content: String = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {path:?}"))?;

        Self::parse_config(&content)
    }

    /// Parse configuration content with validation
    pub fn parse_config(content: &str) -> Result<Self> {
        let mut cfg: Config = Self::default();

        for raw_line in content.lines() {
            let line: &str = raw_line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let split: Option<(&str, &str)> = line.split_once('=');
            let (key, val) = split.ok_or_else(|| {
                GitSendError::ConfigError(format!("Invalid configuration line: {line}"))
            })?;

            match key {
                "default_msg" => cfg.default_msg = val.to_string(),
                "dry_run" => cfg.dry_run = Self::parse_bool(key, val)?,
                "no_pull" => cfg.no_pull = Self::parse_bool(key, val)?,
                "no_push" => cfg.no_push = Self::parse_bool(key, val)?,
                "auto_stash" => cfg.auto_stash = Self::parse_bool(key, val)?,
                "verbose" => cfg.verbose = Self::parse_bool(key, val)?,
                _ => warn!("Unknown configuration key: {}", key),
            }
        }

        Ok(cfg)
    }

    pub fn parse_bool(key: &str, val: &str) -> Result<bool> {
        match val {
            "1" | "true" | "yes" | "on" => Ok(true),
            "0" | "false" | "no" | "off" => Ok(false),
            _ => Err(GitSendError::InvalidConfigValue {
                key: key.to_string(),
                value: val.to_string(),
            }
            .into()),
        }
    }

    /// Save configuration to file
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        let content: String = format!(
            "# git-send configuration file
# Boolean values: 1/true/yes/on or 0/false/no/off

default_msg={}
dry_run={}
no_pull={}
no_push={}
auto_stash={}
verbose={}
",
            self.default_msg,
            self.dry_run,
            self.no_pull,
            self.no_push,
            self.auto_stash,
            self.verbose
        );

        fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;

        Ok(())
    }

    pub fn merge_with_cli(self, matches: &ArgMatches) -> Config {
        Config {
            default_msg: env::var("GIT_SEND_DEFAULT_MSG")
                .ok()
                .unwrap_or(self.default_msg),
            dry_run: matches.get_flag("dry_run")
                || env::var("GIT_SEND_DRY_RUN")
                    .map(|v: String| v == "1" || v.eq_ignore_ascii_case("true"))
                    .unwrap_or(false)
                || self.dry_run,
            no_pull: matches.get_flag("no_pull")
                || env::var("GIT_SEND_NO_PULL")
                    .map(|v: String| v == "1" || v.eq_ignore_ascii_case("true"))
                    .unwrap_or(false)
                || self.no_pull,
            no_push: matches.get_flag("no_push")
                || env::var("GIT_SEND_NO_PUSH")
                    .map(|v: String| v == "1" || v.eq_ignore_ascii_case("true"))
                    .unwrap_or(false)
                || self.no_push,
            auto_stash: self.auto_stash,
            verbose: matches.get_flag("verbose") || self.verbose,
        }
    }
}

fn config_file_debug_log(path: &Path) {
    log::debug!("Resolving config from {}", path.display());
}

pub fn resolve_config_path(matches: &ArgMatches, app_name: &str) -> PathBuf {
    if let Some(path) = matches.get_one::<String>("config") {
        let custom = PathBuf::from(path);
        config_file_debug_log(&custom);
        return custom;
    }

    let mut path: PathBuf = dirs::config_dir().unwrap_or_else(|| {
        warn!("Could not determine config directory, using current directory");
        PathBuf::from(".")
    });
    path.push(app_name);
    path.push("config");
    config_file_debug_log(&path);
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let cfg: Config = Config::default();
        assert_eq!(cfg.default_msg, DEFAULT_COMMIT_MSG);
        assert!(!cfg.dry_run);
        assert!(!cfg.no_pull);
        assert!(!cfg.no_push);
    }

    #[test]
    fn test_config_parse_bool() {
        assert!(Config::parse_bool("test", "1").unwrap());
        assert!(Config::parse_bool("test", "true").unwrap());
        assert!(Config::parse_bool("test", "yes").unwrap());
        assert!(!Config::parse_bool("test", "0").unwrap());
        assert!(!Config::parse_bool("test", "false").unwrap());
        assert!(Config::parse_bool("test", "invalid").is_err());
    }

    #[test]
    fn test_config_parse() {
        let content: &str = r#"
            # Comment
            default_msg=Test message
            dry_run=true
            no_pull=1
            no_push=false
        "#;

        let cfg = Config::parse_config(content).unwrap();
        assert_eq!(cfg.default_msg, "Test message");
        assert!(cfg.dry_run);
        assert!(cfg.no_pull);
        assert!(!cfg.no_push);
    }
}
