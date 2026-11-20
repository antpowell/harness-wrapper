use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub dsh_endpoint: String,
    pub window_width: u32,
    pub window_height: u32,
    pub auto_connect: bool,
    pub retry_attempts: u32,
    pub retry_interval_ms: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            dsh_endpoint: "http://127.0.0.1:3080".to_string(),
            window_width: 1400,
            window_height: 900,
            auto_connect: true,
            retry_attempts: 15,
            retry_interval_ms: 2000,
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        if let Ok(config_str) = fs::read_to_string(Self::config_path()) {
            if let Ok(config) = serde_json::from_str(&config_str) {
                return config;
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), String> {
        let config_str = serde_json::to_string_pretty(self)
            .map_err(|e| e.to_string())?;
        fs::write(Self::config_path(), config_str)
            .map_err(|e| e.to_string())
    }

    fn config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("dsh-webview-wrapper");
        path.push("config.json");
        path
    }
}