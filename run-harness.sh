#!/usr/bin/env bash
# Build and run the harness inside the extras image, network off.
# Two builds: the libproj columns (proj, proj-sys) are built without the pure
# Rust candidates because proj4rs's default `aeqd` feature links its own
# geodesic.c, whose geod_* symbols interpose on libproj's and make PROJ's aeqd
# return NaN (results/combined-build.jsonl keeps that evidence).
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p results
WD=/work/harness ./run-in-image.sh '
set -e
cargo build --release --offline --no-default-features --features c-proj --target-dir /work/.target-libproj
cargo build --release --offline --no-default-features --features c-proj4rs,c-crsdefs,c-geodesy,c-miniproj,c-projcore,c-wbprojection --target-dir /work/.target-pure
cargo build --release --offline --target-dir /work/.target
cd /work
{
  /work/.target-libproj/release/capability-harness -c proj -c proj-sys fixtures/crs.json
  /work/.target-pure/release/capability-harness -c proj4rs -c proj4rs+bound -c crs-definitions -c geodesy -c geodesy+bound -c miniproj -c proj-core -c proj-core+aoi -c wbprojection fixtures/crs.json
} > results/results.jsonl 2> results/stderr.txt
/work/.target/release/capability-harness -c proj -c proj-sys -c proj4rs -f method-aeqd fixtures/crs.json > results/combined-build.jsonl 2>/dev/null
# keep the tree ASCII: escape non-ASCII in libproj messages (degree signs)
python3 -c "import sys; d=open(sys.argv[1]).read(); open(sys.argv[1],\"w\").write(d.encode(\"ascii\",\"backslashreplace\").decode())" results/stderr.txt
wc -l results/results.jsonl
'
