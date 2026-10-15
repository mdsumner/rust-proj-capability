#!/usr/bin/env bash
# Rebuild everything derived, in order, with the network off:
#   oracle (fixtures/crs.json) -> harness (results/) -> pages (fixtures/FIXTURES.md,
#   fixtures/catalog/, results/per-fixture.md, results/README.md).
# Prerequisites, once: ./make-proj-nogrids.sh and
#   NET=bridge WD=/work/harness ./run-in-image.sh cargo fetch
# Hand-written pages (README.md, capability-matrix.md, report.md, */README.md) are
# not touched; check their numbers against results/README.md after a rerun.
set -euo pipefail
cd "$(dirname "$0")"
./run-in-image.sh python3 -I oracle/oracle.py
./run-harness.sh
python3 report/summarise.py results/results.jsonl > results/summary.txt
python3 report/make_docs.py
