$ErrorActionPreference='Stop'
Set-Location (Join-Path $PSScriptRoot '..')
$log=Join-Path (Get-Location).Path 'forge-quick-test.log'
Set-Content -LiteralPath $log -Encoding UTF8 -Value "Forge 0.7.36 regression tests | $(Get-Date -Format o)"
& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'prepare_lock.ps1')
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$ErrorActionPreference='Continue'
& cargo test --locked --all-targets --features gpu-lab --target x86_64-pc-windows-msvc 2>&1 | ForEach-Object { if ($_ -is [System.Management.Automation.ErrorRecord]) { $line=$_.Exception.Message } else { $line="$_" }; Write-Host $line; Add-Content -LiteralPath $log -Encoding UTF8 -Value $line }
$code=$LASTEXITCODE
Add-Content -LiteralPath $log -Encoding UTF8 -Value "QUICK TEST EXIT: $code"
exit $code
