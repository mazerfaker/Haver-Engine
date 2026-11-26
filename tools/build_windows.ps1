param([switch]$CheckOnly, [ValidateSet("release","performance","lightweight","development")][string]$Profile="release")
$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')
$root = (Get-Location).Path
$log = Join-Path $root 'forge-check.log'
$dist = Join-Path $root 'dist'
$stage = Join-Path $root '.forge-package-stage'
$zipName = if ($Profile -eq 'release') { 'ForgeEngine-v0.7.39-Windows-portable.zip' } else { "ForgeEngine-v0.7.39-Windows-$Profile-portable.zip" }
$zip = Join-Path $root $zipName
function Write-Log([string]$line) {
    Write-Host $line
    Add-Content -LiteralPath $log -Value $line -Encoding UTF8
}
function Run-Native([string]$program, [string[]]$arguments) {
    Write-Log "`n> $program $($arguments -join ' ')"
    $old = $ErrorActionPreference
    try {
        $ErrorActionPreference = 'Continue'
        & $program @arguments 2>&1 | ForEach-Object { Write-Log "$($_)" }
        $code = $LASTEXITCODE
    } finally { $ErrorActionPreference = $old }
    if ($code -ne 0) { throw "$program failed (exit $code)" }
}
try {
    Set-Content -LiteralPath $log -Encoding UTF8 -Value "Forge Engine v0.7.39 | $(Get-Date -Format o) | profile=$Profile | CheckOnly=$CheckOnly"
    if (!$CheckOnly -and (Test-Path "$dist/BUILD_OK.txt")) { Remove-Item "$dist/BUILD_OK.txt" }
    if (!$CheckOnly -and (Test-Path $zip)) { Remove-Item $zip }
    if ($env:OS -ne 'Windows_NT') { throw 'Windows required for this build script.' }
    foreach ($cmd in @('rustc','cargo')) { if (!(Get-Command $cmd -ErrorAction SilentlyContinue)) { throw "Missing $cmd. Install Rust MSVC and Visual Studio C++ Build Tools." } }
    Run-Native 'rustc' @('-vV')
    Run-Native 'cargo' @('--version')
    $target = 'x86_64-pc-windows-msvc'
    $env:CARGO_TARGET_DIR = Join-Path $root 'target'
    Run-Native 'powershell' @('-NoProfile','-ExecutionPolicy','Bypass','-File',(Join-Path $PSScriptRoot 'prepare_lock.ps1'))
    Run-Native 'cargo' @('check','--locked','--all-targets','--target',$target)
    Run-Native 'cargo' @('check','--locked','--no-default-features','--bin','forge_player','--target',$target)
    Run-Native 'cargo' @('check','--locked','--no-default-features','--features','gpu-lab','--examples','--target',$target)
    Run-Native 'cargo' @('test','--locked','--all-targets','--target',$target)
    Run-Native 'cargo' @('test','--locked','--no-default-features','--lib','--target',$target)
    Run-Native 'cargo' @('build','--locked','--bins','--target',$target)
    $debug = Join-Path $env:CARGO_TARGET_DIR "$target/debug"
    $demo = Join-Path $root 'examples/signal_lost'
    foreach ($name in @('project.forge.json','level2.forge.json','level3.forge.json','level4.forge.json')) {
        $scene = Join-Path $demo $name
        if (!(Test-Path -LiteralPath $scene -PathType Leaf)) { throw "Missing $scene" }
        Run-Native "$debug/forge_player.exe" @('--smoke',$scene)
        Run-Native "$debug/forge_editor.exe" @('--editor-qa',$scene)
    }
    Run-Native "$debug/forge_editor.exe" @('--editor-qa',(Join-Path $root 'examples/gpu_particles/project.forge.json'))
    Run-Native "$debug/forge_editor.exe" @('--editor-qa',(Join-Path $root 'examples/ui_studio/project.forge.json'))
    Run-Native "$debug/forge_editor.exe" @('--editor-qa',(Join-Path $root 'examples/studio_pro/project.forge.json'))
    Run-Native "$debug/forge_editor.exe" @('--editor-qa',(Join-Path $root 'examples/gpu_quads/project.forge.json'))
    Run-Native "$debug/forge_editor.exe" @('--editor-qa',(Join-Path $root 'examples/atlas_regions/project.forge.json'))
    Run-Native "$debug/forge_editor.exe" @('--editor-qa',(Join-Path $root 'examples/audio_routing/project.forge.json'))
    if ($CheckOnly) { Write-Log 'CHECK PASS'; exit 0 }
    Run-Native 'cargo' @('build','--locked','--profile',$Profile,'--bins','--target',$target)
    $release = Join-Path $env:CARGO_TARGET_DIR "$target/$Profile"
    if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
    New-Item -ItemType Directory -Path "$stage/SignalLost" -Force | Out-Null
    Copy-Item "$release/forge_editor.exe" "$stage/ForgeEngine.exe"
    Copy-Item "$release/forge_player.exe" "$stage/ForgePlayer.exe"
    Copy-Item "$release/forge_player.exe" "$stage/SignalLost/ForgePlayer.exe"
    foreach ($name in @('project.forge.json','level2.forge.json','level3.forge.json','level4.forge.json')) { Copy-Item "$demo/$name" "$stage/SignalLost/$name" }
    Copy-Item "$demo/assets" "$stage/SignalLost/assets" -Recurse
    Copy-Item 'RUN_GAME.bat' "$stage/SignalLost/PLAY.bat"
    Copy-Item 'tools/PORTABLE_EDITOR.bat' "$stage/START_EDITOR.bat"
    Copy-Item 'docs/WINDOWS_BUILD.md' "$stage/README.md"
    Copy-Item 'Cargo.lock' "$stage/Cargo.lock"
    # Test release player against the actual staged resources, not source paths.
    foreach ($name in @('project.forge.json','level2.forge.json','level3.forge.json','level4.forge.json')) { Run-Native "$stage/SignalLost/ForgePlayer.exe" @('--smoke',"$stage/SignalLost/$name") }
    Get-ChildItem "$stage/*.exe" | Select-Object Name,Length | ConvertTo-Json | Set-Content -Encoding UTF8 "$stage/build-sizes.json"
    $info = [ordered]@{version='0.7.39';profile=$Profile;target=$target;editor_bytes=(Get-Item "$stage/ForgeEngine.exe").Length;player_bytes=(Get-Item "$stage/ForgePlayer.exe").Length}
    $info | ConvertTo-Json | Set-Content -Encoding UTF8 "$stage/build-info.json"
    Set-Content "$stage/BUILD_OK.txt" 'Compilation, unit tests, headless demo and editor QA passed. GUI launch not tested.'
    Copy-Item $log "$stage/forge-check.log"
    Compress-Archive -Path "$stage/*" -DestinationPath $zip -Force
    if (Test-Path $dist) { Remove-Item $dist -Recurse -Force }
    Move-Item $stage $dist
    Write-Log "BUILD PASS: $zip"
    exit 0
} catch {
    Write-Log "BUILD FAILED: $_"
    if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
    if (!$CheckOnly -and (Test-Path $zip)) { Remove-Item $zip -Force }
    exit 1
}
