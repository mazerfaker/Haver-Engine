@echo off
cd /d "%~dp0"
ForgePlayer.exe project.forge.json
if errorlevel 1 pause
