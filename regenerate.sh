#!/usr/bin/env bash
# Rebuild everything derived, in order, with the network off:
#   oracle (suites/crs/crs.json) -> harness (results/crs/) -> pages (suites/crs/FIXTURES.md,
#   suites/crs/catalog/, results/crs/per-fixture.md, results/crs/README.md).
# Prerequisites, once: ./make-proj-nogrids.sh and
#   NET=bridge WD=/work/engines/rust-crs ./run-in-image.sh cargo fetch
# Hand-written pages (README.md, capability-matrix.md, report.md, */README.md) are
# not touched; check their numbers against results/crs/README.md after a rerun.
set -euo pipefail
cd "$(dirname "$0")"
./run-in-image.sh python3 -I oracle/crs.py
./run-harness.sh
python3 report/summarise.py results/crs/results.jsonl > results/crs/summary.txt
python3 report/make_docs.py
