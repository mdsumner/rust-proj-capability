#!/usr/bin/env bash
# Build PROJ from a git ref (default master) into oracle/.proj/install and
# pyproj against it into oracle/.proj/venv, for the projections oracle:
#   oracle/build-proj.sh [REF]
#   export LD_LIBRARY_PATH=oracle/.proj/install/lib PROJ_DATA=oracle/.proj/install/share/proj
#   PROJ_GIT_SHA=$(cat oracle/.proj/REV) oracle/.proj/venv/bin/python oracle/projections.py
# Needs cmake, a C++ compiler, the sqlite3 binary and libsqlite3-dev. TIFF and curl are
# switched off: the suite uses no grids.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
ref="${1:-master}"
top="$here/.proj"
src="$top/PROJ"
mkdir -p "$top"
if [ -d "$src/.git" ]; then
  git -C "$src" fetch -q --depth 1 origin "$ref" && git -C "$src" checkout -q --detach FETCH_HEAD
else
  git clone -q --depth 1 --branch "$ref" https://github.com/OSGeo/PROJ "$src"
fi
cmake -S "$src" -B "$top/build" -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX="$top/install" \
  -DBUILD_TESTING=OFF -DENABLE_TIFF=OFF -DENABLE_CURL=OFF -DBUILD_PROJSYNC=OFF -DBUILD_SHARED_LIBS=ON > "$top/cmake.log"
cmake --build "$top/build" -j "$(nproc)" --target install > "$top/build.log"
python3 -m venv "$top/venv"
PROJ_DIR="$top/install" "$top/venv/bin/pip" install -q --no-binary pyproj pyproj
git -C "$src" rev-parse HEAD > "$top/REV"
LD_LIBRARY_PATH="$top/install/lib" PROJ_DATA="$top/install/share/proj" "$top/venv/bin/python" -c "import pyproj; print('pyproj', pyproj.__version__, 'PROJ', pyproj.proj_version_str)"
echo "PROJ $(cat "$top/REV")"
