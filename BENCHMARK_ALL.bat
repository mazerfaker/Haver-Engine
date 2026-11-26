@echo off
cd /d "%~dp0"
if not exist dist\ForgePlayer.exe (echo Build first. & pause & exit /b 1)
if not exist dist\BUILD_OK.txt (echo No verified build marker. & pause & exit /b 1)
if not exist benchmarks mkdir benchmarks
for %%L in (project level2 level3 level4) do (
  dist\ForgePlayer.exe --benchmark "examples\signal_lost\%%L.forge.json" > "benchmarks\%%L.json"
  if errorlevel 1 (echo Benchmark %%L failed. & pause & exit /b 1)
)
copy /Y dist\build-info.json benchmarks\build-info.json >nul
if errorlevel 1 (echo Could not copy build metadata. & pause & exit /b 1)
echo Benchmark files saved in benchmarks. Save a copy before changing build profiles.
pause
