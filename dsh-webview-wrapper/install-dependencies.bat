@echo off
REM Install all dependencies for DSH WebView Wrapper

echo ==========================================
echo DSH WebView Wrapper Dependency Installer
echo ==========================================
echo.

REM Check if Node.js is installed
where node >nul 2>nul
if %errorlevel% neq 0 (
    echo Node.js is not installed. Please install it from https://nodejs.org/
    echo.
    echo Opening Node.js download page...
    start https://nodejs.org/en/download/
    pause
    exit /b 1
)

echo Node.js version:
node --version

echo.
echo Installing Node.js dependencies...
call npm install
if %errorlevel% neq 0 (
    echo Error: Failed to install Node.js dependencies
    pause
    exit /b 1
)

echo.
echo Checking for Rust...
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo.
    echo Rust is not installed. Installing Rust via rustup...
    echo.
    
    REM Download rustup-init
    echo Downloading rustup-init...
    powershell -Command "Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile 'rustup-init.exe'"
    
    if not exist "rustup-init.exe" (
        echo Error: Failed to download rustup-init
        echo Please install Rust manually from https://rustup.rs/
        pause
        exit /b 1
    )
    
    echo Running rustup installer...
    rustup-init.exe -y
    
    if %errorlevel% neq 0 (
        echo Error: Rust installation failed
        del rustup-init.exe
        pause
        exit /b 1
    )
    
    del rustup-init.exe
    
    REM Refresh environment variables
    call "%USERPROFILE%\.cargo\env"
)

echo.
echo Rust version:
cargo --version

echo.
echo Installing Tauri CLI...
cargo install tauri-cli
if %errorlevel% neq 0 (
    echo Warning: Failed to install Tauri CLI
    echo You may need to install it manually: cargo install tauri-cli
)

echo.
echo ==========================================
echo Installation complete!
echo ==========================================
echo.
echo You can now run the application with:
echo   npm run tauri dev    (development mode)
echo   npm run tauri build  (production build)
echo.
pause