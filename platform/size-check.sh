#!/usr/bin/env bash
# E7: stripped release size of the harness built with one candidate at a time
# (native x86_64; the empty-candidate build is the baseline). Run via run-in-image.sh.
set -e
cd /work/harness
for f in none c-proj c-proj4rs c-crsdefs c-geodesy c-miniproj c-projcore c-wbprojection; do
  feat=$([ $f = none ] && echo "" || echo "--features $f")
  td=/work/.target-size/$f
  start=$(date +%s)
  cargo build --release --offline --no-default-features $feat --target-dir $td >/dev/null 2>&1 || { echo "$f build failed"; continue; }
  secs=$(( $(date +%s) - start ))
  cp $td/release/capability-harness /tmp/h && strip /tmp/h
  echo "$f $(stat -c %s /tmp/h) bytes, clean build ${secs}s"
done
