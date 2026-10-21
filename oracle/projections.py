"""Oracle for the projections suite.

Reads suites/projections/projections.json and, for every entry, projects the
graticule intersections in its lon/lat window with PROJ (through pyproj) and
inverts the result again. Writes suites/projections/reference.json: per
entry the points, PROJ's forward results, PROJ's inverse of those, and the
lon/lat definition on the same earth model that engines are given as source.

PROJ runs Transformer.from_pipeline on the bare projection string, so the
projection is executed exactly as written: no CRS interpretation, no datum
step, no axis swap.

Usage (from the repository root): python3 oracle/projections.py
Set PROJ_GIT_SHA to record the PROJ commit when pyproj is built against a
PROJ checkout rather than a release.
"""

import datetime
import json
import math
import os
import sys

import pyproj
from pyproj import Transformer
from pyproj.enums import TransformDirection

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, "suites", "projections", "projections.json")
OUT = os.path.join(ROOT, "suites", "projections", "reference.json")

# Bump when a field in reference.json changes meaning; documented in
# suites/projections/README.md.
SCHEMA_VERSION = 1

DEFAULTS = {"lon": [-180, 180], "lat": [-90, 90], "step": 30}

# parameters that define the earth model, carried over to the lon/lat source
EARTH = ("+ellps=", "+R=", "+a=", "+b=", "+rf=", "+f=", "+es=", "+e=", "+datum=")


def frange(lo, hi, step):
    n = int(round((hi - lo) / step))
    return [lo + i * step for i in range(n + 1)]


def geographic(proj):
    """Lon/lat on the projection's own earth model, so no datum step is involved."""
    earth = [t for t in proj.split() if t.startswith(EARTH)]
    return " ".join(["+proj=longlat"] + (earth or ["+ellps=GRS80"]) + ["+no_defs"])


def num(v):
    return v if math.isfinite(v) and abs(v) < 1e30 else None


def pair(x, y):
    x, y = num(x), num(y)
    return [x, y] if x is not None and y is not None else None


def reference(entry):
    e = {**DEFAULTS, **entry}
    points = [[lon, lat] for lat in frange(e["lat"][0], e["lat"][1], e["step"])
              for lon in frange(e["lon"][0], e["lon"][1], e["step"])]
    out = {k: v for k, v in entry.items() if k not in ("lon", "lat", "step")}
    out.update(window={"lon": e["lon"], "lat": e["lat"], "step": e["step"]},
               geographic=geographic(e["proj"]), points=points)
    try:
        t = Transformer.from_pipeline(
            "+proj=pipeline +step +proj=unitconvert +xy_in=deg +xy_out=rad +step " + e["proj"])
    except Exception as err:  # noqa: BLE001
        out["error"] = str(err)
        return out
    fwd, inv = [], []
    for lon, lat in points:
        try:
            xy = pair(*t.transform(lon, lat, direction=TransformDirection.FORWARD))
        except Exception:  # noqa: BLE001
            xy = None
        fwd.append(xy)
        if xy is None:
            inv.append(None)
            continue
        try:
            inv.append(pair(*t.transform(*xy, direction=TransformDirection.INVERSE)))
        except Exception:  # noqa: BLE001
            inv.append(None)
    out.update(fwd=fwd, inv=inv)
    return out


def main():
    with open(SRC) as f:
        entries = json.load(f)
    env = {
        "schema_version": SCHEMA_VERSION,
        "generated": datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds"),
        "proj_version": pyproj.proj_version_str,
        "proj_git_sha": os.environ.get("PROJ_GIT_SHA"),
        "pyproj_version": pyproj.__version__,
        "defaults": DEFAULTS,
    }
    out = {"environment": env, "projections": [reference(e) for e in entries]}
    with open(OUT, "w") as f:
        json.dump(out, f, separators=(",", ":"), ensure_ascii=True)
        f.write("\n")
    n = sum(len(p["points"]) for p in out["projections"])
    bad = [p["id"] for p in out["projections"] if "error" in p]
    if len(bad) == len(entries):
        sys.exit(f"PROJ refused every projection (first error: {out['projections'][0]['error']}); is PROJ_DATA set?")
    print(f"wrote {os.path.relpath(OUT, ROOT)}: {len(entries)} projections, {n} points, "
          f"PROJ {env['proj_version']}" + (f"; PROJ refused: {bad}" if bad else ""))


if __name__ == "__main__":
    main()
