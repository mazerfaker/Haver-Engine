@echo off
cd /d "%~dp0"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0tools\build_windows.ps1" -Profile development
set "RESULT=%ERRORLEVEL%"
pause
exit /b %RESULT%
