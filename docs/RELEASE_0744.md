# Haver Engine v0.7.44 — Save Game PRO foundation
Base: CI-verified 80a9c97f7ebecc8c5aa10baa772eeec3b12c134f.
Candidate pending CI.
- Three independent checkpoint slots, selected in the Player checkpoint window.
- F5/F9 use the selected slot. Slot 1 keeps the original filename and v1 schema; no destructive migration.
- Bounds checks, roundtrip, overwrite isolation and corruption tests.
- Existing checkpoint validation, atomic save, campaign validation and physics/Lua rebuild retained.
- No GPU runtime integration, live audio dynamics, schema migration or manual GUI validation claimed.
- Cargo.lock retains dependency pins; only workspace package version changes. Patch based on v0.7.43.
