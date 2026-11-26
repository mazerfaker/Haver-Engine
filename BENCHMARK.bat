@echo off
cd /d "%~dp0"
if not exist dist\HaverPlayer.exe (echo Build first. & pause & exit /b 1)
dist\HaverPlayer.exe --benchmark examples\signal_lost\project.forge.json > benchmark.json
if errorlevel 1 (echo Benchmark failed. & pause & exit /b 1)
type benchmark.json
pause
