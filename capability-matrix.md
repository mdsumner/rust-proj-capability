# Capability matrix: PROJ ports and rewrites from Rust

Measured on 2026-10-15 against PROJ 9.9.0 (pyproj 3.8.0), with network off
and no proj-data grids. Configuration: [fixtures/README.md](fixtures/README.md).
Verdict grid: [results/per-fixture.md](results/per-fixture.md). Raw JSON lines:
`results/results.jsonl`. Findings and interpretation: [report.md](report.md).

Each cell is Y (yes), P (partial, say what) or N (no). Evidence is one of:
- a fixture id, linked to its page in the [fixture catalog](fixtures/FIXTURES.md) (inputs, what PROJ did, every candidate's result);
- a code reference, given as `crate-version/path:line` inside the pinned
  crate source (the cargo registry);
- a file under `platform/`.

## Candidates

| candidate | version (published) | repo | role |
|---|---|---|---|
| PROJ | 9.9.0 (2026-09-15) | https://github.com/OSGeo/PROJ | oracle. Its column is libproj reached through `proj-sys` raw FFI. |
| `proj` + `proj-sys` | 0.31.0 + 0.27.0 (2025-08-29) | https://github.com/georust/proj | Rust bindings, built against the image's libproj 9.9.0 via pkg-config (not `bundled_proj`) |
| `proj4rs` (+ `proj4wkt`) | 0.2.1 (2026-10-09), proj4wkt 0.1.1 (2025-09-19) | https://github.com/3liz/proj4rs | port of the proj4 lineage |
| `crs-definitions` | 0.6.0 (2026-09-16) | https://tangled.org/rwell.org/crs-definitions | static EPSG table, executed here by proj4rs |
| `geodesy` | 0.15.0 (2026-03-03) | https://github.com/busstoptaktik/geodesy | operator pipelines (the rethink tier) |
| `miniproj` | 0.10.4 (2026-01-07) | https://github.com/GEOMAR-Kiel/Miniproj | added: EPSG projections generated at build time. 41k recent downloads. |
| `proj-core` (+ `proj-wkt`) | 0.11.0 (2026-08-11) | https://github.com/roteiro-gis/proj-rust | added: pure Rust with an embedded EPSG registry and operation selection |
| `wbprojection` | 0.3.3 (2026-08-04) | https://github.com/jblindsay/whitebox_next_gen | added: Whitebox projection library. 5 reverse dependencies in the Whitebox stack. |

Checked and not added: `oxiproj` (1.3k recent downloads, 2 reverse
dependencies), `geonative-proj` (TM only), `gis-tools`, `epsg` (unmaintained
since 2023). The crates.io queries are recorded in `report.md`.

The `@math.gl/projection` column from the brief is omitted. The sidebyside
repository has no `crs/` page yet (checked 2026-10-15), so there are no
results to carry over. See `report.md` for how the precedent's hypotheses
fared.

## Summary grid

| layer | PROJ | proj (safe API) | proj4rs | crs-definitions | geodesy | miniproj | proj-core | wbprojection |
|---|---|---|---|---|---|---|---|---|
| I1 authority codes | Y | Y | N | P | N | P | P | P |
| I2 PROJ strings | Y | Y | P | n/a | P | N | P | P |
| I3 WKT | Y | Y | P | n/a | N | N | P | P |
| I4 PROJJSON | Y | Y | N | N | N | N | P | N |
| I5 compound/bound/derived | Y | P | P | P | N | N | P | P |
| I6 axis order, units | Y | Y | P | P | P | N | P | P |
| I7 export | Y | P | N | n/a | N | N | P | P |
| E1 projection methods | Y | Y | P | n/a | P | P | P | P |
| E2 Helmert | Y | P | P | P | P | N | P | P |
| E3 grid shifts | Y | Y | P | P | P | N | P | P |
| E4 operation selection | Y | P | N | N | N | N | P | N |
| E5 3D and dynamic | Y | P | P | n/a | P | N | P | P |
| E6 numerical fidelity | ref | Y | Y | Y | P | P | P | N |
| E7 platform (wasm32 / threads) | P | N | Y | Y | Y | Y | Y | Y |
| E8 failure behaviour | P | P | P | n/a | P | N | P | N |

Verdict counts over the 93 fixtures (`results/summary.txt`). "exact" is under
1 mm; "close" is under 1 m. Columns: refused = refused where the oracle also
refused; n/a = the input does not apply to this candidate.

| candidate | exact | close | off | failed | unresolved | refused | n/a |
|---|---|---|---|---|---|---|---|
| proj-sys | 88 | 0 | 0 | 0 | 0 | 5 | 0 |
| proj (safe) | 86 | 1 | 1 | 0 | 0 | 5 | 0 |
| proj4rs | 20 | 1 | 6 | 0 | 61 | 5 | 0 |
| proj4rs+bound | 46 | 2 | 12 | 1 | 27 | 5 | 0 |
| crs-definitions | 19 | 3 | 4 | 1 | 10 | 1 | 55 |
| geodesy | 9 | 0 | 5 | 1 | 73 | 5 | 0 |
| geodesy+bound | 27 | 2 | 15 | 1 | 43 | 5 | 0 |
| miniproj | 8 | 0 | 11 | 0 | 69 | 5 | 0 |
| proj-core | 33 | 0 | 4 | 12 | 39 | 5 | 0 |
| proj-core+aoi | 34 | 0 | 3 | 12 | 39 | 5 | 0 |
| wbprojection | 32 | 2 | 33 | 1 | 20 | 4 | 0 |

wbprojection has one more fixture ([form-invalid-param](fixtures/catalog/form-invalid-param.md)) where it answered and
the oracle refused, which is why its row does not sum to 93. The "+bound" and
"+aoi" rows are wrapper columns: the same library fed what a wrapper would
supply.

## Evidence by layer

### I1 authority codes

- **PROJ: Y.**
  - proj.db: EPSG v13.102 (2026-08-27), ESRI (ArcGIS Pro 3.6), IAU_2015, IGNF, OGC.
  - Deprecated codes resolve: EPSG:3785 ([form-3785-deprecated](fixtures/catalog/form-3785-deprecated.md)).
  - IAU works ([pair-mars-iau](fixtures/catalog/pair-mars-iau.md)).
  - The unknown code is refused ([seed-bad-code](fixtures/catalog/seed-bad-code.md)).
- **proj: Y.** Same database through `Proj::new`.
  - An unknown code comes back as "Unknown error (code 4096)" ([seed-bad-code](fixtures/catalog/seed-bad-code.md)).
- **proj4rs: N.**
  - It takes PROJ strings only.
  - `Proj::from_epsg_code` exists only behind the optional `crs-definitions` feature (`proj4rs-0.2.1/src/proj.rs:496`).
- **crs-definitions: P.** A static table of 6184 EPSG codes (2000-32766).
  - Source: generated from PostGIS `spatial_ref_sys` (`crs-definitions-0.6.0/scripts/regenerate.py:4`).
  - Keyed by `u16`: EPSG:99999 cannot even be asked.
  - No ESRI, IAU or OGC codes.
  - Missing: EPSG:9354 and EPSG:9518 ([seed-9354](fixtures/catalog/seed-9354.md), [pair-4979-egm2008](fixtures/catalog/pair-4979-egm2008.md)).
  - The deprecated EPSG:3785 is present.
- **geodesy: N.**
  - No authority lookup; only user macros via `register_resource` (`geodesy-0.15.0/src/context/mod.rs:58`).
- **miniproj: P.**
  - The EPSG v11.004 dump is compiled in (`miniproj-epsg-registry-0.10.4/data/gen_reg.sql`).
  - It only covers geographic 2D CRSs and projected CRSs in its 8 methods.
  - Missing: 3031 (polar stereographic B), 7415 and 3785.
- **proj-core: P.**
  - Embedded EPSG v12.013 from PROJ 9.6.2's proj.db (`proj-core-0.11.0/data/epsg.provenance.json`).
  - EPSG only, plus an `OGC:CRS84` alias.
  - Missing: geocentric 4978 ([pair-4979-4978](fixtures/catalog/pair-4979-4978.md)) and deprecated 3785.
- **wbprojection: P.**
  - Hand-built registry, claimed to cover about 5600 EPSG codes plus ESRI 54008/54009/54030.
  - Missing in this run: 7415, 4979, 7912, 4807, 3785, and ESRI 54042, 54052 and 102020.
  - The 3785 alias works only through `from_epsg_with_catalog`.

### I2 PROJ strings

- **PROJ: Y.**
  - Full vocabulary.
  - Unknown parameters are silently dropped: `+foo` disappears from the export ([form-unknown-param](fixtures/catalog/form-unknown-param.md)).
- **proj: Y**, with a caveat:
  - `new_known_crs` rejects a PROJ string without `+type=crs`: "Object is not a CRS". pyproj appends it silently.
  - The harness appends it and records a note.
- **proj4rs: P.**
  - 26 `proj=` names.
  - Unknown parameters are ignored silently.
  - `+type=crs`, `+wktext` and `+nadgrids=@null` are accepted ([form-3857-proj-export](fixtures/catalog/form-3857-proj-export.md) is exact).
  - `+towgs84` takes 3 or 7 parameters, position vector (`proj4rs-0.2.1/src/datum_transform.rs:73`).
- **geodesy: P.**
  - `parse_proj` translates projection parameters only.
  - `+datum`, `+towgs84`, `+units`, `+a`/`+b` are ignored silently and the ellipsoid becomes GRS80. Results:

    | fixture | off by |
    |---|---|
    | [form-2056-proj-bound](fixtures/catalog/form-2056-proj-bound.md) | 164 m |
    | [form-3857-proj-export](fixtures/catalog/form-3857-proj-export.md) | 28.6 km |
    | [form-2264-proj-bound](fixtures/catalog/form-2264-proj-bound.md) | 445 km |
- **miniproj: N.**
- **proj-core: P.**
  - 12 `proj=` names.
  - Strict: it rejects unknown parameters, `+a`/`+b` and `+wktext` ([form-unknown-param](fixtures/catalog/form-unknown-param.md), [form-3857-proj-export](fixtures/catalog/form-3857-proj-export.md)).
  - A non-registry CRS parses but cannot be transformed: "no compatible registry operation found ... EPSG:0" ([form-27700-proj-bare](fixtures/catalog/form-27700-proj-bare.md), [method-sterea](fixtures/catalog/method-sterea.md)).
- **wbprojection: P.**
  - It parses many `proj=` names but ignores `+units` ([form-2264-proj-bound](fixtures/catalog/form-2264-proj-bound.md), 445 km).
  - It ignores `+lat_ts` on merc ([method-merc](fixtures/catalog/method-merc.md), 4390 km).
  - It accepts `+lat_ts=95` ([form-invalid-param](fixtures/catalog/form-invalid-param.md)).

### I3 WKT

- **PROJ: Y.**
  - WKT1 GDAL, WKT1 ESRI, WKT2:2015 and WKT2:2019 all resolve to the same operation as the code (`form-27700-*`, [form-2056-wkt1-esri](fixtures/catalog/form-2056-wkt1-esri.md)).
- **proj: Y.** Same.
- **proj4rs (proj4wkt): P.** WKT1-style only. Defects:
  - It appends `+towgs84=0,0,0,0,0,0,0` when the WKT has no TOWGS84 (`proj4wkt-0.1.1/src/projstr.rs:81`). Every 27700 WKT is therefore 106.6 m from PROJ (`form-27700-wkt*`).
  - It has no Hotine mapping ([form-2056-wkt1-esri](fixtures/catalog/form-2056-wkt1-esri.md)).
  - It fails on WKT2:2019 with a datum ensemble ([form-4326-wkt2-2019](fixtures/catalog/form-4326-wkt2-2019.md)).
  - It ignores PRIMEM and AXIS.
- **geodesy: N.**
- **miniproj: N.**
- **proj-core: P.**
  - WKT carrying an EPSG ID is answered from the registry: exact for 27700 WKT1 GDAL and WKT2:2019.
  - Failures without an ID or in other dialects:
    - WKT2:2015: "missing a base geographic CRS".
    - ESRI WKT: fails to transform as a custom CRS.
    - Authority axis order is rejected ([form-4326-wkt2-2019](fixtures/catalog/form-4326-wkt2-2019.md)).
  - TOWGS84 is not read.
- **wbprojection: P.**
  - An EPSG ID in the WKT wins over the WKT body.
  - The datum is matched by name, and TOWGS84 is ignored.
  - Results: ESRI 2056 is 164 m off ([form-2056-wkt1-esri](fixtures/catalog/form-2056-wkt1-esri.md)); 27700 is 29.2 m off (Helmert convention, E2).

### I4 PROJJSON

- **PROJ, proj: Y** ([form-27700-projjson](fixtures/catalog/form-27700-projjson.md)).
- **proj-core: P.**
  - It has a PROJJSON reader, but rejects PROJ's own PROJJSON for 27700: "unsupported axis order/directions `north, east`".
- **proj4rs, crs-definitions, geodesy, miniproj, wbprojection: N.**

### I5 compound, bound, derived, vertical

- **PROJ: Y.**
  - 7415 (compound), BoundCRS from `+towgs84`, and vertical 9518 all work.
- **proj: P.**
  - Resolves everything, but the safe `convert()` is 2D. z and t are dropped without a report ([pair-4979-4978](fixtures/catalog/pair-4979-4978.md): 74 m, from h=100 lost).
- **proj4rs: P.**
  - It gets the horizontal member.
  - `+geoidgrids` is ignored silently (`proj4rs-0.2.1/src/parameters.rs:146`).
  - No vertical CRS support.
- **crs-definitions: P.**
  - Its 7415 string is the horizontal member with `+towgs84`: 4.8 mm ([seed-7415](fixtures/catalog/seed-7415.md)).
  - Has no 9518.
- **geodesy, miniproj: N.**
- **proj-core: P.**
  - Compound CRSs are understood, but CRS84 to 7415 is refused rather than dropped: "cannot transform between an explicit vertical CRS and a horizontal-only CRS".
  - The egm2008 grid is required, and it fails without it ([pair-4979-egm2008](fixtures/catalog/pair-4979-egm2008.md)).
- **wbprojection: P.**
  - A separate `CompoundCrs` API exists (`wbprojection-0.3.3/src/compound_crs.rs`).
  - `from_epsg(7415)` fails.

### I6 axis order and units

- **PROJ, proj: Y.**
  - `new_known_crs` calls `proj_normalize_for_visualization` (`proj-0.31.0/src/proj.rs:233`).
  - Grads and the Paris meridian work ([form-grads](fixtures/catalog/form-grads.md)); US feet work ([form-2264-ftus](fixtures/catalog/form-2264-ftus.md)).
- **proj4rs: P.**
  - `+axis`, `+pm`, `+units=us-ft` and grads via the bound string are all correct ([form-grads](fixtures/catalog/form-grads.md) and form-2264-* through proj4rs+bound).
  - It works in radians and has no notion of authority axis order.
- **geodesy: P.**
  - `adapt`/`axisswap`/`unitconvert` operators exist, but `parse_proj` drops `+units`.
- **miniproj: N.**
  - US-feet CRSs come out in metres ([form-2264-ftus](fixtures/catalog/form-2264-ftus.md), 445 km).
  - EPSG:4807 is treated as Greenwich degrees ([form-grads](fixtures/catalog/form-grads.md), 171 km).
- **proj-core: P.**
  - Always lon/lat; US feet are correct ([form-2264-ftus](fixtures/catalog/form-2264-ftus.md)).
  - Input with authority axis order is rejected (WKT/PROJJSON).
  - A non-Greenwich prime meridian is unsupported ([form-grads](fixtures/catalog/form-grads.md)).
- **wbprojection: P.**
  - US feet from EPSG: 0.8 m (datum, not units).
  - US feet from a PROJ string are ignored (445 km).
  - EPSG:4807 is unknown.

### I7 export

- **PROJ: Y.** PROJ string, WKT and PROJJSON.
  - `proj_as_proj_string` gives the bare string.
  - `proj_crs_create_bound_crs_to_WGS84` adds `+towgs84` when PROJ can pick a single Helmert operation: yes for 2056, 7415, 2154, 4807; no for 27700, ED50, GDA94 (`fixtures/crs.json`, `dst_export`).
  - Note: `projinfo -o PROJ` silently binds, but `proj_as_proj_string` does not. The oracle uses `pyproj.CRS.to_proj4()` for bare strings.
- **proj: P.**
  - Has `def()`, `to_projjson()` and `as_wkt()`.
  - No `proj_as_proj_string` and no bound-CRS creation in the safe API (raw `proj-sys` has both; the proj-sys column calls them).
- **proj4rs, geodesy, miniproj: N.**
- **proj-core: P.** WKT1, WKT2 and PROJJSON writers; no PROJ-string writer.
- **wbprojection: P.** ESRI and OGC WKT1 only.

### E1 projection methods

- **PROJ: Y.**
  - 189 entries in `proj -l`.
  - All 32 method fixtures are exact, including tpers, geos, ob_tran, omerc two-point, krovak, nzmg, igh, wintri, eck4, natearth, hammer, qsc, healpix, rhealpix and isea.
- **proj4rs: P.**
  - 26 names in 18 families (`proj4rs-0.2.1/src/projections/mod.rs:221`).
  - Present: geos, krovak, moll, laea, stere, sterea, lcc, aea, Poder/Engsager tmerc, merc, somerc, cea, eqc, aeqd.
  - Absent: tpers, ob_tran, omerc, nzmg, igh, wintri, eck4, natearth, hammer, qsc, healpix, rhealpix, isea, eqearth, robin, cass, poly, eqdc, ortho, sinu.
- **geodesy: P.**
  - About 10 projection operators: tmerc/utm (Poder/Engsager), btmerc, merc, webmerc, laea, lcc, omerc (not two-point), somerc.
  - Of the awkward list it has none.
- **miniproj: P.** 8 methods.
  - TM, LCC 1SP/2SP, AEA, oblique stereographic, LAEA, polar stereographic A, pseudo-Mercator.
- **proj-core: P.** 23 methods.
  - TM (Poder/Engsager), polar stereographic A/B/C, LCC variants, AEA, LAEA, oblique stereographic, Hotine A/B (not two-point), Cassini, Mercator, eqc, Krovak (EPSG only), Equal Earth, polyconic, aeqd.
  - No `+proj=` access for several of them.
- **wbprojection: P.**
  - 94 projection kinds, including wintri, eck4, natearth, hammer, eqearth, robin, ortho, sinu, poly, geos.
  - Many are spherical approximations (E6).
  - Missing: tpers, ob_tran, nzmg, igh, qsc, healpix, rhealpix, isea.

### E2 datum via Helmert

- **PROJ: Y.** 3-, 7- and 15-parameter Helmert in both conventions, time-dependent ([pair-itrf2014-gda2020-epoch](fixtures/catalog/pair-itrf2014-gda2020-epoch.md)).
- **proj: P.**
  - Same engine, but there is no epoch in the safe API: 0.29 m on the epoch pair ([pair-itrf2014-gda2020-epoch](fixtures/catalog/pair-itrf2014-gda2020-epoch.md)).
- **proj4rs: P.**
  - `+towgs84` 3/7 position vector, hubbed through WGS84.
  - The datum step is skipped when either side has no datum (`proj4rs-0.2.1/src/transform.rs:110`).
  - No time-dependent Helmert.
  - Exact when the bound string carries the parameters: [seed-2056](fixtures/catalog/seed-2056.md), [form-2056-proj-bound](fixtures/catalog/form-2056-proj-bound.md).
- **crs-definitions: P.**
  - Carries whatever towgs84 PostGIS had. That beats PROJ's bound export where PROJ declines to bind:

    | fixture | crs-definitions | proj4rs+bound |
    |---|---|---|
    | [form-27700-epsg](fixtures/catalog/form-27700-epsg.md) | 3 mm | 109 m |
    | [pair-ed50-wgs84](fixtures/catalog/pair-ed50-wgs84.md) | 7.45 m | 172 m |
- **geodesy: P.**
  - The `helmert` operator covers 3/7/14/15 parameters with an explicit, required convention and `t_epoch` (`geodesy-0.15.0/src/inner_op/helmert.rs:300`).
  - Exact on 2056 once a wrapper builds `cart | helmert | inv cart` (geodesy+bound [seed-2056](fixtures/catalog/seed-2056.md)).
  - Nothing reaches it from CRS input.
- **miniproj: N.** No datum step.
  - [pair-ed50-wgs84](fixtures/catalog/pair-ed50-wgs84.md): 172 m. [pair-nad27-nad83](fixtures/catalog/pair-nad27-nad83.md): 35 m.
- **proj-core: P.**
  - 3/7-parameter position vector and affine transforms from the registry.
  - No time-dependent Helmert: [pair-itrf2014-gda2020-epoch](fixtures/catalog/pair-itrf2014-gda2020-epoch.md) fails.
  - `+towgs84` in strings is parsed but never used for a custom CRS.
- **wbprojection: P.**
  - It applies the coordinate frame formula to position vector parameters (`wbprojection-0.3.3/src/datum.rs:170`, `proj_string.rs:611`). That is consistent with `form-27700-*` being 29.2 m off.
  - [pair-ed50-wgs84](fixtures/catalog/pair-ed50-wgs84.md): 7.45 m.

### E3 grid shifts

- **PROJ: Y.** NTv2, GTG, network.
  - With grids missing it fell back every time and recorded which operation it used (`fixtures/README.md`).
- **proj: Y.** Same libproj. `ProjBuilder::set_search_paths` / `grid_cache_enable`; `enable_network` behind the `network` feature.
- **proj4rs: P.**
  - NTv2, and GeoTIFF behind the `tiff` feature.
  - Grids are provided through a catalog with no loader by default (`proj4rs-0.2.1/src/nadgrids/catlg.rs:124`).
  - A missing mandatory grid is an error, not a fallback: [pair-nad27-nad83](fixtures/catalog/pair-nad27-nad83.md) gives "NAD grid not available".
- **crs-definitions: P.**
  - 205 strings use `+datum=NAD27` (implicit grids) and 34 name `nzgd2kgrid0005.gsb`. These fail without grids.
- **geodesy: P.**
  - `gridshift` supports NTv2, GTX, GSA and Gravsoft from `Plain` search paths. No GeoTIFF, no network.
  - A missing grid is `NotFound` (`geodesy-0.15.0/src/context/plain.rs:104`).
- **miniproj: N.**
- **proj-core: P.**
  - NTv2, GTX, and GeoTIFF (feature). Only `ntv2_0.gsb` is embedded.
  - Missing grids make it skip candidates.
  - A coverage miss is an error with no Helmert fallback: [pair-nad27-nad83](fixtures/catalog/pair-nad27-nad83.md) picked "NAD27 to NAD83 (4)" (Canada) for a CONUS point and then failed.
- **wbprojection: P.**
  - NTv2 and NADCON loaders exist, but no grids are bundled.

### E4 operation selection

- **PROJ: Y.**
  - Candidates are ranked by area and accuracy and selected per point, and the selection is reported (`oracle.op`, `oracle.candidates`).
- **proj: P.**
  - The selection is PROJ's, but the safe API does not expose the last-used operation. `proj_info().description` reads "unavailable until proj_trans is called".
  - proj-sys reports it.
- **proj4rs, crs-definitions, wbprojection: N.**
  - They hub through WGS84 with whatever the definitions carry.
  - [pair-4283-7844](fixtures/catalog/pair-4283-7844.md) and [pair-4283-7855](fixtures/catalog/pair-4283-7855.md): 1.5 m each.
- **geodesy: N.** The caller composes the pipeline.
- **miniproj: N.**
- **proj-core: P.**
  - Registry operations are ranked by area of use and accuracy, and the selected operation and diagnostics are reported (`proj-core-0.11.0/src/transform.rs:382`).
  - Exact on GDA94 to GDA2020, OSGB36 and 28355 to 7855.
  - Selection happens once per `Transform`:
    - [pair-ed50-wgs84](fixtures/catalog/pair-ed50-wgs84.md) is 7.8 m off without an area of interest and exact with one (proj-core+aoi).
    - [seed-7855](fixtures/catalog/seed-7855.md) chose "WGS 84 to GDA2020 (3)" (1.5 m off).
  - No operations are synthesised, so a custom CRS cannot be transformed.

### E5 3D and dynamic

- **PROJ: Y.** Geocentric ([pair-4979-4978](fixtures/catalog/pair-4979-4978.md)), epochs ([pair-itrf2014-gda2020-epoch](fixtures/catalog/pair-itrf2014-gda2020-epoch.md)), and a vertical ballpark when the grid is missing ([pair-4979-egm2008](fixtures/catalog/pair-4979-egm2008.md)).
- **proj: P.**
  - The safe API is 2D: [pair-4979-4978](fixtures/catalog/pair-4979-4978.md) is 74 m off and the epoch pair 0.29 m.
  - `coordinate_metadata_create(epoch)` exists, but there is no per-point t.
- **proj4rs: P.**
  - Geocentric works ([pair-4979-4978](fixtures/catalog/pair-4979-4978.md) via the bound string), and height is carried through Helmert.
  - No geoid, no vertical datums, no epochs.
- **geodesy: P.** `cart`, time-dependent `helmert`, `deformation`, and `gridshift` for geoids, as operators.
- **miniproj: N.** Only an ellipsoid-to-geocentric helper.
- **proj-core: P.**
  - 3D with vertical grids.
  - No epochs, and no geocentric CRS in the registry.
- **wbprojection: P.**
  - Geocentric via `transform_to_3d`.
  - Vertical and epoch support are documented as prototypes.

### E6 numerical fidelity

Forward and inverse are measured against PROJ on the 93-fixture corpus.

- **proj, proj-sys:** exact on all fixtures except the safe-API 2D losses above.
- **proj4rs: Y.**
  - Every method it has is exact forward and inverse, including tmerc 18 degrees off the central meridian ([method-tmerc-far](fixtures/catalog/method-tmerc-far.md)) and the near-antipodal laea point (3 mm inverse).
  - The one disagreement is [method-eqc](fixtures/catalog/method-eqc.md) (30 km): proj4rs computes spherical eqc, while PROJ 9.9 gives the ellipsoidal result. `proj +proj=eqc +lat_ts=-42 +ellps=WGS84` returns x=12179061.95, against 12160796.02 with `+R=6378137`.
- **geodesy: P.**
  - tmerc, merc, lcc and somerc forward are exact.
  - somerc inverse is 4.97 km off ([method-somerc](fixtures/catalog/method-somerc.md)).
  - South polar laea is wrong ([seed-esri-102020](fixtures/catalog/seed-esri-102020.md), 11,125 km; [edge-laea-antipode](fixtures/catalog/edge-laea-antipode.md)).
  - omerc `lat_0`/`gamma` are ignored silently ([method-omerc-alpha](fixtures/catalog/method-omerc-alpha.md), 4750 km).
- **miniproj: P.**
  - Exact on TM, UTM and pseudo-Mercator.
  - [seed-2154](fixtures/catalog/seed-2154.md) (LCC 2SP): 1.23 km.
  - South polar stereographic A is grossly wrong ([method-ups-32761](fixtures/catalog/method-ups-32761.md), 287,000 km).
- **proj-core: P.**
  - Exact forward on its methods.
  - Inverse LCC and AEA on southern-hemisphere cones return the wrong longitude ([method-lcc](fixtures/catalog/method-lcc.md), [method-aea](fixtures/catalog/method-aea.md) inverse: 2880 km and 3421 km; `results.jsonl` `inv`).
  - UPS South is 5.62 km off ([method-ups-32761](fixtures/catalog/method-ups-32761.md)).
  - eqc is spherical (30 km).
- **wbprojection: N.**
  - EPSG:28355 has lost its 10,000,000 m false northing ([seed-28355](fixtures/catalog/seed-28355.md), [pair-28355-7855](fixtures/catalog/pair-28355-7855.md)).
  - Further cases:

    | fixture | off by | cause |
    |---|---|---|
    | [method-merc](fixtures/catalog/method-merc.md) | 4390 km | `+lat_ts` ignored |
    | [method-robin](fixtures/catalog/method-robin.md) | 680 km | |
    | [method-sinu](fixtures/catalog/method-sinu.md) | 30 km | |
    | [method-laea](fixtures/catalog/method-laea.md) | 621 m | spherical |
    | [method-aeqd](fixtures/catalog/method-aeqd.md) | 644 m | spherical |
    | [method-sterea](fixtures/catalog/method-sterea.md) | 65.9 m | single-stereographic kernel |
    | [method-somerc](fixtures/catalog/method-somerc.md) | 381 m | mapped to spherical Hotine |
    | [method-tmerc-far](fixtures/catalog/method-tmerc-far.md) | 10.5 m | Snyder series, 18 degrees from the central meridian |
    | [edge-32755-far](fixtures/catalog/edge-32755-far.md) | 235 m | Snyder series, 28 degrees from the central meridian |
  - Inverse LCC has the same southern-cone error as proj-core.
- **Oracle caveat:** [seed-wintri](fixtures/catalog/seed-wintri.md) measures a PROJ inconsistency, not a candidate. `+proj=wintri` as a CRS is exported with `+lat_1=0` (`projinfo "+proj=wintri +datum=WGS84 +type=crs" -o PROJ`). The same string as an operation uses the documented default of 50.467 (`proj +proj=wintri` gives x=11523356). wbprojection matches the operation path.

### E7 platform

Measured with `platform/wasm-check.sh`, `platform/size-check.sh` and the
compile-time `Send + Sync` probe in `harness/src/main.rs` (results in
`platform/*.txt`). Sizes are stripped x86_64 binaries, relative to a
554,816-byte harness baseline.

| candidate | wasm32-unknown-unknown | no_std | Send + Sync | build dependencies | size added |
|---|---|---|---|---|---|
| PROJ (libproj) | not as a crate (needs emscripten) | no | per context | cmake, C++, sqlite3, libtiff, libcurl | libproj 4.9 MB + proj.db 10.5 MB, plus grids |
| proj | FAIL (C++ toolchain) | no | no: `*mut PJconsts` (compiler, `platform/send-sync.txt`) | pkg-config libproj >= 9.6.2, else bundled 9.6.2 via cmake; bindings pregenerated from 9.6.0 headers | 27 KB (dynamic link) |
| proj4rs | OK with `default-features = false`; default FAIL (the `aeqd` feature compiles C) | no | yes | cc (default) | 262 KB |
| crs-definitions | OK | yes | n/a | none | 5.7 MB (proj4 and WKT tables, default features) |
| geodesy | OK with `default-features = false, features = ["js"]`; default FAIL (uuid randomness) | no | yes (`Minimal`) | none | 588 KB |
| miniproj | OK | no | yes | build.rs parses a 24.6 MB SQL dump (13 s clean build) | 772 KB |
| proj-core | OK | no | yes (`Transform`) | none (`forbid(unsafe_code)`) | 2.2 MB |
| wbprojection | OK | no | yes (`Crs`) | none | 2.3 MB |

Linking proj4rs (default features) and libproj into one binary breaks PROJ's
aeqd. proj4rs-geodesic exports `geod_*` symbols that interpose on libproj's,
and aeqd silently returns NaN (`results/combined-build.jsonl`;
`harness/README.md`).

### E8 failure behaviour

All candidates refused the antipodal tpers/ortho points and the unknown
method. Differences show up elsewhere:

- **PROJ: P.**
  - Explicit errors, but "unsupported" and "invalid" share errno 1027, "Invalid value for an argument" ([form-unknown-method](fixtures/catalog/form-unknown-method.md), [form-invalid-param](fixtures/catalog/form-invalid-param.md)).
  - Unknown parameters are dropped silently.
  - At lat 90 in 3857 it returns a finite y=242,528,680 m rather than an error or HUGE_VAL ([edge-merc-pole](fixtures/catalog/edge-merc-pole.md)).
- **proj: P.** PROJ's messages, plus "Unknown error (code 4096)" for an unknown code.
- **proj4rs: P.**
  - Distinct variants: `ProjectionNotFound` vs `InvalidParameterValue`.
  - But `ProjectionNotFound` is also the unknown-EPSG error, unknown parameters are silent, and lat 90 is a "Tolerance condition" error.
- **geodesy: P.**
  - `NotFound` for an operator, `General` for a bad value.
  - Datum and unknown parameters are ignored silently.
  - Per-point failure is NaN, not an error.
  - An unknown `ellps=` panics (`geodesy-0.15.0/src/op/parsed_parameters.rs:141`).
  - `cart` counts a point as failed if its t is NaN, which is geodesy's own `Coor2D` default (`geodesy-0.15.0/src/inner_op/cart.rs:13`).
- **miniproj: N.**
  - An `Option` with no reason.
  - No domain checks: lat 90 gives a finite wrong value ([edge-merc-pole](fixtures/catalog/edge-merc-pole.md), 4420 km).
  - Silent wrong answers: [method-ups-32761](fixtures/catalog/method-ups-32761.md), [form-2264-ftus](fixtures/catalog/form-2264-ftus.md).
- **proj-core: P.**
  - Strict and explicit, but "unrecognized CRS format" covers an unknown code, a missing geocentric CRS and an ESRI code alike.
  - The first-reported error for the 3857 string varies between runs (`+a` vs `+wktext`).
- **wbprojection: N.**
  - The error enum distinguishes `UnsupportedProjection` from `InvalidParameter`.
  - But it accepts `+lat_ts=95`, and most of its 33 "off" results are silent.
