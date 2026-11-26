Forge Engine v0.7.14 — targeted hotfix

Extract into your existing ForgeEngine-v0.7.14 folder and allow replacement of two files. Keep target/ and Cargo.lock.
Then run BUILD_WINDOWS.bat.

Changes: Reactor Sentinel animation now explicitly sets looping=true (matches Animation::default). Static demo validation now checks required animation keys and looping type even for objects without sprites. No Rust source or dependencies changed.
Static validation: 4/4 levels passed; original missing field reproduced by negative test. Rust / Windows release not rerun here.
