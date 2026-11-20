use std::{
    env, fs,
    fs::{File, OpenOptions},
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

use tokio::sync::Mutex;
use url::Url;

const STARTUP_TIMEOUT: Duration = Duration::from_secs(90);
const STARTUP_POLL_INTERVAL: Duration = Duration::from_millis(250);
const DEFAULT_HARNESS_DIR: &str = r"C:\Engineering\LLM\projects\deepseek-harness";
const DEFAULT_NODE: &str = r"C:\Program Files\Volta\node.exe";

struct HarnessProcess {
    child: Child,
    endpoint: Option<String>,
}

pub struct HarnessManager {
    process: Mutex<Option<HarnessProcess>>,
}

impl HarnessManager {
    pub fn new() -> Self {
        Self {
            process: Mutex::new(None),
        }
    }

    pub async fn start_or_connect(&self) -> Result<String, String> {
        let mut process = self.process.lock().await;

        if let Some(running) = process.as_mut() {
            match running.child.try_wait() {
                Ok(None) => {
                    if let Some(endpoint) = &running.endpoint {
                        return Ok(endpoint.clone());
                    }
                }
                Ok(Some(_)) | Err(_) => *process = None,
            }
        }

        if let Some(endpoint) = discover_running_endpoint().await {
            return Ok(endpoint);
        }

        if env::var_os("DSH_EXTERNAL_HARNESS").is_some() {
            let started = Instant::now();
            loop {
                if let Some(endpoint) = discover_running_endpoint().await {
                    return Ok(endpoint);
                }
                if started.elapsed() >= STARTUP_TIMEOUT {
                    return Err(format!(
                        "Timed out waiting for the external harness authenticated URL. Open the harness log for details."
                    ));
                }
                tokio::time::sleep(STARTUP_POLL_INTERVAL).await;
            }
        }

        let log_path = harness_log_path();
        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Could not create the harness log directory: {e}"))?;
        }

        let stderr = open_log(&log_path)?;
        let stdout = stderr
            .try_clone()
            .map_err(|e| format!("Could not prepare harness logging: {e}"))?;
        let harness_dir = env::var_os("DSH_HARNESS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_HARNESS_DIR));
        let node = env::var_os("DSH_NODE").unwrap_or_else(|| DEFAULT_NODE.into());

        #[cfg(target_os = "windows")]
        let mut command = {
            let mut command = Command::new("cmd");
            command.args(["/D", "/C"]).arg(node);
            command
        };
        #[cfg(not(target_os = "windows"))]
        let mut command = Command::new(node);

        let mut child = command
            .args([
                "--import",
                "tsx/esm",
                "apps/cli/src/bin.ts",
                "web",
                "--no-open",
            ])
            .current_dir(&harness_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .spawn()
            .map_err(|e| {
                format!(
                    "Failed to start the harness with Node in {}: {e}",
                    harness_dir.display()
                )
            })?;

        let started = Instant::now();
        loop {
            if let Some(endpoint) = endpoints_in_log(&log_path).into_iter().next() {
                *process = Some(HarnessProcess {
                    child,
                    endpoint: Some(endpoint.clone()),
                });
                return Ok(endpoint);
            }

            match child.try_wait() {
                Ok(Some(status)) => {
                    *process = None;
                    let detail = log_error_summary(&log_path)
                        .map(|summary| format!(" Last error: {summary}"))
                        .unwrap_or_default();
                    return Err(format!(
                        "The harness exited with {status} before reporting its authenticated URL.{detail}"
                    ));
                }
                Err(e) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    *process = None;
                    return Err(format!("Could not monitor the harness process: {e}"));
                }
                Ok(None) => {}
            }

            if started.elapsed() >= STARTUP_TIMEOUT {
                let _ = child.kill();
                let _ = child.wait();
                *process = None;
                return Err(format!(
                    "Timed out waiting for the harness authenticated URL. See {}.",
                    log_path.display()
                ));
            }

            tokio::time::sleep(STARTUP_POLL_INTERVAL).await;
        }
    }
}

fn open_log(path: &PathBuf) -> Result<File, String> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .map_err(|e| format!("Could not open harness log {}: {e}", path.display()))
}

pub fn harness_log_path() -> PathBuf {
    if let Some(path) = env::var_os("DSH_WEBVIEW_HARNESS_LOG") {
        return PathBuf::from(path);
    }

    env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("webview-harness.log")))
        .unwrap_or_else(|| PathBuf::from("webview-harness.log"))
}

pub fn ensure_harness_log() -> Result<PathBuf, String> {
    let path = if env::var_os("DSH_EXTERNAL_HARNESS").is_some() {
        existing_server_log_path()
    } else {
        harness_log_path()
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Could not create the harness log directory: {e}"))?;
    }
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("Could not open harness log {}: {e}", path.display()))?;
    Ok(path)
}

fn log_error_summary(path: &PathBuf) -> Option<String> {
    let contents = fs::read_to_string(path).ok()?;
    contents
        .lines()
        .map(str::trim)
        .filter(|line| {
            line.contains("Error:") || line.contains("EPERM") || line.contains("fatal")
        })
        .next_back()
        .map(|line| line.chars().take(500).collect())
}

fn existing_server_log_path() -> PathBuf {
    if let Some(path) = env::var_os("DSH_WEB_LOG") {
        return PathBuf::from(path);
    }

    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".dsh")
        .join("logs")
        .join("web-server.log")
}

async fn discover_running_endpoint() -> Option<String> {
    for log_path in [harness_log_path(), existing_server_log_path()] {
        for endpoint in endpoints_in_log(&log_path).into_iter().take(10) {
            if check_endpoint(&endpoint).await {
                return Some(endpoint);
            }
        }
    }
    None
}

fn endpoints_in_log(path: &PathBuf) -> Vec<String> {
    let Ok(contents) = fs::read_to_string(path) else {
        return Vec::new();
    };

    let mut endpoints: Vec<String> = contents
        .lines()
        .rev()
        .filter_map(|line| {
            line.split_whitespace()
                .map(|part| part.trim_matches(|c| matches!(c, '\'' | '"' | '(' | ')' | ',' | ';')))
                .find_map(|part| validate_endpoint(part).ok())
        })
        .collect();
    endpoints.dedup();
    endpoints
}

pub fn validate_endpoint(endpoint: &str) -> Result<String, String> {
    let url = Url::parse(endpoint).map_err(|_| "Enter a valid harness URL.".to_string())?;
    if url.scheme() != "http" {
        return Err("The harness URL must use HTTP on the local machine.".to_string());
    }

    let local_host = match url.host_str() {
        Some("127.0.0.1" | "localhost" | "::1") => true,
        _ => false,
    };
    if !local_host {
        return Err("Only loopback harness URLs are allowed.".to_string());
    }

    let has_token = url
        .query_pairs()
        .any(|(key, value)| key == "token" && !value.is_empty());
    if !has_token {
        return Err("The harness URL must include its authentication token.".to_string());
    }

    Ok(url.into())
}

pub async fn check_endpoint(endpoint: &str) -> bool {
    if validate_endpoint(endpoint).is_err() {
        return false;
    }

    let Ok(client) = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .redirect(reqwest::redirect::Policy::none())
        .build()
    else {
        return false;
    };

    client
        .get(endpoint)
        .send()
        .await
        .map(|response| response.status().is_success() || response.status().is_redirection())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::validate_endpoint;

    #[test]
    fn accepts_loopback_authenticated_urls() {
        assert!(validate_endpoint("http://127.0.0.1:3080/?token=test-token").is_ok());
        assert!(validate_endpoint("http://localhost:3080/?token=test-token").is_ok());
    }

    #[test]
    fn rejects_remote_or_unauthenticated_urls() {
        assert!(validate_endpoint("http://example.com/?token=test-token").is_err());
        assert!(validate_endpoint("http://127.0.0.1:3080/").is_err());
        assert!(validate_endpoint("https://127.0.0.1:3080/?token=test-token").is_err());
    }
}
