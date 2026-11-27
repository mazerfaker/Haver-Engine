param([ValidateRange(1,65536)][int]$Instances=1024,[switch]$GpuProfile)
# Manual hardware verification, never a claim of runtime success from cargo check.
$ErrorActionPreference='Stop'
Set-Location (Join-Path $PSScriptRoot '..')
$env:RUST_LOG='info,wgpu_core=warn,wgpu_hal=warn'
& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'prepare_lock.ps1')
if ($LASTEXITCODE -ne 0) { throw 'Lock migration failed' }
$env:HAVER_GPU_INSTANCES="$Instances"
if ($GpuProfile) { $env:FORGE_GPU_PROFILE='1' } else { Remove-Item Env:FORGE_GPU_PROFILE -ErrorAction SilentlyContinue }
Write-Host "Observe $Instances moving particles; check for shader validation errors, corruption, blank output and device loss. Close the window to finish."
Write-Host 'Logged frame diagnostics are not GPU frame timings. GPU timestamps are optional and adapter-dependent.'
$old=$ErrorActionPreference
try {
    $ErrorActionPreference='Continue'
    & cargo run --locked --release --no-default-features --features gpu-instancing-lab --example gpu_instancing_lab 2>&1 | Tee-Object -FilePath "haver-gpu-instancing-$Instances.log"
    $code=$LASTEXITCODE
} finally { $ErrorActionPreference=$old }
if ($code -ne 0) { throw "GPU lab exited $code" }
Write-Host 'Process exited successfully; record visual results and hardware separately. No automatic visual validation.'
