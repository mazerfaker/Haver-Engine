@echo off
cd /d "%~dp0"
where cargo >nul 2>&1
if errorlevel 1 (
 echo Rust toolchain not installed. Install Rust and Visual Studio C++ Build Tools.
 pause
 exit /b 1
)
cargo check --all-targets > forge-check.log 2>&1
if errorlevel 1 goto failed
cargo test --all-targets >> forge-check.log 2>&1
if errorlevel 1 goto failed
echo Checks passed. See forge-check.log.
pause
exit /b 0
:failed
type forge-check.log
echo Checks failed. See forge-check.log.
pause
exit /b 1
