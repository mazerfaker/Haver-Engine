# Forge Engine 0.7.12 — Combat & Audio Tools
Based on the actual 0.7.11 source archive. No Auto Rig or automatic animator.

## Changes
- Optional serialized combat component (team, health, damage, directional rectangular hitbox, cooldown, positional knockback with endpoint static collision check). Configure in Inspector > Combat. For ordinary non-demo projects, J attacks with the current horizontal input (defaults right). Enable combat on player and opponent, assign different teams. Existing demo combat remains unchanged. Dead combat objects stop running simulation scripts. This is not a full impulse or swept collision system.
- Editor Audio menu previews WAV/OGG from disk with a 32 MiB per-file limit and explicit Stop. Uses existing Bevy audio dependency. Playback is untested. Existing mixer remains; spatial audio and effects not added.
- Editor texture modification timestamp polling limited to twice per second rather than every UI pass.
- Updated version and portable build identifiers. Existing feature matrix and release profiles retained.

## Not completed
GPU integration, true docking, new icon suite, hierarchical UI/inventory, spatial audio/effects, further binary-size reductions. Existing implementations and experimental shader files preserved.

## Verification
Python demo validation and asset pack tests passed. Rust compiler not available; no Cargo check, Rust tests, GUI test, WGSL validation, performance measurements or EXE. Test CHECK_WINDOWS.bat locally. New combat test included but not executed.
