#!/usr/bin/env bash
# Shallow checkouts of PROJ (with history back to SINCE) and proj4rs main
# under tracker/.cache/, for tracker/watch.py.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
since="${SINCE:-400 days ago}"  # keep this older than watch.py --since
mkdir -p "$here/.cache"
fetch() {  # url dir
  if [ -d "$2/.git" ]; then
    git -C "$2" fetch -q --shallow-since="$since" origin HEAD && git -C "$2" checkout -q --detach FETCH_HEAD
  else
    git clone -q --shallow-since="$since" "$1" "$2"
  fi
}
fetch https://github.com/OSGeo/PROJ "$here/.cache/PROJ"
fetch https://github.com/3liz/proj4rs "$here/.cache/proj4rs"
