@echo off
set SCRIPT_DIR=%~dp0..
if exist "%SCRIPT_DIR%\target\debug\daypress-transpiler.exe" (
    "%SCRIPT_DIR%\target\debug\daypress-transpiler.exe" %*
) else (
    cargo run -p daypress-transpiler --bin daypress-transpiler -- %*
)
