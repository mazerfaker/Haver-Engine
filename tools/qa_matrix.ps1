$ErrorActionPreference='Stop'
Set-Location (Join-Path $PSScriptRoot '..')
& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'prepare_lock.ps1')
if ($LASTEXITCODE -ne 0) { throw 'Lockfile migration failed' }
$sets=@('','physics-rapier','lua','audio','lighting','physics-rapier,lua,audio,lighting','editor')
foreach($features in $sets){
  $argsList=@('check','--locked','--no-default-features','--all-targets')
  if($features){$argsList+=@('--features',$features)}
  Write-Host "Checking features: $features"
  & cargo @argsList
  if($LASTEXITCODE -ne 0){throw "Feature check failed: $features"}
}
& cargo test --locked --all-targets
if($LASTEXITCODE -ne 0){throw 'Full tests failed'}
& cargo test --locked --no-default-features --lib
if($LASTEXITCODE -ne 0){throw 'Minimal tests failed'}
& cargo check --locked --no-default-features --features gpu-lab --examples
if($LASTEXITCODE -ne 0){throw 'GPU lab compilation failed'}
