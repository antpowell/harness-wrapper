@echo off
setlocal
set "PATH=C:\Program Files\Volta;%PATH%"
cd /d "C:\Engineering\LLM\projects\deepseek-harness" || exit /b 1
if not exist "%USERPROFILE%\.dsh\logs" mkdir "%USERPROFILE%\.dsh\logs"
node --import tsx/esm apps/cli/src/bin.ts web --no-open >> "%USERPROFILE%\.dsh\logs\web-server.log" 2>&1
