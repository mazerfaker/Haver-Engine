@echo off
cd /d "%~dp0"
if not exist "target\x86_64-pc-windows-msvc\debug\forge_editor.exe" (
 echo Run QUICK_BUILD.bat first.
 pause
 exit /b 1
)
"target\x86_64-pc-windows-msvc\debug\forge_editor.exe"
if errorlevel 1 pause
