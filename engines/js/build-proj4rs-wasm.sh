#!/usr/bin/env bash
# Build the proj4rs wasm module for Node from 3liz/proj4rs at REF (default
# main) into engines/js/proj4rs-pkg/, recording the commit in REV.
# Extra wasm-pack flags go in WASM_PACK_ARGS (e.g. --no-opt where binaryen
# cannot be downloaded).
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
ref="${1:-main}"
src="$here/.cache/proj4rs"
if [ -d "$src/.git" ]; then
  git -C "$src" fetch -q --depth 1 origin "$ref"
  git -C "$src" checkout -q --detach FETCH_HEAD
else
  git clone -q --depth 1 --branch "$ref" https://github.com/3liz/proj4rs "$src"
fi
cd "$src/proj4rs"
# shellcheck disable=SC2086
wasm-pack build --release --target nodejs --out-dir "$here/proj4rs-pkg" ${WASM_PACK_ARGS:-} \
  --no-default-features --features=proj4js-compat --features=with-wasm-entrypoint
git -C "$src" log -1 --format='%h %ad' --date=short > "$here/proj4rs-pkg/REV"
cat "$here/proj4rs-pkg/REV"
