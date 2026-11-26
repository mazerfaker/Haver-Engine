# Haver Engine v0.7.45 — Animation Studio Refinement
Base: CI-verified 98c8eba9153d02219d716369dd7470c86c7f3154. Candidate pending CI.
- Scale selected property/visual scalar lanes and optional clip events by 0.1–10x around the left boundary of the selection.
- Preserve values and interpolation handles. Reject collisions and out-of-range times atomically; events remain inside clip duration.
- Three new Rust regression tests for shared origin, key collisions and event bounds rollback.
- Existing transform, curve, clipboard, preview and selection tools remain. No automatic rigging or animation, no clip-duration or sprite-frame retiming.
- No new dependencies. Manual GUI validation and real GPU/audio testing remain outstanding.
