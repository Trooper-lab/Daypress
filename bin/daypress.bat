@echo off
set SCRIPT_DIR=%~dp0..
if exist "%SCRIPT_DIR%\target\debug\daypress.exe" (
    "%SCRIPT_DIR%\target\debug\daypress.exe" %*
) else (
    cargo run -p daypress-server --bin daypress -- %*
)
