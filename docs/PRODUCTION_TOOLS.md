# v0.6 Production Tools — implementation status
Source upgrade from v0.5, not a tested executable. Schema 2 retained; new fields have serde defaults.

- Persistent Rapier world: retains pipeline, bodies, colliders, contact caches and velocities between ticks; rebuilds on topology/material/joint changes. Rope constraints configured in Production menu. Lua/graph vertical velocity requests forwarded to Rapier. Existing AABB raycast and trigger logic retained; accurate Rapier queries and collision events not implemented.
- Navigation: bounded 4-connected BFS around static AABBs, target and speed inspector. For non-player Body=None NPCs. No navmesh, patrol FSM or cached path; large obstacles can exceed search limits.
- Tile chunks: sparse 16×16 index and editor count; legacy object renderer retained. No chunk mesh batching or new autotile rules.
- Particles: standalone reuses sprite entities, capped at 4096, optional texture. Still CPU particle calculation; not GPU compute. Editor preview uses legacy implementation.
- Input: standalone keyboard remapping (limited supported keys), gamepad left stick/deadzone and south button, existing HUD pointer buttons. Editor still uses existing WASD path. No multi-touch action abstraction.
- Build profile: resolution consumed by desktop player, package identifier stored only. Android/iOS require platform entrypoints, asset packaging, SDKs and signing; no APK/IPA produced.
- Lighting, audio mixer and UI designer: previous v0.5 implementations retained; normal maps field reserved, no shader integration, audio effects or new visual designer in this release.
- QA: source review and package checks only, Rust toolchain absent. Added unit tests for navigation and chunk index; not executed. Use CHECK_WINDOWS.bat or GitHub Actions.
