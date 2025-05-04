// src/config.rs

use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf}; // <-- Add this import

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub break_interval: u64,
    pub microbreak_interval: u64,
}

impl AppConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = PathBuf::from("config.json");
        if path.exists() {
            let data = fs::read_to_string(path)?;
            let config: Self = serde_json::from_str(&data)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }
}
