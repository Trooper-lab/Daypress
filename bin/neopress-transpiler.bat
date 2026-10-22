@echo off
set SCRIPT_DIR=%~dp0..
if exist "%SCRIPT_DIR%\target\debug\neopress-transpiler.exe" (
    "%SCRIPT_DIR%\target\debug\neopress-transpiler.exe" %*
) else (
    cargo run -p neopress-transpiler --bin neopress-transpiler -- %*
)
