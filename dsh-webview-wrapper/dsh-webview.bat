@echo off
cd /d "%~dp0"
REM DSH WebView Wrapper Launcher
REM This script starts the DSH WebView wrapper application

echo ==========================================
echo DSH WebView Wrapper Launcher
echo ==========================================

REM Check if Node.js is installed
where node >nul 2>nul
if %errorlevel% neq 0 (
    echo Error: Node.js is not installed or not in PATH
    echo Please install Node.js from https://nodejs.org/
    pause
    exit /b 1
)

REM Check if npm packages are installed
if not exist "node_modules" (
    echo Installing dependencies...
    call npm install
    if %errorlevel% neq 0 (
        echo Error: Failed to install dependencies
        pause
        exit /b 1
    )
)

REM Check if Rust is installed
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo Warning: Rust is not installed
    echo Please install Rust from https://rustup.rs/
    echo.
    echo Opening installation instructions...
    start https://www.rust-lang.org/tools/install
    pause
    exit /b 1
)

REM Start the application
echo Starting DSH WebView Wrapper...
echo.

REM Check if we're in development or production mode
if exist "src-tauri\target\release\dsh-webview-wrapper.exe" (
    echo Running production build...
    start src-tauri\target\release\dsh-webview-wrapper.exe
) else (
    echo Running in development mode...
    echo This may take a while on first run as Rust compiles...
    call npm run tauri dev
)

echo.
echo DSH WebView Wrapper started!
echo.