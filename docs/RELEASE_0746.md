# Haver Engine v0.7.46 — Asset Database & Prefab Graph
Base: CI-verified 2823ba54809fffccc4eb64fb6b99401e41500f33. Candidate pending CI.
- Exact asset or prefab path reverse dependency search, direct/indirect shortest graph distances, deterministic ordering and cycle handling.
- Read-only indexed snapshot, explicit search, result invalidation on index replacement/project switch.
- Bounded graph traversal (262144 edges, 32768 nodes); display first 1024 owners.
- Two Rust regression tests for nested relationships, cycles, missing resources and duplicate edges; static integration gates.
- Existing asset transactions, cache, prefab transfer, animation and Save Game PRO remain. No automatic migration/deletion, dynamic Lua path inference or claimed performance gain.
- No new dependencies. GUI and hardware validation still required.
