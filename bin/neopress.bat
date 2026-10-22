@echo off
set SCRIPT_DIR=%~dp0..
if exist "%SCRIPT_DIR%\target\debug\neopress.exe" (
    "%SCRIPT_DIR%\target\debug\neopress.exe" %*
) else (
    cargo run -p neopress-server --bin neopress -- %*
)
