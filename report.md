# Report: a capability map of PROJ ports and rewrites, from Rust

2026-10-15. Measured against PROJ 9.9.0 / pyproj 3.8.0 in
`ghcr.io/hypertidy/gdal-r-python-extras:latest`, with network off and no
proj-data grids. The corpus is 93 fixtures and 11 candidate columns, giving
1023 JSON lines.

Where to look:
- layer-by-layer scores and evidence: [capability-matrix.md](capability-matrix.md)
- every fixture, with a page each: [suites/crs/FIXTURES.md](suites/crs/FIXTURES.md)
- verdict grid, fixtures by candidates: [results/crs/per-fixture.md](results/crs/per-fixture.md)
- raw output: `results/crs/results.jsonl` ([results/crs/README.md](results/crs/README.md))

## How it was run

1. **Fixtures.** `suites/crs/fixtures-src.json` holds the brief's seed list and
   pairs plus its additions:
   - NAD27 to NAD83, ED50 to WGS84;
   - ITRF2014 to GDA2020 at 2025.0;
   - WGS 84 to WGS 84 + EGM2008 height;
   - EPSG:27700 as code, bare and bound PROJ strings, WKT1 GDAL (plain and
     bound), WKT1 ESRI, WKT2:2015, WKT2:2019 and PROJJSON;
   - EPSG:2264 in US feet.

   I added 32 method fixtures, 10 edge fixtures (pole, seam, antipode, far
   from the central meridian, Mercator at the pole), unknown, invalid and
   deprecated inputs, grads, Mars, geocentric, and LV95 to LV03.
2. **Oracle.** `oracle/crs.py` produces `suites/crs/crs.json`. For each
   fixture it records:
   - the reference forward and inverse coordinates;
   - the operation used (`get_last_used_operation`) and the candidates
     considered;
   - the bare (`proj_as_proj_string`) and bound-to-WGS84 PROJ exports.

   It refuses to run if the network is enabled or if any grid file is on
   `PROJ_DATA`.
3. **Harness.** `engines/rust-crs/` is one binary with a cargo feature per candidate,
   run by `run-harness.sh` with `--network none`. It judges each result by
   distance from the oracle in metres:
   - "exact" means under 1 mm, "close" under 1 m;
   - it also measures the inverse of the oracle's forward result;
   - for bare-vs-code fixtures, it measures distance from a reference fixture.
4. **Platform.** `platform/` holds:
   - wasm32 builds per crate, in a throwaway container;
   - stripped binary size per candidate;
   - a compile-time `Send + Sync` probe;
   - versions and publish dates.

Harness decisions that affect what the numbers mean:

- Most fixtures start from OGC:CRS84. Candidates without a lookup are handed
  `+proj=longlat +datum=WGS84` for that side, so the shared starting point is
  not what is being scored.
- libproj's `crs_to_crs` rejects a PROJ string without `+type=crs` ("Object is
  not a CRS"). pyproj appends it silently, so the proj adapters append it and
  say so in `notes`.
- `geodesy+bound` and `proj4rs+bound` are wrapper columns. The library gets
  PROJ's bound export, and for geodesy the harness also does the translation
  into a pipeline. Their scores are what a thin wrapper achieves, not what the
  crate does alone.
- The libproj columns and the pure-Rust columns come from separate builds.
  proj4rs's default `aeqd` feature links its own `geodesic.c`. Its exported
  `geod_*` symbols interpose on libproj's, and PROJ's aeqd then silently
  returns NaN. The evidence is `results/crs/combined-build.jsonl`.

## Findings

1. **The datum step is still the expensive gap, and binding to WGS84 only
   closes part of it.**
   - The precedent's number reproduces exactly. The bare PROJ string for
     EPSG:2056 is 163.88 m from the code's result. The bound string is
     0.000 m, for PROJ itself, for proj4rs and for geodesy+bound
     ([form-2056-proj-bare](suites/crs/catalog/form-2056-proj-bare.md), [form-2056-proj-bound](suites/crs/catalog/form-2056-proj-bound.md)).
   - However, `proj_crs_create_bound_crs_to_WGS84` adds `+towgs84` only when
     PROJ can pick a single Helmert operation. It declines for:
     - EPSG:27700: several OSGB36 candidates, and the best one is the OSTN15
       grid;
     - ED50;
     - GDA94.

     Fed those bound strings, a port is 109 m off for 27700 and 172 m off for
     ED50.
   - The old static table does better on exactly those cases, because it
     carries a towgs84: crs-definitions is 3 mm off for 27700 and 7.45 m for
     ED50.
   - So "feed ports bound strings" is right but not sufficient. A wrapper
     should fall back to an explicit operation choice when the bound export
     comes back without `+towgs84`. It can detect that by comparing the bound
     string with the bare one.
   - Note: `projinfo -o PROJ` silently binds, while `proj_as_proj_string`
     does not. A "bare" string taken from projinfo already has the 2056
     `+towgs84`.
2. **Operation selection is no longer PROJ-only in Rust.**
   - proj-core ships an EPSG registry (v12.013) and ranks operations by area
     and accuracy. It is exact on:
     - GDA94 to GDA2020, where every hub port is 1.505 m off;
     - OSGB36;
     - 28355 to 7855;
     - ED50 once given an area of interest.
   - It also reports the operation it chose.
   - Its limits:
     - selection happens once per `Transform`, not per point (ED50 is 7.8 m
       off without an area of interest);
     - it never synthesises an operation, so any custom CRS (bare PROJ string,
       ESRI WKT without an ID, sterea string) cannot be transformed;
     - a grid coverage miss is an error, with no fallback. For NAD27 it picked
       a Canadian grid operation for a CONUS point and failed. PROJ fell back
       to an 11 m Helmert hub.
3. **Ports execute what they port, to 0.000 mm.**
   - proj4rs is exact, forward and inverse, on every method it has: tmerc 18
     and 28 degrees off the meridian, polar and oblique stereographic, laea at
     the antipode, krovak, geos, somerc, aeqd.
   - The one method disagreement is eqc. PROJ 9.9 gives the ellipsoidal
     result (x=12179061.95 against 12160796.02 on a sphere of radius a).
     proj4rs, proj-core and wbprojection all compute the spherical formula,
     30 km apart.
4. **The rewrites are uneven, and several are silently wrong.** Observed with
   the fixture that shows each one:

   | candidate | defect | fixture | size |
   |---|---|---|---|
   | wbprojection | drops EPSG:28355's false northing | [seed-28355](suites/crs/catalog/seed-28355.md) | 10,000 km |
   | wbprojection | ignores merc `+lat_ts` | [method-merc](suites/crs/catalog/method-merc.md) | 4390 km |
   | wbprojection | ignores `+units` | [form-2264-proj-bound](suites/crs/catalog/form-2264-proj-bound.md) | 445 km |
   | wbprojection | applies position vector `+towgs84` with the coordinate frame formula | `form-27700-*` | 29 m |
   | miniproj | south polar stereographic A grossly wrong | [method-ups-32761](suites/crs/catalog/method-ups-32761.md) | 287,000 km |
   | miniproj | outputs metres for US-feet CRSs | [form-2264-ftus](suites/crs/catalog/form-2264-ftus.md) | 445 km |
   | proj-core and wbprojection | wrong longitude from the inverse of southern-hemisphere LCC/AEA (same wrong value in both) | [method-lcc](suites/crs/catalog/method-lcc.md), [method-aea](suites/crs/catalog/method-aea.md) | |
   | geodesy | somerc inverse off | [method-somerc](suites/crs/catalog/method-somerc.md) | 4.97 km |
   | geodesy | south polar laea wrong | [seed-esri-102020](suites/crs/catalog/seed-esri-102020.md) | 11,125 km |
   | geodesy | ignores omerc `lat_0`/`gamma` | [method-omerc-alpha](suites/crs/catalog/method-omerc-alpha.md) | |

   None of these raises an error.
5. **Authority lookup is the common gap, as predicted.** Only libproj
   resolves ESRI, IAU and deprecated codes.
   - crs-definitions is keyed by `u16`: 6184 EPSG codes, none above 32766.
   - proj-core is EPSG-only and has no geocentric CRSs.
   - wbprojection is a hand-built list.
   - miniproj covers only its 8 methods.
6. **The georust binding is PROJ with three gaps in its safe API.**
   - `convert()` is 2D, so z and t are dropped without a report: 74 m on
     geographic 3D to geocentric, 0.29 m on the epoch pair.
   - It cannot report which operation it used.
   - It has no bound-CRS or bare PROJ-string export.

   All three are one `proj-sys` call away (the proj-sys column is 88/88 on
   executable fixtures). But `Proj` does not expose its raw pointer, so the
   FFI path has to own its own objects. `Proj` is `!Send`/`!Sync` (checked by
   the compiler), and the crate does not build for wasm32.
7. **Oracle caveats found along the way.**
   - `+proj=wintri` as a CRS is exported with `lat_1=0`, while the same string
     as an operation uses the documented 50.467. [seed-wintri](suites/crs/catalog/seed-wintri.md) therefore tests
     PROJ's CRS path, not the method.
   - PROJ returns a finite 242,528,680 m for lat 90 in EPSG:3857 rather than
     an error.
   - "Unsupported method" and "invalid value" share errno 1027.

## What each candidate is for

**PROJ / `proj` + `proj-sys`.** This is the only candidate that does all
fifteen layers. What it costs: a C++ library (4.9 MB), a 10.5 MB database,
sqlite3, libtiff and libcurl, no wasm32 from the crate, and `!Send` handles.

Use the safe `proj` API for 2D transforms between CRSs it can name. Drop to
`proj-sys` for:
- 3D and epochs;
- the last-used operation;
- `proj_crs_create_bound_crs_to_WGS84` + `proj_as_proj_string`, which is the
  export that feeds ports.

On the oracle side of a `CrsTransform` trait, it is also the right tool to
produce the definitions every other column consumes.

**proj4rs.** A faithful, small (262 KB), thread-safe, wasm-capable executor
for the proj4 subset: 26 names, 0.000 mm on all of them, eqc aside. It knows
nothing about authorities, operations or vertical datums. Feed it strings and
it hubs through WGS84 with whatever `+towgs84` they carry. This is why it is
the right backend for rwarp-wasm's `CrsTransform`, provided the strings come
from somewhere that knows the datum.

Two cautions:
- Build it with `default-features = false` (drops aeqd) for wasm.
- Build it with `default-features = false` in any native binary that also
  links libproj, or PROJ's aeqd breaks.

**crs-definitions.** A lookup, not a library. Its strings come from PostGIS
and carry `+towgs84` where PostGIS had one. That makes it better than PROJ's
bound export for 27700 and ED50. What it lacks:
- newer codes (9354, 9518);
- codes whose method proj4rs lacks (8857 is eqearth);
- everything not EPSG;
- 205 NAD27 strings and 34 NZGD49 strings that need grids.

Use it as a fallback table behind a real resolver, not as the resolver.

**geodesy.** An operator toolkit. Its Poder/Engsager tmerc, time-dependent
Helmert with an explicit convention, deformation and gridshift operators are
correct where tested. Once a wrapper writes `cart | helmert | inv cart`, it is
exact on the 2056 datum step.

It is not CRS software:
- no authority, WKT or PROJJSON input;
- `parse_proj` silently drops datum, units and ellipsoid shape;
- about 10 projections;
- wrong results in somerc inverse and south polar laea.

It is for composing pipelines you have already designed.

**miniproj.** A tiny, dependency-free (at runtime), wasm-capable EPSG
projection table for the common methods (TM, LCC, AEA, LAEA,
stereographic). It is accurate for TM and UTM. It has no datum step and no
error type. It has silent unit and south-polar bugs, and an LCC 2SP offset on
EPSG:2154 (1.23 km). Use it only for same-datum projection of known EPSG
codes after checking your codes against PROJ.

**proj-core.** The surprise: a pure-Rust, `forbid(unsafe)`, wasm-capable
library that does EPSG lookup, registry operation selection with reported
operations, and Helmert, NTv2 and vertical grids. It matches PROJ on the
GDA2020, OSGB36 and ED50 (with AOI) cases where the ports fail. It is strict
rather than lenient: it rejects unknown parameters and authority axis order.

The gaps:
- it is closed-world, so custom CRSs cannot be transformed;
- no epochs;
- no geocentric CRSs;
- selection happens once per transform;
- the southern-cone inverse bug, and 5.6 km on UPS South.

It is the closest thing to a database-equipped Rust alternative to libproj,
for EPSG-coded data.

**wbprojection.** Broad on paper: 94 projection kinds, EPSG, PROJ strings,
WKT, geocentric, prototypes for epochs and vertical. In this corpus it was
off on 33 of 93 fixtures, with no error in most of them. It is for Whitebox's
own pipeline. Outside it, treat every result as needing verification against
PROJ.

## Wrapper or structural

A wrapper can supply the following. The column that demonstrates each is in
brackets.

- **The datum step for ports**, where PROJ can bind: PROJ's bound export
  makes proj4rs exact on 2056, 7415, 2154 and 4807 [proj4rs+bound]. Where PROJ
  declines to bind (27700, ED50, GDA94), the wrapper has to pick an
  operation's Helmert parameters itself, or use crs-definitions' towgs84 as a
  fallback.
- **CRS input for geodesy**: translating ellipsoid, `+towgs84` and `+units`
  into `cart | helmert | inv cart | unitconvert` takes geodesy from 9 to 27
  exact fixtures [geodesy+bound]. Its operators were never the problem.
- **Point-sensitive operation choice in proj-core**: passing the point as the
  area of interest turns ED50 from 7.8 m to exact [proj-core+aoi]. A wrapper
  could cache one `Transform` per area.
- **What the georust safe API lacks**: 3D, epoch, last-used operation and
  bound export are all reachable through proj-sys [proj-sys].
- **Authority lookup for ports**: any resolver that emits PROJ strings
  (libproj on the server, crs-definitions offline) supplies I1 to proj4rs.
- **Axis order**: every pure-Rust candidate works in visualisation order. The
  wrapper owns the authority-order conversion.

Structural limits (a wrapper cannot fix these without becoming the missing
piece):

- **Operation selection without a database**: proj4rs, geodesy, miniproj and
  wbprojection hub through WGS84 with whatever the definitions say. GDA94 to
  GDA2020 is 1.5 m off for all of them. Only libproj and proj-core carry the
  operations.
- **Grid fallback policy**: PROJ falls back to the next candidate or a
  ballpark when a grid is missing. proj4rs errors on a missing mandatory
  grid; proj-core errors on a coverage miss. Getting PROJ's behaviour needs
  the candidate list, which only a database has.
- **Methods that are absent**: tpers, ob_tran, igh, qsc, healpix, rhealpix
  and isea exist only in libproj among the candidates. A closure-backed
  `CrsTransform` can route those to libproj, but no port runs them.
- **Silent numerical errors** in miniproj, wbprojection, geodesy (somerc
  inverse, polar laea) and the proj-core inverse cone bug. A wrapper can only
  detect them by checking against an oracle.
- **Time-dependent transformations from CRS input**: only libproj does this
  end to end. geodesy has the operator but no route from a CRS pair to it.
- **Coexistence**: proj4rs (default features) and libproj cannot share a
  native binary without breaking PROJ's aeqd. This is not structural to
  either library, but it constrains a `CrsTransform` that offers both
  backends.

## For the `CrsTransform` trait (rwarp)

The trait's three backends map onto the findings:

- **Native libproj.** Use proj-sys for anything beyond 2D, and to produce
  the definitions the other backends consume.
- **proj4rs.** Fed bound strings, with a check that the bound export actually
  gained `+towgs84`. Otherwise fall back to an explicit operation or a
  crs-definitions string, and warn. Build it without `aeqd` when it shares a
  binary with libproj.
- **A caller-supplied closure.** This is where proj-core fits for EPSG-coded
  inputs (pass an area of interest), and where the methods no port has go.

A capability probe of this harness's shape could run once per CRS pair and
choose the backend. The decision rule: exact on an oracle point means use the
port; otherwise route to libproj.

## What changed from the TypeScript precedent

| precedent finding | Rust result |
|---|---|
| The port executes ported methods to 0.000 mm | Confirmed for proj4rs on every method it has. New: PROJ 9.9's eqc is ellipsoidal and every Rust port is spherical (30 km), so "ported" can lag PROJ. |
| The lookup is the common gap | Confirmed. New: crs-definitions is `u16`-keyed (no codes above 32766) and has codes whose method proj4rs cannot run (8857). |
| The datum step is the expensive gap: bare EPSG:2056 is 164 m off, bound is exact | Reproduced exactly (163.88 m and 0.000 m). New: binding fails silently for 27700, ED50 and GDA94 (109 m and 172 m), where an old static table does better. New: `projinfo -o PROJ` binds silently, so the bare string has to come from `proj_as_proj_string`. |
| Operation selection (GDA94 to GDA2020, 1.5 m) lives only in the database | True of every hub port (1.505 m). New: in Rust a second database exists. proj-core is exact on that case, with the limits above. |
| (none) | New for Rust: the binding and a port cannot share a native binary as shipped (the `geod_*` clash). The georust safe API hides 3D, epoch and last-used operation. Four of the five non-binding libraries have silent errors of hundreds of metres or more. |

The `@math.gl/projection` column is not included: the sidebyside repository
has no `crs/` page to take results from (checked 2026-10-15).

## Candidates considered (crates.io, 2026-10-15)

Searched for "projection crs", "coordinate transformation", "epsg", "proj4",
"wkt crs" and "projjson", and checked reverse dependencies:

| crate | added? | why |
|---|---|---|
| proj4rs | yes | 19 reverse dependencies |
| proj | yes | 27 reverse dependencies |
| crs-definitions | yes | 9 reverse dependencies |
| geodesy | yes | 7 reverse dependencies |
| miniproj | yes | 41k recent downloads, 1 reverse dependency, maintained |
| proj-core | yes | 4.4k recent downloads, 4 reverse dependencies, active |
| wbprojection | yes | 10k recent downloads, 5 reverse dependencies |
| oxiproj | no | 1.3k downloads, 2 reverse dependencies |
| geonative-proj | no | TM only |
| gis-tools | no | WGS84 and Mercator only |
| epsg | no | last release 2023 |
| proj4wkt | used as proj4rs's WKT front end | |

## Reproducing

```
NET=bridge WD=/work/harness ./run-in-image.sh cargo fetch   # once, network on
./run-in-image.sh python3 -I oracle/crs.py               # oracle, network off
./run-harness.sh                                            # harness, network off
python3 report/summarise.py results/crs/results.jsonl --md > results/crs/per-fixture.md
docker run --rm -v $PWD/platform:/p:ro ghcr.io/hypertidy/gdal-r-python-extras:latest bash /p/wasm-check.sh
./run-in-image.sh bash /work/platform/size-check.sh
```

The `cargo fetch` and wasm checks are the only steps run with the network on.
The grid-free `PROJ_DATA` (`.proj-nogrids/`) is built by `./make-proj-nogrids.sh`, which copies proj.db,
proj.ini and the init files out of the image (see `suites/crs/README.md`).
