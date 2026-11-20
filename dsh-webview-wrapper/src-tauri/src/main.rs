// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use tauri::Manager;
use tokio::sync::Mutex;

mod auth;
mod harness;
mod tray;

use auth::AuthState;
use harness::HarnessManager;

struct AppState {
    auth: Arc<Mutex<AuthState>>,
    harness: HarnessManager,
    // The tray icon must be kept alive for the lifetime of the app.
    tray: std::sync::Mutex<Option<tauri::tray::TrayIcon<tauri::Wry>>>,
}

#[tauri::command]
async fn set_auth_url(url: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let url = harness::validate_endpoint(&url)?;
    let mut auth = state.auth.lock().await;
    auth.set_endpoint(url);
    Ok(())
}

#[tauri::command]
async fn get_auth_url(state: tauri::State<'_, AppState>) -> Result<Option<String>, String> {
    let auth = state.auth.lock().await;
    Ok(auth.get_endpoint().cloned())
}

#[tauri::command]
async fn check_endpoint(url: String) -> Result<bool, String> {
    Ok(harness::check_endpoint(&url).await)
}

#[tauri::command]
fn get_harness_log_path() -> Result<String, String> {
    harness::ensure_harness_log().map(|path| path.to_string_lossy().into_owned())
}

#[tauri::command]
async fn start_dsh_and_get_url(
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    state.harness.start_or_connect().await
}

#[tauri::command]
async fn open_endpoint(window: tauri::WebviewWindow, url: String) -> Result<(), String> {
    let url = harness::validate_endpoint(&url)?;
    let parsed = url
        .parse()
        .map_err(|e| format!("Could not parse the harness URL: {e}"))?;
    window
        .navigate(parsed)
        .map_err(|e| format!("Could not open the harness: {e}"))
}

#[cfg(target_os = "windows")]
fn configure_webview_data_directory() {
    use std::{env, fs, path::PathBuf};

    if env::var_os("WEBVIEW2_USER_DATA_FOLDER").is_some() {
        return;
    }

    let data_dir = env::var_os("DSH_WEBVIEW_DATA_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(|dir| dir.join("webview2-data")))
        });

    let Some(data_dir) = data_dir else {
        eprintln!("Could not determine the WebView2 data directory");
        return;
    };

    if let Err(e) = fs::create_dir_all(&data_dir) {
        eprintln!("Could not create WebView2 data directory: {e}");
        return;
    }

    env::set_var("WEBVIEW2_USER_DATA_FOLDER", data_dir);
}

#[cfg(not(target_os = "windows"))]
fn configure_webview_data_directory() {}

fn main() {
    configure_webview_data_directory();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            auth: Arc::new(Mutex::new(AuthState::new())),
            harness: HarnessManager::new(),
            tray: std::sync::Mutex::new(None),
        })
        .setup(|app| {
            let tray = tray::create_tray(app.handle())?;
            if let Some(window) = app.get_webview_window("main") {
                tray::attach_window_events(&window);
            }
            *app.state::<AppState>().tray.lock().unwrap() = Some(tray);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            set_auth_url,
            get_auth_url,
            check_endpoint,
            get_harness_log_path,
            start_dsh_and_get_url,
            open_endpoint
        ]);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        builder.run(tauri::generate_context!())
    }));
    let error = match result {
        Ok(Ok(())) => return,
        Ok(Err(error)) => error.to_string(),
        Err(payload) => payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_else(|| "Unknown WebView2 initialization error".to_string()),
    };

    let message = format!(
        "DSH WebView could not start.\n\n{error}\n\nClose other DSH WebView instances and try again."
    );
    eprintln!("{message}");
    let _ = rfd::MessageDialog::new()
        .set_title("DSH WebView startup failed")
        .set_description(&message)
        .set_level(rfd::MessageLevel::Error)
        .show();
}
