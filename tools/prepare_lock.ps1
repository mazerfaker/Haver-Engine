# Preserve resolved dependency versions; only migrate the root package version.
$ErrorActionPreference='Stop'
Set-Location (Join-Path $PSScriptRoot '..')
if (!(Test-Path 'Cargo.lock')) { & cargo generate-lockfile; if ($LASTEXITCODE -ne 0) { throw 'Cargo lockfile generation failed' }; exit 0 }
$path=Join-Path (Get-Location).Path 'Cargo.lock'
$text=[IO.File]::ReadAllText($path)
$pattern='(?m)(^\[\[package\]\]\r?\nname = "forge_engine"\r?\nversion = ")0\.7\.(?:14|15|16|17|18|19|20|21|22|23|24|25|26|27|28|29|30|31|32|33)(")'
if ([regex]::Matches($text,$pattern).Count -eq 1) {
  [IO.File]::WriteAllText($path,[regex]::Replace($text,$pattern,'${1}0.7.34${2}'),[Text.UTF8Encoding]::new($false))
  Write-Host 'Migrated forge_engine package version only; dependency pins unchanged.'
}
