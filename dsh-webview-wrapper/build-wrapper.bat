@echo off
setlocal
cd /d "%~dp0"
if not exist .tmp mkdir .tmp
set "TEMP=%CD%\.tmp"
set "TMP=%CD%\.tmp"
call "C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"

if "%~1"=="" goto release
if /i "%~1"=="test" goto test
if /i "%~1"=="debug" goto debug

echo Usage: build-wrapper.bat [test^|debug^|release]
exit /b 2

:test
cd /d src-tauri || exit /b 1
cargo.exe test
exit /b %ERRORLEVEL%

:debug
cd /d src-tauri || exit /b 1
cargo.exe build
exit /b %ERRORLEVEL%

:release
call npm.cmd run build || exit /b 1
cd /d src-tauri || exit /b 1
cargo.exe build --release
exit /b %ERRORLEVEL%
