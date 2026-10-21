# Platform checks (layer E7)

Raw output for the E7 rows of [../capability-matrix.md](../capability-matrix.md#e7-platform).

| file | what it records | how it was made |
|---|---|---|
| `wasm-check.txt` | whether each crate compiles for `wasm32-unknown-unknown`, with default and minimal features | `wasm-check.sh`, in a throwaway container that adds the target there (network on, no effect on the host toolchain) |
| `size-check.txt` | stripped release size of the harness built with one candidate at a time, plus clean build time; libproj and proj.db sizes | `size-check.sh`, via `../run-in-image.sh` |
| `send-sync.txt` | compiler check of `Send + Sync` on each candidate's handle type; the `proj::Proj` probe fails as expected | `cargo test --no-run` (module `send_sync` in `../engines/rust-crs/src/main.rs`, feature `c-proj-send-sync-probe`) |
| `versions.txt` | version, publish date, licence and MSRV of each crate | crates.io API |

Summary:

- **wasm32:** every pure-Rust crate compiles.
  - proj4rs needs `default-features = false`: its default `aeqd` feature compiles C.
  - geodesy needs `features = ["js"]` for uuid randomness.
  - The `proj` crate does not compile for wasm32: it needs a C++ toolchain and libproj.
- **Send + Sync:** all pure-Rust handles are; `proj::Proj` is not (`*mut PJconsts`).
