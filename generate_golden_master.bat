@echo off
set SCRIPT_DIR=%~dp0
where node >nul 2>nul
if %ERRORLEVEL% equ 0 (
    node "%SCRIPT_DIR%generate_golden_master.js" %*
    exit /b %ERRORLEVEL%
)

where php >nul 2>nul
if %ERRORLEVEL% equ 0 (
    php "%SCRIPT_DIR%generate_golden_master.php" %*
    exit /b %ERRORLEVEL%
)

echo Error: Neither node nor php found on PATH >&2
exit /b 1
