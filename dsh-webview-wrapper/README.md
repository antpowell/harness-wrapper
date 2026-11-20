# DSH WebView2 Wrapper

A native Rust/Tauri window for the local DeepSeek Harness web interface. The normal launcher starts the harness without opening a browser, reads the authenticated loopback URL from the shared harness log, and navigates the native WebView2 window to that complete URL.

## Normal startup behavior

1. `dsh-web-startup.bat` checks for a listener on port 3080 and starts `dsh-harness-only.bat` if none exists.
2. The harness launcher runs `node --import tsx/esm apps/cli/src/bin.ts web --no-open` in the harness repository and appends output to `%USERPROFILE%\.dsh\logs\web-server.log`.
3. The wrapper watches for the exact `http://127.0.0.1:3080/?token=...` URL and accepts the harness's authentication redirect during health checks.
4. The wrapper opens that URL directly in WebView2, not in an iframe or external browser.

Only loopback HTTP URLs containing a non-empty `token` query parameter are accepted. The wrapper allows one application instance; a second launch focuses the existing window. Closing the wrapper leaves the harness running. The batch launcher starts Node outside the Tauri process because direct wrapper-launched Node processes received `EPERM` when writing the normal harness profile on this machine.

## Prerequisites

- Node.js compatible with the harness repository
- Rust and Cargo
- Visual Studio C++ Build Tools
- Microsoft WebView2 Runtime

The default harness repository is `C:\Engineering\LLM\projects\deepseek-harness`.

## Build and run

From this directory:

```bat
build-wrapper.bat
run-release.bat
```

The release executable is `src-tauri\target\release\dsh-webview-wrapper.exe`. `run-release.bat` delegates to the parent `dsh-web-startup.bat` so manual and scheduled launches use the same startup path.

For a debug build or tests:

```bat
build-wrapper.bat debug
build-wrapper.bat test
```

The build script initializes the installed MSVC environment and uses a project-local temporary directory. This avoids Git's unrelated `link` executable and restricted system temporary directories.

## System tray behavior

The wrapper minimizes to the system tray instead of exiting:

- Clicking the tray icon shows or hides the main window.
- The tray menu has **Show**, **Hide**, and **Quit** items.
- Clicking the window's close button hides it to the tray.
- Minimizing the window hides it from the taskbar when the tray icon is available; restore it from the tray. If Windows blocks tray registration, the window minimizes normally and remains available on the taskbar.

To fully exit the wrapper, choose **Quit** from the tray menu.

## Install as a Windows app

From the parent project directory, run:

```powershell
powershell -ExecutionPolicy Bypass -File .\install-app.ps1
```

This builds the release executable, copies it to a permanent location, creates a Start Menu shortcut, and registers the wrapper in **Settings > Apps > Installed apps** under the current user profile. To install for all users (requires an admin PowerShell window):

```powershell
powershell -ExecutionPolicy Bypass -File .\install-app.ps1 -Machine
```

To remove the app:

```powershell
powershell -ExecutionPolicy Bypass -File .\uninstall-app.ps1
# or, for a machine-wide install:
powershell -ExecutionPolicy Bypass -File .\uninstall-app.ps1 -Machine
```

## Automatic launch at logon

From the parent project directory, run:

```powershell
powershell -ExecutionPolicy Bypass -File .\install-startup.ps1
```

This builds the release executable and creates or updates the `DeepSeekHarness Web` scheduled task. At logon, the task runs `dsh-web-startup.bat`, which starts the harness if needed and opens the native wrapper.

To remove the logon task:

```powershell
Unregister-ScheduledTask -TaskName 'DeepSeekHarness Web' -Confirm:$false
```

## Errors and logs

If automatic startup fails, the error panel shows the underlying harness error when available. **Open Harness Log** opens the active log in the default text-file editor. A complete authenticated loopback URL can also be entered manually.

## Configuration

Optional environment variables:

- `DSH_HARNESS_DIR`: harness repository path used when the wrapper starts Node directly.
- `DSH_NODE`: Node executable path used for direct startup; defaults to `C:\Program Files\Volta\node.exe`.
- `DSH_EXTERNAL_HARNESS`: when set, the wrapper waits for an externally started harness instead of spawning Node. The normal startup batch sets this.
- `DSH_WEB_LOG`: existing DSH web log used for endpoint discovery; defaults to `%USERPROFILE%\.dsh\logs\web-server.log`.
- `DSH_WEBVIEW_HARNESS_LOG`: log for a harness started directly by the wrapper; defaults to `webview-harness.log` beside the executable.
- `DSH_WEBVIEW_DATA_DIR`: WebView2 profile directory. By default, the wrapper uses `webview2-data` beside the executable, which also works under Windows Administrator Protection.
- `WEBVIEW2_USER_DATA_FOLDER`: standard WebView2 profile override; takes precedence when already set.
