@echo off
cd /d "%~dp0"
HaverPlayer.exe project.forge.json
if errorlevel 1 pause
