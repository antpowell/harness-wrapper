@echo off
rem DeepSeek Harness and native WebView launcher (Task Scheduler)
setlocal
set "ROOT=%~dp0"
set "WRAPPER_DIR=%ROOT%dsh-webview-wrapper"
set "HARNESS_DIR=C:\Engineering\LLM\projects\deepseek-harness"
set "EXE_PATH=%WRAPPER_DIR%\src-tauri\target\release\dsh-webview-wrapper.exe"
set "DSH_EXTERNAL_HARNESS=1"

if not exist "%EXE_PATH%" (
    if not exist "%USERPROFILE%\.dsh\logs" mkdir "%USERPROFILE%\.dsh\logs"
    >> "%USERPROFILE%\.dsh\logs\webview-startup.log" echo [%DATE% %TIME%] Release executable not found: %EXE_PATH%
    exit /b 1
)

powershell.exe -NoProfile -Command "if (Get-NetTCPConnection -LocalPort 3080 -State Listen -ErrorAction SilentlyContinue) { exit 0 } else { exit 1 }"
if errorlevel 1 (
    start "" /B /D "%HARNESS_DIR%" cmd.exe /D /C "%ROOT%dsh-harness-only.bat"
)

start "" "%EXE_PATH%"
