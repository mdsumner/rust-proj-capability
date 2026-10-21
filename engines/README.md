# Engines

| directory | suite | engines |
|---|---|---|
| [rust-crs](rust-crs/README.md) | [CRS](../suites/crs/README.md) | proj, proj-sys, proj4rs, crs-definitions, geodesy, miniproj, proj-core, wbprojection, as of the original study |
| `rust-projections` | [projections](../suites/projections/README.md) | proj4rs (crates.io release), proj4rs-main (git main, pinned in Cargo.lock), proj-core, wbprojection, geodesy |
| `js` | [projections](../suites/projections/README.md) | proj4js (npm `proj4`), math.gl (`@math.gl/projection`, the deck.gl v10 path), proj4rs-wasm (proj4rs main built for Node by `build-proj4rs-wasm.sh`) |

An engine adapter only computes. It is handed the same inputs as every other
engine and writes what the library returned; judging happens once, in
`report/`, with code that belongs to no engine.

## Adding an engine

- Rust: a module in `rust-projections/src/engines.rs` that prepares a
  transform from `Projection.proj` and `Projection.geographic`, and an entry in
  `REGISTRY`.
- JavaScript: a block in `js/run.mjs` that pushes `{name, version, prepare}`.
- Anything else: a program that reads `suites/projections/reference.json` and
  writes the JSON lines described in the suite README; pass its output file to
  `report/projections.py` with the others.

## Following upstream

`proj4rs-main` and `proj4rs-wasm` follow the proj4rs main branch: the nightly
workflow runs `cargo update -p proj4rs-main` and rebuilds the wasm module, and
each result records the commit it ran. Released crates and npm packages are
pinned; bump them deliberately so a change in the grid is a change in the
engine, not in the harness.
