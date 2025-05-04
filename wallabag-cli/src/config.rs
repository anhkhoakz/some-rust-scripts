use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub base_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub username: String,
    pub password: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

impl Config {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("wallabag-cli/config.json")
    }

    pub fn load() -> Option<Self> {
        let path: PathBuf = Self::config_path();
        match fs::read_to_string(&path) {
            Ok(data) => match serde_json::from_str::<Self>(&data) {
                Ok(config) => Some(config),
                Err(e) => {
                    eprintln!("Failed to parse config: {}", e);
                    None
                }
            },
            Err(e) => {
                eprintln!("Config file not found or unreadable: {}", e);
                None
            }
        }
    }

    pub fn save(&self) {
        let path: PathBuf = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(path, serde_json::to_string_pretty(self).unwrap());
    }
}
