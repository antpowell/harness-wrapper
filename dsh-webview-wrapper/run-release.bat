@echo off
cd /d "%~dp0"
REM DSH WebView Wrapper - Run Release Build
REM This script runs the release build without creating an installer

echo ==========================================
echo DSH WebView Wrapper - Release Mode
echo ==========================================
echo.

REM Check if the release executable exists
set EXE_PATH=src-tauri\target\release\dsh-webview-wrapper.exe

if not exist "%EXE_PATH%" (
    echo Release executable not found! Building it now...
    echo.
    call build-wrapper.bat
    if not exist "%EXE_PATH%" (
        echo Build failed! Please check the error messages above.
        pause
        exit /b 1
    )
)

echo Found executable: %EXE_PATH%
echo.
echo Starting DSH WebView Wrapper...
echo.

REM Start the harness if needed, then run the executable
call "%~dp0..\dsh-web-startup.bat"

echo.
echo Application started!
echo.
echo Note: If you see "Access is denied" errors, this is due to sandbox restrictions.
echo The application requires window creation permissions to run.
echo.
pause