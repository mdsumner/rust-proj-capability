#!/usr/bin/env bash
# The projections suite, end to end, without the container:
#   PROJ reference (pyproj) -> Rust engines -> JS and wasm engines -> pages.
# Needs: python3 with pyproj, cargo, node; wasm-pack for the proj4rs-wasm
# column (skipped if engines/js/proj4rs-pkg/ is missing).
#   ./run-projections.sh            reuse suites/projections/reference.json
#   ORACLE=1 ./run-projections.sh   recompute it with the installed PROJ first
set -euo pipefail
cd "$(dirname "$0")"
out=results/projections
mkdir -p "$out"
if [ "${ORACLE:-0}" = 1 ] || [ ! -f suites/projections/reference.json ]; then
  python3 oracle/projections.py
fi
cargo build --release --quiet --manifest-path engines/rust-projections/Cargo.toml
engines/rust-projections/target/release/projections-engines suites/projections/reference.json > "$out/engines-rust.jsonl"
(cd engines/js && [ -d node_modules ] || npm ci --silent)
node engines/js/run.mjs suites/projections/reference.json > "$out/engines-js.jsonl"
python3 report/projections.py "$out/engines-rust.jsonl" "$out/engines-js.jsonl"
python3 report/front_page.py
