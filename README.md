# PROJ ports and rewrites: a benchmark

PROJ is ported and rewritten again and again, in Rust, in JavaScript and as
WebAssembly. This repository is a shared benchmark for all of them: the same
inputs, PROJ's own answers, and judging code that belongs to no engine. It
is meant as a reference the ports can point to when they plan, compare and
negotiate, whether or not PROJ itself adopts it.

| | question | engines | reference |
|---|---|---|---|
| [Projections suite](results/projections/README.md) | does the engine compute the same projection as PROJ, over the whole area where it is used? | proj4rs (release, main, wasm), proj-core, wbprojection, geodesy, proj4js, math.gl | PROJ master, rebuilt nightly |
| [CRS suite](results/crs/per-fixture.md) | can it resolve a CRS and execute a CRS-to-CRS transformation the way PROJ does? | proj, proj-sys, proj4rs, crs-definitions, geodesy, miniproj, proj-core, wbprojection | PROJ 9.9.0 (the 2026-10-15 study) |
| [PROJ watch](results/tracker/README.md) | which PROJ projections changed recently, and are they ported or covered? | proj4rs | PROJ master |

A nightly workflow (`.github/workflows/benchmark.yml`) builds PROJ master,
recomputes the projections reference, follows proj4rs main, reruns every
engine, and commits the regenerated pages. How to add an engine:
[engines/README.md](engines/README.md). How to add a projection:
[suites/projections/README.md](suites/projections/README.md).

## Projections suite: first findings

As of 2026-10-21, 54 setups (the proj4rs gallery's set plus world
projections and two probes of recent PROJ changes):

- **proj4rs main matches PROJ to the millimetre on 36 of the 40 setups it
  supports**, natively and as wasm. Its latest release on crates.io (0.2.1)
  supports 29 of them: ortho, gnom, omerc, tpers, nsper, cass, eqdc, robin,
  eqearth, poly and sinu are on main only.
- **The watch found a PROJ fix that proj4rs has not followed.** PROJ
  [ca23885d](https://github.com/OSGeo/PROJ/commit/ca23885da621269333eb00c5010e2ff2ac3423f0)
  (2026-02) made spherical tmerc stable at lat = lat_0 and decides the
  inverse's hemisphere with `copysign(phi, D)`. proj4rs still has the older
  test, so with `+R` and a non-zero `lat_0` its inverse returns the wrong sign
  of latitude for every point north of lat_0
  ([tmerc-sphere](results/projections/pages/tmerc-sphere.md)).
- **math.gl and proj4js agree with each other and differ from PROJ on
  ellipsoidal ortho by 33.7 km** ([ortho](results/projections/pages/ortho.md)).
  math.gl 5.0.0-alpha.18 matches PROJ on 29 setups.
- **PROJ's own round trip is checked too**
  ([proj-check.md](results/projections/proj-check.md)), so a problem in the
  reference is reported as PROJ's, not as a port's.

## CRS suite: is there a Rust crate for me?

A coordinate library has to do two separate things:

- **resolve:** turn what you typed into a complete parameter set;
- **execute:** run those parameters.

PROJ's ports and rewrites fail the two independently. The CRS suite
measures both for the Rust landscape. The same 93 fixtures are run through 11
candidate columns and judged against PROJ 9.9.0 itself, with network off and
no grids installed. The brief is in [docs/brief.md](docs/brief.md).

### TL;DR

**Short answer.** Only `proj` (the libproj binding) does everything PROJ
does, and it brings a C++ build and no wasm. In pure Rust, use `proj-core`
for EPSG-coded data where datum shifts matter. Use `proj4rs` when you already
have good PROJ strings. Everything else is niche or needs checking against
PROJ.

| crate | what it is | CRS input it accepts | projection families | datums and operations | exact / 93 |
|---|---|---|---|---|---|
| [`proj`](https://crates.io/crates/proj) | binding to libproj (C++) | everything: EPSG, ESRI, IAU and IGNF codes, PROJ strings, every WKT, PROJJSON | all of PROJ (189) | all of PROJ: Helmert, grids, operation selection | 86 |
| [`proj-core`](https://crates.io/crates/proj-core) | pure Rust, with an embedded EPSG registry | EPSG codes; strict PROJ strings; WKT and PROJJSON only when they carry an EPSG ID. Custom CRSs parse but can't be transformed | 23 methods: TM, Mercator, LCC, AEA, LAEA, oblique and polar stereographic, Hotine, Cassini, Krovak, polyconic, aeqd, Equal Earth, eqc | registry operations ranked by area and accuracy, Helmert, NTv2 and vertical grids; no epochs | 33 |
| [`proj4rs`](https://crates.io/crates/proj4rs) | port of proj4 | PROJ strings; WKT1 via `proj4wkt` (adds a spurious zero `towgs84`); EPSG only with the `crs-definitions` feature | 26 names in 18 families: tmerc/utm, merc, lcc, aea, laea, stere, sterea, somerc, krovak, geos, moll, cea, eqc, aeqd | `+towgs84` hubbed through WGS84, NTv2 | 20 (46 with bound strings) |
| [`crs-definitions`](https://crates.io/crates/crs-definitions) | static lookup table, not a transformer | EPSG codes 2000-32766 (`u16`), returning a PROJ string or WKT | n/a (pair with proj4rs) | carries PostGIS's `towgs84` | 19 of 38 applicable |
| [`geodesy`](https://crates.io/crates/geodesy) | operator-pipeline toolkit | its own pipeline syntax. `parse_proj` drops datum, units and ellipsoid | about 10: tmerc/utm, merc, webmerc, laea, lcc, omerc, somerc | operators you compose yourself: Helmert (with epoch), gridshift, deformation | 9 (27 wrapped) |
| [`miniproj`](https://crates.io/crates/miniproj) | EPSG projections generated at build time | EPSG codes only | 8 EPSG methods: TM, LCC, AEA, LAEA, oblique and polar stereographic, pseudo-Mercator | none: projection within one datum only | 8 |
| [`wbprojection`](https://crates.io/crates/wbprojection) | Whitebox's projection library | EPSG (hand-built list), PROJ strings, WKT1; ignores `+units` and `TOWGS84` in places | 94 kinds, including world maps (robin, wintri, eqearth, natearth, sinu, ortho). Many are spherical approximations | Helmert with the wrong sign convention for position-vector `towgs84`; NTv2 and NADCON loaders | 32 (33 off) |

"Exact" means within 1 mm of PROJ 9.9.0 on the fixture corpus. Most misses
are honest refusals, not wrong answers; [results/crs/per-fixture.md](results/crs/per-fixture.md) shows which.

**What no pure-Rust crate does (yet):**
- resolve ESRI, IAU or deprecated codes, or any WKT/PROJJSON without an EPSG ID;
- pick a datum operation for a custom CRS;
- time-dependent (epoch) transforms from a CRS pair;
- tpers, ob_tran, igh, qsc, healpix, rhealpix or isea.

Even with `proj`, the safe API is 2D: z and t are dropped. Use `proj-sys`
for 3D and epochs.

**What using each one looks like:**

| crate | core call | integrations | used by (crates.io reverse deps) |
|---|---|---|---|
| `proj` | `Proj::new_known_crs("EPSG:4326", "EPSG:3857", None)?.convert((x, y))` | `geo-types` (default), optional network grids; handles are `!Send`; no wasm32 | 27, including `geo`, `geopolars`, `geoarrow2`, `geotiff` |
| `proj-core` | `Transform::new("EPSG:4283", "EPSG:7844")?.convert((x, y))`, plus `convert_3d`; reports the chosen operation | `geo-types` and `rayon` (default), GeoTIFF grids; `forbid(unsafe)`, wasm32, `Send + Sync` | 4 |
| `proj4rs` | `transform(&Proj::from_proj_string(a)?, &Proj::from_proj_string(b)?, &mut pt)`, in radians | `geo-types` feature, a wasm/JS entry point, `proj4rs-clib`; wasm32 needs `default-features = false` | 19, including `cogrs`, `oxigdal-proj` |
| `crs-definitions` | `crs_definitions::from_code(2056)` | `no_std`, no dependencies | 9, including `proj4rs`, `copc-rs` |
| `geodesy` | `ctx.op("utm zone=32")?` then `ctx.apply(op, Fwd, &mut coords)` | `kp` command-line tool; wasm32 with the `js` feature | 7, including `galileo-types`, `geoarrow-wasm` |
| `miniproj` | `get_projection(3857)?.deg_to_projected(lon, lat)` | no runtime dependencies; wasm32 | 1 (`bevy_geo_tiles`) |
| `wbprojection` | `Crs::from_epsg(n)?.transform_to(x, y, &dst)` | `rayon`, `serde`; wasm32 | 5, all in the Whitebox stack |

**Which one:**
- **Any CRS, correct datums, grids, 3D or epochs, and you can link C++:** `proj` (drop to `proj-sys` for 3D and epochs).
- **Pure Rust or wasm, EPSG-coded data, datum shifts must be right:** `proj-core`. Pass an area of interest, and check the inverse of southern-hemisphere LCC/AEA.
- **Pure Rust or wasm, and PROJ strings with `+towgs84` already come from somewhere else (a server, PostGIS, `crs-definitions`):** `proj4rs`. It is exact on every method it has.
- **You design the pipeline yourself:** `geodesy`.
- **Same-datum TM/UTM from an EPSG code, as small as possible:** `miniproj`, after checking your codes.
- **`wbprojection`:** inside Whitebox. Elsewhere, verify every result against PROJ.

**Compared with R and Python.** In R, `sf` and `terra` sit on GDAL and
libproj. In Python, `pyproj` sits on libproj, and `geopandas` and `rasterio`
build on it. So in capability they all equal the `proj` row above. The
difference is in what you get without asking:

| | R (`sf`, `terra`) / Python (`pyproj`, `geopandas`, `rasterio`) | Rust today |
|---|---|---|
| getting PROJ | ships in CRAN binaries and wheels, `proj.db` included | `proj` links a system libproj or builds one with cmake; pure-Rust crates bring their own partial registry, or none |
| CRS input | one constructor takes anything: `st_crs()`, `pyproj.CRS()` | only `proj` is that open (table above) |
| CRS travels with the data | yes: `st_transform(x, "EPSG:3577")`, `project(r, crs)`, `gdf.to_crs()` | no: `geo-types` geometries carry no CRS, so you track it yourself and transform coordinates (`geo`'s `Transform` over `proj` is the nearest equivalent) |
| axis order | handled: `always_xy=True`, sf's lon/lat default | `proj` normalises it; the pure-Rust crates are lon/lat only and reject authority order |
| seeing which operation ran | `sf_proj_pipelines()`, `TransformerGroup`, `get_last_used_operation()` | `proj-sys` and `proj-core` only; not the safe `proj` API |
| 3D and epochs | z and t pass through `Transformer.transform()` | `proj-sys` only (the safe `proj` API is 2D) |
| grids | on-demand CDN: `sf_proj_network(TRUE)`, `pyproj.network.set_network_enabled()` | `proj` with the `network` feature; pure-Rust crates load local files only |
| raster reprojection | `terra::project()`, `rasterio.warp.reproject()` (GDAL warper) | outside these crates: the `gdal` binding, or `rwarp` (which this study informs) |
| what Rust offers that they don't | | pure-Rust crates build for wasm32, give one static binary with no `proj.db`, and are `Send + Sync` |

The short version: if you're used to `sf` or `pyproj`, `proj` gives you the
same engine but less of the ergonomics. The pure-Rust crates give you
portability in exchange for part of the engine.

### Headline findings

- **The datum step is the expensive gap, and binding to WGS84 only partly
  closes it.**
  - The bare PROJ string for EPSG:2056 is 164 m off and the bound string is
    exact
    ([form-2056-proj-bare](suites/crs/catalog/form-2056-proj-bare.md)).
  - PROJ declines to bind EPSG:27700, ED50 and GDA94, leaving ports 109 m,
    172 m and 1.5 m off.
- **Operation selection is no longer PROJ-only.** `proj-core` carries an EPSG
  registry and matches PROJ where every hub port is off
  ([pair-4283-7844](suites/crs/catalog/pair-4283-7844.md)). It cannot transform
  custom CRSs.
- **`proj4rs` executes what it ports to 0.000 mm.** The exception is `eqc`,
  where PROJ 9.9 has moved to the ellipsoid
  ([method-eqc](suites/crs/catalog/method-eqc.md)).
- **Several rewrites are silently wrong** by hundreds of metres to thousands
  of kilometres. The cases, with fixtures, are in
  [report.md](report.md#findings).
- **`proj4rs` (default features) and libproj cannot share a native binary.**
  The `geod_*` symbols clash and PROJ's aeqd returns NaN
  ([engines/rust-crs/README.md](engines/rust-crs/README.md#symbol-clash-why-there-are-two-builds)).

## Read

| | |
|---|---|
| [report.md](report.md) | findings, what each candidate is for, wrapper vs structural, changes from the TypeScript precedent |
| [capability-matrix.md](capability-matrix.md) | 15 layers by 8 candidates, with evidence |
| [results/crs/per-fixture.md](results/crs/per-fixture.md) | verdict grid: every fixture by every candidate |
| [suites/crs/FIXTURES.md](suites/crs/FIXTURES.md) | fixture catalog: a page per fixture with inputs, PROJ's answer and each candidate's result |
| [suites/crs/README.md](suites/crs/README.md) | oracle configuration and data schema |
| [engines/rust-crs/README.md](engines/rust-crs/README.md) | how candidates are fed and judged |
| [platform/README.md](platform/README.md) | wasm32, binary size, thread safety |
| [results/projections/README.md](results/projections/README.md) | projections suite: verdict grid, a page and a map per projection |
| [suites/projections/README.md](suites/projections/README.md) | how the projections suite is run and judged |
| [engines/README.md](engines/README.md) | every engine, and how to add one |
| [results/tracker/README.md](results/tracker/README.md) | PROJ change watch |

### Candidates

CRS suite: PROJ 9.9.0 (oracle) and the following crates:

| crate | version | added by |
|---|---|---|
| `proj` + `proj-sys` | 0.31.0 + 0.27.0 | brief |
| `proj4rs` (+ `proj4wkt`) | 0.2.1 | brief |
| `crs-definitions` | 0.6.0 | brief |
| `geodesy` | 0.15.0 | brief |
| `miniproj` | 0.10.4 | added from crates.io |
| `proj-core` (+ `proj-wkt`) | 0.11.0 | added from crates.io |
| `wbprojection` | 0.3.3 | added from crates.io |

There are also wrapper columns: `proj4rs+bound`, `geodesy+bound` and
`proj-core+aoi`.

## Reproduce

Projections suite, no container needed (python3 with pyproj, cargo, node,
and wasm-pack for the proj4rs-wasm column):

```
engines/js/build-proj4rs-wasm.sh       # proj4rs main as a Node wasm module
ORACLE=1 ./run-projections.sh          # PROJ reference with the installed pyproj, every engine, pages
oracle/build-proj.sh master            # optional: PROJ and pyproj built from a PROJ ref, as the nightly does
tracker/checkout.sh && python3 tracker/watch.py --proj tracker/.cache/PROJ --proj4rs tracker/.cache/proj4rs
```

CRS suite: everything runs in `ghcr.io/hypertidy/gdal-r-python-extras`:
`./make-proj-nogrids.sh` and a one-time `cargo fetch`, then
`./regenerate.sh`. Details are in [report.md](report.md#reproducing) and
[PUBLISHING.md](PUBLISHING.md).
