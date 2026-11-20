use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Manager, Runtime, WebviewWindow, WindowEvent,
};

const TRAY_ID: &str = "dsh-tray";

fn show_window<R: Runtime>(window: &WebviewWindow<R>) {
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
}

fn hide_window<R: Runtime>(window: &WebviewWindow<R>) {
    let _ = window.hide();
}

fn tray_is_available<R: Runtime>(window: &WebviewWindow<R>) -> bool {
    window
        .app_handle()
        .tray_by_id(TRAY_ID)
        .and_then(|tray| tray.rect().ok().flatten())
        .is_some()
}

/// Create the system-tray icon and menu for the wrapper.
pub fn create_tray<R: Runtime>(
    app: &tauri::AppHandle<R>,
) -> tauri::Result<tauri::tray::TrayIcon<R>> {
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let hide_i = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
    let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_i, &hide_i, &quit_i])?;

    let builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("DSH WebView")
        .menu(&menu)
        // Left-click toggles the window; right-click opens the menu.
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => {
                app.exit(0);
            }
            "hide" => {
                if let Some(window) = app.get_webview_window("main") {
                    hide_window(&window);
                }
            }
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    show_window(&window);
                }
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                ..
            }
            | TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } => {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    let visible = window.is_visible().unwrap_or(false);
                    let focused = window.is_focused().unwrap_or(false);
                    if visible && focused {
                        hide_window(&window);
                    } else {
                        show_window(&window);
                    }
                }
            }
            _ => {}
        });

    // Always provide a real image. Depending on the optional default window icon
    // can silently produce a tray without an icon if it is unavailable.
    let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png"))?;
    let tray = builder.icon(icon).build(app)?;
    tray.set_visible(true)?;

    Ok(tray)
}

/// Attach tray-related window events. Never hide the only window unless the
/// tray icon is actually registered and can be used to restore it.
pub fn attach_window_events<R: Runtime>(window: &WebviewWindow<R>) {
    let close_window = window.clone();
    let resize_window = window.clone();

    window.on_window_event(move |event| match event {
        WindowEvent::CloseRequested { api, .. } => {
            if tray_is_available(&close_window) {
                api.prevent_close();
                hide_window(&close_window);
            }
        }
        WindowEvent::Resized(_) => {
            if matches!(resize_window.is_minimized(), Ok(true))
                && tray_is_available(&resize_window)
            {
                hide_window(&resize_window);
            }
        }
        _ => {}
    });
}
