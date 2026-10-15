#!/usr/bin/env bash
# Build .proj-nogrids/: PROJ_DATA with proj.db, proj.ini and init files only,
# copied from the image's /usr/local/share/proj (which carries 819 MB of grids).
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p .proj-nogrids
./run-in-image.sh 'cd /usr/local/share/proj && cp proj.db proj.ini world other.extra nad27 nad83 nad.lst CH GL27 ITRF2000 ITRF2008 ITRF2014 ITRF2020 projjson.schema.json triangulation.schema.json deformation_model.schema.json /work/.proj-nogrids/ && ls /work/.proj-nogrids'
