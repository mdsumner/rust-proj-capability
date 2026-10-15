#!/usr/bin/env bash
# Run a command inside the gdal-r-python-extras image with this directory at /work.
# NET=none (default) keeps the container offline; NET=bridge for dependency fetch only.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
img="${IMG:-ghcr.io/hypertidy/gdal-r-python-extras:latest}"
exec docker run --rm --network "${NET:-none}" \
  --user "$(id -u):$(id -g)" \
  -v "$here":/work -w "${WD:-/work}" \
  -e HOME=/work/.home -e CARGO_HOME=/work/.cargo-home -e RUSTUP_HOME=/opt/rustup \
  -e CARGO_TARGET_DIR=/work/.target \
  -e PROJ_DATA=/work/.proj-nogrids -e PROJ_NETWORK=OFF \
  "$img" bash -c "$*"
