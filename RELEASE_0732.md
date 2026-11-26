# Forge Engine v0.7.32 — Bezier XY & Dependency Graph

Source candidate based on v0.7.31. No new Cargo dependencies. No Auto Rig or automatic animator.

## Animation Studio
- ScalarKey adds optional `bezier_x: [f32; 2]`; absent handles retain the original linear time mapping (1/3, 2/3). The cubic time coordinate is inverted by 24 bisection iterations for custom handles, then the value curve is sampled. Handles are clamped to normalized [0,1]; no overshoot. Existing Key and legacy transform tracks unchanged.
- Drag both X and Y of handles in the existing property graph. Equal-value segments cannot edit relative Y via mouse (no amplitude). Reset presets restore legacy X.
- Multi-channel time-range tool selects any of six property/visual axes in the current clip, shifts matching keys transactionally and rejects bounds or collisions. Existing group tools and copy/paste remain. No cross-clip operation or multi-lane drag canvas.

## Asset Database
- Graph edges for static object resources, linked prefab sources and external nested prefab references. Select source in Transitive dependency graph; cycle-safe visited traversal, missing documents reported, resource presence remains in asset list.
- Cache direct document dependencies in `.forge-assets.json`, 4096 documents, stamp by size/mtime seconds/nanoseconds. Prune unvisited entries; active scene always read from memory. 32 MiB database limit, existing import hash cache preserved. Cache may miss external edits that preserve both size and mtime. Static scan cannot resolve dynamic Lua paths. No asset deletion.

## GPU Instancing
- Experimental lab and runtime preserved byte-for-byte. No integration: requested prerequisite (successful lab verification) has not been met. Cargo/rustc unavailable here; actual GPU validation not performed.
- On Windows run QUICK_CHECK.bat, QUICK_TEST.bat, CHECK_EXPERIMENTS.bat, then GPU_INSTANCING_LAB.bat. Verify moving particles and absence of validation errors before integration.

## Fixture
RUN_BEZIER_XY_SCENE.bat / examples/bezier_xy/project.forge.json; nested root.prefab.json -> leaf.prefab.json. Open fixture in editor, refresh database, select root prefab in transitive graph. Repeat refresh to see cache hits.

## QA
Static checks only; no Rust compilation, native GUI execution, GPU or performance measurements. Three new Rust tests authored but not executed. Lock migration updates only root package version. Patch excludes Cargo.lock and target.
