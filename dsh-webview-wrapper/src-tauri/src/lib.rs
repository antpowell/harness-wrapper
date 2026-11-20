// Library module for DSH WebView Wrapper
// This can be used for testing or as a library

pub mod config;
pub mod auth;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DshConfig {
    pub endpoint: String,
    pub auto_connect: bool,
    pub retry_attempts: u32,
    pub retry_interval_ms: u64,
}

impl Default for DshConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://127.0.0.1:3080".to_string(),
            auto_connect: true,
            retry_attempts: 15,
            retry_interval_ms: 2000,
        }
    }
}