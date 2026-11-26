$ErrorActionPreference='Stop'
Set-Location (Join-Path $PSScriptRoot '..')
$log=Join-Path (Get-Location).Path 'forge-gpu-check.log'
Set-Content -LiteralPath $log -Encoding UTF8 -Value "Forge 0.7.16 GPU lab source check | $(Get-Date -Format o)"
& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'prepare_lock.ps1')
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$ErrorActionPreference='Continue'
& cargo check --locked --no-default-features --features gpu-lab --examples --target x86_64-pc-windows-msvc 2>&1 | ForEach-Object { Write-Host "$_"; Add-Content -LiteralPath $log -Encoding UTF8 -Value "$_" }
$code=$LASTEXITCODE
Add-Content -LiteralPath $log -Encoding UTF8 -Value "GPU LAB CARGO EXIT: $code. WGSL runtime and visual output not tested."
exit $code
