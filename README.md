# DSH Harness Wrapper

A native Windows desktop wrapper for the [DeepSeek Harness](https://github.com/) local web interface. Instead of opening the harness in a browser tab, this project runs the harness headless and displays its authenticated UI in a real application window with a system-tray presence.

## What it is and what it uses

**Purpose.** DeepSeek Harness normally serves its web UI at `http://127.0.0.1:3080` and opens it in your default browser, requiring an authentication token in the URL. This wrapper turns that into a proper desktop application:

- It starts the harness with `web --no-open`, so no browser tab is opened.
- It watches the shared harness log (`%USERPROFILE%\.dsh\logs\web-server.log`) for the exact authenticated loopback URL (`http://127.0.0.1:3080/?token=...`) and accepts the harness's authentication redirect during health checks.
- It navigates a native window directly to that URL — not an iframe, not an external browser. Only loopback HTTP URLs with a non-empty `token` query parameter are accepted.
- A second launch focuses the existing window instead of starting another copy; closing the window hides it to the system tray (Show / Hide / Quit from the tray menu). Closing the wrapper leaves the harness itself running.

**Technology.**

| Layer | Technology |
| --- | --- |
| Native shell | Rust + [Tauri 2](https://v2.tauri.app) on Microsoft Edge WebView2 (`tauri` with `tray-icon`, single-instance plugin, `reqwest`/`tokio` for endpoint discovery) |
| Frontend shell | TypeScript + Vite (a minimal loading/error page; the real UI is the harness itself rendered by WebView2) |
| Harness backend | Node.js — your local DeepSeek Harness checkout, launched with `node --import tsx/esm apps/cli/src/bin.ts web --no-open` |
| Startup & install | Windows batch launchers, PowerShell install scripts, Task Scheduler for logon autostart |

## Repository layout

```
harness-wrapper/
├── dsh-web-startup.bat        # Recommended launcher: starts harness if needed, then opens the wrapper window
├── dsh-harness-only.bat       # Starts only the harness (edit paths for your machine)
├── install-app.ps1            # Builds + installs as a Windows app (Start Menu shortcut, "Installed apps" entry)
├── uninstall-app.ps1          # Removes the installed app (-Machine for a machine-wide install)
├── install-startup.ps1        # Registers the "DeepSeekHarness Web" logon task and Start Menu shortcut
└── dsh-webview-wrapper/       # The Tauri application source (Rust + WebView2)
    ├── src-tauri/             # Rust backend: endpoint discovery, auth handling, tray behavior
    ├── src/                   # TypeScript frontend shell (loading / error page)
    ├── build-wrapper.bat      # Build script (test | debug | release)
    └── README.md              # Detailed wrapper documentation (configuration, logs, tray behavior)
```

## Prerequisites

- A local **DeepSeek Harness** checkout with its dependencies installed (this project wraps it; it is not the harness itself).
- Node.js compatible with your harness checkout.
- Rust (stable MSVC toolchain) and Cargo.
- Visual Studio C++ Build Tools. `build-wrapper.bat` calls VS 2019 BuildTools' `vcvars64.bat` — adjust that line if you use a different version.
- Microsoft WebView2 Runtime (included with recent Windows installs).

### Configure for your machine (first time only)

Two batch files contain machine-specific paths. Edit them before first run:

- **`dsh-harness-only.bat`** — set the `cd /d` line to your harness checkout path, and adjust the Node/PATH line if you don't use Volta.
- **`dsh-web-startup.bat`** — set `HARNESS_DIR` to the same harness checkout path.

## How to run it correctly

### 1. Build

From this directory:

```bat
cd dsh-webview-wrapper
build-wrapper.bat
```

This runs `npm run build` (Vite) and then `cargo build --release`. The release executable is `dsh-webview-wrapper\src-tauri\target\release\dsh-webview-wrapper.exe`.

> The build script initializes the MSVC environment and uses a project-local temporary directory, which avoids Git's unrelated `link` executable and restricted system temp directories. For other modes: `build-wrapper.bat debug` or `build-wrapper.bat test`.

### 2. Launch — use the startup batch file

Always start the wrapper through **`dsh-web-startup.bat`** (in this repo root), not by double-clicking the exe:

```bat
dsh-web-startup.bat
```

It checks whether a listener already exists on port 3080, starts `dsh-harness-only.bat` from your harness checkout if not, and then opens the native window on the authenticated URL. `run-release.bat` inside the wrapper folder delegates to this same startup path.

Why not run the exe directly? The batch launcher starts Node *outside* the Tauri process because direct wrapper-launched Node processes received `EPERM` when writing the harness profile on the reference machine. Running the exe by itself only works if a harness is already running and logging to `%USERPROFILE%\.dsh\logs\web-server.log`, or if you set `DSH_EXTERNAL_HARNESS=1`.

### 3. Optional: install as a Windows app

```powershell
powershell -ExecutionPolicy Bypass -File .\install-app.ps1
# Machine-wide (admin PowerShell):
powershell -ExecutionPolicy Bypass -File .\install-app.ps1 -Machine
```

This builds the release executable, copies it to a permanent location (`%LOCALAPPDATA%\DSH WebView` by default), creates a Start Menu shortcut, and registers **DSH WebView** under *Settings → Apps → Installed apps*. Remove it with `uninstall-app.ps1` (add `-Machine` if installed machine-wide).

### 4. Optional: launch automatically at logon

```powershell
powershell -ExecutionPolicy Bypass -File .\install-startup.ps1
```

This builds the release executable and creates/updates the `DeepSeekHarness Web` scheduled task, which runs `dsh-web-startup.bat` at logon. Remove it with:

```powershell
Unregister-ScheduledTask -TaskName 'DeepSeekHarness Web' -Confirm:$false
```

## Configuration (optional environment variables)

| Variable | Meaning |
| --- | --- |
| `DSH_HARNESS_DIR` | Harness repository path used when the wrapper starts Node directly. |
| `DSH_NODE` | Node executable for direct startup; defaults to `C:\Program Files\Volta\node.exe`. |
| `DSH_EXTERNAL_HARNESS` | When set, the wrapper waits for an externally started harness instead of spawning Node. The startup batch files set this. |
| `DSH_WEB_LOG` | Existing DSH web log used for endpoint discovery; defaults to `%USERPROFILE%\.dsh\logs\web-server.log`. |
| `DSH_WEBVIEW_HARNESS_LOG` | Log for a harness started directly by the wrapper; defaults to `webview-harness.log` beside the executable. |
| `DSH_WEBVIEW_DATA_DIR` | WebView2 profile directory; defaults to `webview2-data` beside the executable (works under Windows Administrator Protection). |
| `WEBVIEW2_USER_DATA_FOLDER` | Standard WebView2 profile override; takes precedence when already set. |

## Troubleshooting

- **The window never appears.** Confirm the harness is running (`http://127.0.0.1:3080` responds) and that `%USERPROFILE%\.dsh\logs\web-server.log` contains a `token=` URL line. Startup errors are appended to `%USERPROFILE%\.dsh\logs\webview-startup.log`.
- **Error panel in the window.** If automatic startup fails, the error panel shows the underlying harness error when available; **Open Harness Log** opens the active log in your default text editor. A complete authenticated loopback URL can also be entered manually.
- **Tray icon missing / window minimized to taskbar.** If Windows blocks tray registration, the window minimizes normally and stays on the taskbar.

Further detail lives in [`dsh-webview-wrapper/README.md`](dsh-webview-wrapper/README.md) and [`dsh-webview-wrapper/BUILD_GUIDE.md`](dsh-webview-wrapper/BUILD_GUIDE.md).
