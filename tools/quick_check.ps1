$ErrorActionPreference='Stop'
Set-Location (Join-Path $PSScriptRoot '..')
$log=Join-Path (Get-Location).Path 'forge-quick-check.log'
Set-Content -LiteralPath $log -Encoding UTF8 -Value "Forge 0.7.34 quick source check | $(Get-Date -Format o)"
& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'prepare_lock.ps1')
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$ErrorActionPreference='Continue'
& cargo check --locked --all-targets --features gpu-lab --target x86_64-pc-windows-msvc 2>&1 | ForEach-Object { if ($_ -is [System.Management.Automation.ErrorRecord]) { $line=$_.Exception.Message } else { $line="$_" }; Write-Host $line; Add-Content -LiteralPath $log -Encoding UTF8 -Value $line }
$code=$LASTEXITCODE
if ($code -eq 0) {
  & cargo check --locked --no-default-features --bin forge_player --target x86_64-pc-windows-msvc 2>&1 | ForEach-Object { if ($_ -is [System.Management.Automation.ErrorRecord]) { $line=$_.Exception.Message } else { $line="$_" }; Write-Host $line; Add-Content -LiteralPath $log -Encoding UTF8 -Value $line }
  $code=$LASTEXITCODE
}
Add-Content -LiteralPath $log -Encoding UTF8 -Value "QUICK CHECK EXIT: $code"
exit $code
