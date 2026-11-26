@echo off
setlocal
cd /d "%~dp0"
where cargo >nul 2>&1 || (echo Install Rust MSVC and Visual Studio C++ Build Tools. & exit /b 1)
>forge-check.log rustc --version
>>forge-check.log cargo --version
cargo check --all-targets >>forge-check.log 2>&1 || goto failed
cargo test --all-targets >>forge-check.log 2>&1 || goto failed
cargo run --bin forge_player -- --smoke examples/signal_lost/project.forge.json >>forge-check.log 2>&1 || goto failed
echo PASS. See forge-check.log.
exit /b 0
:failed
type forge-check.log
echo FAILED. Send forge-check.log for debugging.
exit /b 1
