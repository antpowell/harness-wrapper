use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthState {
    pub endpoint_url: Option<String>,
    pub is_authenticated: bool,
    pub auth_token: Option<String>,
}

impl Default for AuthState {
    fn default() -> Self {
        Self {
            endpoint_url: None,
            is_authenticated: false,
            auth_token: None,
        }
    }
}

impl AuthState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_endpoint(&mut self, url: String) {
        self.endpoint_url = Some(url);
    }

    pub fn get_endpoint(&self) -> Option<&String> {
        self.endpoint_url.as_ref()
    }

    pub fn clear(&mut self) {
        self.endpoint_url = None;
        self.is_authenticated = false;
        self.auth_token = None;
    }
}