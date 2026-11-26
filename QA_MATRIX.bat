@echo off
cd /d "%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File tools\qa_matrix.ps1
exit /b %errorlevel%
