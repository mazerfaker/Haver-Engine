$ErrorActionPreference='Stop'
Set-Location (Join-Path $PSScriptRoot '..')
$log=Join-Path (Get-Location).Path 'forge-quick-check.log'
Set-Content -LiteralPath $log -Encoding UTF8 -Value "Forge 0.7.16 quick source check | $(Get-Date -Format o)"
& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'prepare_lock.ps1')
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$ErrorActionPreference='Continue'
& cargo check --locked --all-targets --target x86_64-pc-windows-msvc 2>&1 | ForEach-Object { Write-Host "$_"; Add-Content -LiteralPath $log -Encoding UTF8 -Value "$_" }
$code=$LASTEXITCODE
Add-Content -LiteralPath $log -Encoding UTF8 -Value "QUICK CHECK EXIT: $code"
exit $code
