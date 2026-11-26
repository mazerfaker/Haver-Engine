$ErrorActionPreference='Stop'
Set-Location (Join-Path $PSScriptRoot '..')
$log=Join-Path (Get-Location).Path 'forge-gpu-check.log'
Set-Content -LiteralPath $log -Encoding UTF8 -Value "Forge 0.7.15 GPU lab source check | $(Get-Date -Format o)"
if (!(Test-Path Cargo.lock)) { & cargo generate-lockfile; if ($LASTEXITCODE -ne 0) { exit 1 } }
$ErrorActionPreference='Continue'
& cargo check --locked --no-default-features --features gpu-lab --example gpu_lighting_lab --target x86_64-pc-windows-msvc 2>&1 | ForEach-Object { Write-Host "$_"; Add-Content -LiteralPath $log -Encoding UTF8 -Value "$_" }
$code=$LASTEXITCODE
Add-Content -LiteralPath $log -Encoding UTF8 -Value "GPU LAB CARGO EXIT: $code. WGSL runtime and visual output not tested."
exit $code
