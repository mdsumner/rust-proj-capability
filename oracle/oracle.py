"""Oracle for the Rust PROJ capability map.

Reads fixtures/fixtures-src.json, runs every fixture through pyproj (PROJ) with
network off and no proj-data grids, and writes fixtures/crs.json: the fixtures
with literal CRS strings, reference forward/inverse coordinates, the operation
PROJ used, the candidate operations it considered, and PROJ exports (bare and
bound-to-WGS84 PROJ strings) that port candidates are fed.

Run inside the gdal-r-python-extras image with --network none:
  ./run-in-image.sh python3 -I oracle/oracle.py
"""

import datetime
import json
import math
import os
import sqlite3
import subprocess
import sys
import warnings

import pyproj
from pyproj import CRS, Transformer
from pyproj.transformer import TransformerGroup

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
SRC = os.path.join(ROOT, "fixtures", "fixtures-src.json")
OUT = os.path.join(ROOT, "fixtures", "crs.json")

# Bump when a field in crs.json changes meaning; documented in fixtures/README.md.
SCHEMA_VERSION = 1

EXPORT_FORMATS = {
    "PROJ": ["-o", "PROJ"],
    "WKT1_GDAL": ["-o", "WKT1_GDAL", "--single-line"],
    "WKT1_ESRI": ["-o", "WKT1_ESRI", "--single-line"],
    "WKT2_2015": ["-o", "WKT2_2015", "--single-line"],
    "WKT2_2019": ["-o", "WKT2_2019", "--single-line"],
    "PROJJSON": ["-o", "PROJJSON", "--single-line"],
}


def pyproj_export(defn, fmt):
    """Unbound export through pyproj (proj_as_proj_string / proj_as_wkt / proj_as_projjson).

    projinfo is not used here because its PROJ-string output silently binds
    the CRS to WGS84 when it can (it emitted +towgs84 for EPSG:2056)."""
    try:
        crs = CRS.from_user_input(defn)
        if fmt == "PROJ":
            with warnings.catch_warnings():
                warnings.simplefilter("ignore")
                return crs.to_proj4(), None
        if fmt == "PROJJSON":
            return crs.to_json(), None
        return crs.to_wkt(fmt), None
    except Exception as e:  # noqa: BLE001
        return None, str(e)


def projinfo(defn, fmt, bound=False):
    """Export a CRS; bound=True goes through proj_crs_create_bound_crs_to_WGS84
    (projinfo --boundcrs-to-wgs84). Returns (text, error)."""
    if not bound:
        return pyproj_export(defn, fmt)
    if defn.lstrip().startswith("+") and "+type=crs" not in defn:
        defn = defn + " +type=crs"  # otherwise projinfo reads it as an operation
    args = ["projinfo", "-q"] + EXPORT_FORMATS[fmt] + ["--boundcrs-to-wgs84", defn]
    r = subprocess.run(args, capture_output=True, text=True)
    text = r.stdout.strip()
    if r.returncode != 0 or not text:
        return None, (r.stderr.strip() or "projinfo failed")
    lines = [l for l in text.splitlines() if l.strip()]
    if fmt == "PROJ":
        lines = [l for l in lines if l.startswith("+")]
        return (lines[-1] if lines else None), None
    return "\n".join(lines), None


def resolve_ref(defn):
    """'@export:AUTH:CODE:FMT[-BOUND]' -> literal export text."""
    if not defn.startswith("@export:"):
        return defn, None
    _, auth, code, fmt = defn.split(":")
    bound = fmt.endswith("-BOUND")
    fmt = fmt.replace("-BOUND", "")
    return projinfo(f"{auth}:{code}", fmt, bound)


def kind_of(crs):
    if crs.is_compound:
        h = crs.sub_crs_list[0]
        return "compound-" + kind_of(h)
    if crs.is_geocentric:
        return "geocentric"
    if crs.is_geographic:
        return "geographic"
    if crs.is_projected:
        return "projected"
    if crs.is_vertical:
        return "vertical"
    return "other"


def horizontal(crs):
    return crs.sub_crs_list[0] if crs.is_compound else crs


def unit_info(crs):
    """Unit of the first axis of the horizontal member, in metres or radians per unit."""
    h = horizontal(crs)
    ax = h.axis_info[0] if h.axis_info else None
    if ax is None:
        return None, None
    return ax.unit_name, ax.unit_conversion_factor


def crs_exports(defn, crs):
    """PROJ strings a port is fed: bare and bound-to-WGS84, of the horizontal member."""
    h = horizontal(crs)
    hdef = h.to_wkt() if crs.is_compound else defn
    out = {}
    out["proj"], out["proj_error"] = projinfo(hdef, "PROJ")
    out["proj_bound"], out["proj_bound_error"] = projinfo(hdef, "PROJ", bound=True)
    if crs.is_compound:
        out["proj_full"], _ = projinfo(defn, "PROJ")
        out["proj_full_bound"], _ = projinfo(defn, "PROJ", bound=True)
    out["wkt1_gdal_bound"], _ = projinfo(hdef, "WKT1_GDAL", bound=True)
    return out


def op_summary(op):
    if op is None:
        return None
    try:
        p = op.to_proj4()
    except Exception:  # noqa: BLE001
        p = None
    return {
        "name": op.description,
        "accuracy": op.accuracy,
        "ballpark": "ballpark" in op.name.lower(),
        "proj": p,
    }


def last_op(tr):
    """Operation PROJ actually used (proj_trans_get_last_used_operation).

    When the transformer wraps a single operation PROJ has no last-used
    record; the transformer itself is then the operation."""
    try:
        return op_summary(tr.get_last_used_operation())
    except Exception:  # noqa: BLE001
        s = op_summary(tr)
        s["single_operation"] = True
        return s


def finite(v):
    return all(isinstance(x, float) and math.isfinite(x) for x in v)


def run_fixture(fx):
    o = {}
    src_def, err = resolve_ref(fx["src"])
    if err:
        o["error"] = f"src export: {err}"
        return src_def, None, o
    dst_def, err = resolve_ref(fx["dst"])
    if err:
        o["error"] = f"dst export: {err}"
        return src_def, None, o
    try:
        src = CRS.from_user_input(src_def)
    except Exception as e:  # noqa: BLE001
        o["resolved"] = False
        o["error"] = f"src: {e}"
        return src_def, dst_def, o
    try:
        dst = CRS.from_user_input(dst_def)
    except Exception as e:  # noqa: BLE001
        o["resolved"] = False
        o["error"] = f"dst: {e}"
        return src_def, dst_def, o
    o["resolved"] = True
    o["src_name"], o["dst_name"] = src.name, dst.name
    o["src_kind"], o["dst_kind"] = kind_of(src), kind_of(dst)
    o["src_unit"], o["src_unit_factor"] = unit_info(src)
    o["dst_unit"], o["dst_unit_factor"] = unit_info(dst)
    o["src_export"] = crs_exports(src_def, src)
    o["dst_export"] = crs_exports(dst_def, dst)

    # source point in src units (visualisation order)
    if "point" in fx:
        pt = [float(v) for v in fx["point"]]
    else:
        lon, lat = fx["point_lonlat"]
        t0 = Transformer.from_crs(horizontal(src).geodetic_crs, horizontal(src), always_xy=True)
        x, y = t0.transform(lon, lat, errcheck=True)
        pt = [x, y]
    o["point"] = pt

    # candidate operations PROJ considered
    try:
        g = TransformerGroup(src, dst, always_xy=True)
        o["candidates"] = [
            {"name": t.description, "accuracy": t.accuracy, "available": True} for t in g.transformers[:6]
        ] + [{"name": u.name, "accuracy": u.accuracy, "available": False} for u in g.unavailable_operations[:6]]
        o["best_available"] = g.best_available
    except Exception as e:  # noqa: BLE001
        o["candidates_error"] = str(e)

    try:
        with warnings.catch_warnings(record=True) as w:
            warnings.simplefilter("always")
            tr = Transformer.from_crs(src, dst, always_xy=True)
        if w:
            o["warnings"] = [str(x.message) for x in w]
    except Exception as e:  # noqa: BLE001
        o["executed"] = False
        o["error"] = f"create: {e}"
        return src_def, dst_def, o
    o["transformer"] = tr.description
    try:
        fwd = list(tr.transform(*pt, errcheck=True))
        o["executed"] = finite(fwd)
        o["fwd"] = [v if math.isfinite(v) else repr(v) for v in fwd]
    except Exception as e:  # noqa: BLE001
        o["executed"] = False
        o["error"] = f"forward: {e}"
        return src_def, dst_def, o
    o["op"] = last_op(tr)
    if o["executed"]:
        try:
            inv = list(tr.transform(*fwd, direction="INVERSE", errcheck=True))
            o["inv"] = inv
            o["inv_op"] = last_op(tr)
        except Exception as e:  # noqa: BLE001
            o["inv_error"] = str(e)
    return src_def, dst_def, o


def environment():
    datadir = pyproj.datadir.get_data_dir()
    con = sqlite3.connect(os.path.join(datadir, "proj.db"))
    meta = dict(con.execute("select key, value from metadata").fetchall())
    con.close()
    grids = [f for f in sorted(os.listdir(datadir)) if f.endswith((".tif", ".gtx", ".gsb", ".json")) and "schema" not in f]
    r = subprocess.run(["projinfo"], capture_output=True, text=True)
    return {
        "schema_version": SCHEMA_VERSION,
        "generated": datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds"),
        "proj_version": pyproj.proj_version_str,
        "projinfo_banner": (r.stdout + r.stderr).splitlines()[0],
        "pyproj_version": pyproj.__version__,
        "proj_data": datadir,
        "proj_data_files": sorted(os.listdir(datadir)),
        "grid_files_present": grids,
        "network_enabled": pyproj.network.is_network_enabled(),
        "epsg_version": meta.get("EPSG.VERSION"),
        "epsg_date": meta.get("EPSG.DATE"),
        "esri_version": meta.get("ESRI.VERSION"),
        "grid_configuration": "no proj-data grids (proj.db, proj.ini and init files only), network off",
        "axis_order": "always_xy=True: oracle normalised for visualisation (east, north)",
    }


def main():
    if pyproj.network.is_network_enabled():
        sys.exit("network is enabled; refusing to run the oracle")
    with open(SRC) as f:
        spec = json.load(f)
    env = environment()
    if env["grid_files_present"]:
        sys.exit(f"grid files present in {env['proj_data']}: {env['grid_files_present'][:5]}")
    out = []
    for fx in spec["fixtures"]:
        src_def, dst_def, o = run_fixture(fx)
        rec = dict(fx)
        rec["src_ref"], rec["dst_ref"] = fx["src"], fx["dst"]
        rec["src"], rec["dst"] = src_def, dst_def
        rec["point"] = o.get("point", fx.get("point"))
        rec.pop("point_lonlat", None)
        rec["oracle"] = o
        out.append(rec)
        print(f"{fx['id']:32s} {'ok ' if o.get('executed') else 'ERR'} {o.get('op', {}) and o['op']['name'] if o.get('op') else o.get('error', '')}", file=sys.stderr)
    with open(OUT, "w") as f:
        json.dump({"environment": env, "fixtures": out}, f, indent=1, ensure_ascii=True)
        f.write("\n")


if __name__ == "__main__":
    main()
