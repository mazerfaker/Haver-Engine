# Manual hardware verification, never a claim of runtime success from cargo check.
$ErrorActionPreference='Stop'
Set-Location (Join-Path $PSScriptRoot '..')
$env:RUST_LOG='info,wgpu_core=warn,wgpu_hal=warn'
& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'prepare_lock.ps1')
if ($LASTEXITCODE -ne 0) { throw 'Lock migration failed' }
Write-Host 'Observe 1024 moving particles; check for shader validation errors, corruption, blank output and device loss. Close the window to finish.'
$old=$ErrorActionPreference
try {
    $ErrorActionPreference='Continue'
    & cargo run --locked --release --no-default-features --features gpu-instancing-lab --example gpu_instancing_lab 2>&1 | Tee-Object -FilePath 'haver-gpu-instancing.log'
    $code=$LASTEXITCODE
} finally { $ErrorActionPreference=$old }
if ($code -ne 0) { throw "GPU lab exited $code" }
Write-Host 'Process exited successfully; record visual results and hardware separately. No automatic visual validation.'
