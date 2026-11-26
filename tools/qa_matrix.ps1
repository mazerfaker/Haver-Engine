$ErrorActionPreference='Stop'
Set-Location (Join-Path $PSScriptRoot '..')
$sets=@('','physics-rapier','lua','audio','lighting','physics-rapier,lua,audio,lighting','editor')
foreach($features in $sets){
  $argsList=@('check','--no-default-features','--all-targets')
  if($features){$argsList+=@('--features',$features)}
  Write-Host "Checking features: $features"
  & cargo @argsList
  if($LASTEXITCODE -ne 0){throw "Feature check failed: $features"}
}
& cargo test --all-targets
if($LASTEXITCODE -ne 0){throw 'Full tests failed'}
& cargo test --no-default-features --lib
if($LASTEXITCODE -ne 0){throw 'Minimal tests failed'}
& cargo check --no-default-features --features gpu-lab --example gpu_lighting_lab
if($LASTEXITCODE -ne 0){throw 'GPU lab compilation failed'}
