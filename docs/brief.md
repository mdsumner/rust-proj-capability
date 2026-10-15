# Brief: a capability map of PROJ ports and rewrites, from Rust

16 October 2026. Context and the question, what to measure, how, and what
to hand back. Written to be farmed out; nothing here needs the author.

## The question

A coordinate library answers two separate questions for any CRS you give
it, and ports and rewrites of PROJ fail them independently:

1. **CRS input (resolve).** Can it turn what you typed into a complete
   parameter set: method, parameters, ellipsoid, datum, units, axis order?
   An authority code (`EPSG:3031`) is a lookup. A PROJ string, WKT or
   PROJJSON already carries the parameters, but may carry things the
   library cannot read.
2. **Transform execution (execute).** Given the parameters, can it run
   them: does it have the projection method, the datum step, the grid, and
   the policy for choosing between candidate operations?

The TypeScript precedent for this exercise is the sidebyside `crs/` page
(mdsumner/sidebyside, `crs/index.html`), which runs @math.gl/projection
against PROJ-in-wasm and tiers every input as built in / needs an
authority / needs PROJ / unknown to both. The findings there are the
starting hypotheses here: the port executes ported methods to 0.000 mm;
the lookup is the common gap; the datum step is the expensive gap (a bare
proj string of EPSG:2056 is 164 m off, a bound-to-WGS84 string is exact);
operation selection (GDA94 to GDA2020, 1.5 m) lives only in the database.

We want the same map for the Rust landscape, against PROJ itself as the
reference.

## Candidates

| candidate | what it is | role in the map |
|---|---|---|
| PROJ (libproj, 9.x) | the reference implementation: database, operation selection, grids, network | the oracle; every other column is measured against it |
| `proj` (georust) + `proj-sys` | Rust bindings to libproj | expected to be PROJ with a build cost; confirm what the safe API exposes and what it does not (contexts, normalize-for-visualization, operation factory, last-used operation, PROJJSON export, grid/network control) |
| `proj4rs` (3liz) | pure Rust port of the proj4 lineage, wasm-capable | the port tier; expected to be methods without database |
| `geodesy` (busstoptaktik) | a Rust geodesy library by a PROJ contributor; operator pipelines, own syntax, not aiming at parity | the rethink tier; expected to be strong on pipeline composition, absent on authority input |
| `crs-definitions` | static EPSG-to-proj-string table (from proj4js) | the authority stand-in that ports lean on; measure what its strings lose |
| (optional) `@math.gl/projection` 5.0 alpha | the TS port deck.gl v10 adopts | one column for cross-language comparison; results already exist on the sidebyside page |

Add any other Rust crate that claims CRS transformation if it is
maintained and used (check crates.io reverse dependencies before adding).
Record each candidate's version and date.

## Layers to map

Score each candidate on each layer: yes / partial (say what) / no, with
the evidence (a fixture id, a doc link, or a code reference).

CRS input:

- I1 authority codes: `EPSG:`, `ESRI:`, `IAU:`, `OGC:CRS84`; deprecated codes; what "lookup" means (bundled database, static table, none)
- I2 PROJ strings: full parameter vocabulary or a subset; behaviour on unknown parameters (reject, ignore, warn); `+type=crs`, `+wktext`, `+nadgrids=@null`
- I3 WKT1 (GDAL and ESRI dialects), WKT2:2015/2019
- I4 PROJJSON
- I5 compound, bound, derived, vertical and engineering CRSs; what is dropped and whether the drop is reported
- I6 axis order and units: authority order vs visualisation order; feet, grads, radians; `proj_normalize_for_visualization` or equivalent
- I7 export: can it emit PROJ string / WKT / PROJJSON back, and is a round trip stable

Transform execution:

- E1 projection methods: count and list; include the awkward ones (tpers, geos, ob_tran, omerc two-point, krovak, nzmg, igh, wintri, eck4, natearth, hammer, qsc, healpix/rhealpix, isea)
- E2 datum via Helmert: `+towgs84` 3 and 7 parameter, frame rotation convention (position vector vs coordinate frame), time-dependent 14/15 parameter
- E3 grid shifts: NTv2, GeoTIFF (GTG), `+nadgrids`, optional grids (`@`), loading from disk / network / embedded; behaviour when a grid is missing (error, ballpark, silent identity)
- E4 operation selection: given two CRSs, does it choose an operation from a database by area of use and accuracy, fall back to ballpark, report which it used; or does it hub through WGS84 with whatever the definitions say
- E5 3D and dynamic: geocentric, ellipsoidal height, vertical datums and geoid grids, epochs and point motion
- E6 numerical fidelity: forward and inverse against PROJ on a fixed corpus, per method, with observed maxima; near-pole, near-seam and antipodal behaviour; float64 throughout
- E7 platform: `wasm32-unknown-unknown`, `no_std`, thread safety, build dependencies (cmake, sqlite, tiff, curl), binary and data size shipped
- E8 failure behaviour: explicit error vs NaN vs HUGE_VAL vs silent wrong answer; is "unsupported" distinguishable from "invalid input"

## Method

This is a harness, not a reading exercise. Same inputs through every
candidate, judged by code that belongs to none of them.

1. **Fixtures.** Start from the sidebyside `crs/` seed list (EPSG:4326,
   3857, 32755, 3031, 28355, 7855, 9354, 8857, ESRI:54042, ESRI:54052,
   ESRI:102020, tpers, ob_tran+moll, wintri, igh, EPSG:7415, 2056, 2154,
   a bad code) and the pairs (4283 to 7844, 4283 to 7855, 28355 to 7855,
   4326 to 3031). Add: a NAD27 to NAD83 pair (grid), an ED50 to WGS84 pair
   (several Helmert candidates), one dynamic-datum pair with an epoch, one
   vertical/compound with a geoid grid, one each of WKT1-ESRI, WKT2 and
   PROJJSON for the same CRS, and a feet-unit state plane. One test point
   per fixture inside its area of use.
2. **Oracle.** pyproj pinned to a stated PROJ version, **network off, no
   `proj-data` grids installed**: one configuration, the same as the
   browser page, so results are comparable and a missing-grid case shows
   what each candidate does when the best operation is unavailable (PROJ
   falls back to the next candidate or a ballpark; record which). Generate
   reference forward and inverse coordinates and the name of the operation
   PROJ selected (`proj_trans_get_last_used_operation`). Do not enable
   network at any point; if a fixture needs a grid to mean anything, keep
   it and report the fallback rather than fetching the grid.
3. **Harness.** One Rust binary with a feature flag or subcommand per
   candidate, reading the fixture file and writing one JSON line per
   (fixture, candidate): resolved yes/no and the error text, executed
   yes/no and the error text, the point, and the difference from the
   oracle in metres (degrees times 111320 for geographic targets). Missing
   candidates are skipped, not failed (trianglewins convention).
4. **Report.** A capability matrix (layers by candidates), the per-fixture
   table, and one paragraph per candidate saying what it is for. Where a
   candidate fails a layer, say whether a wrapper could supply it (as the
   bound-to-WGS84 export supplies the datum step to a port) or whether it
   is structural (operation selection without a database).

## Things already known, to verify rather than rediscover

- PROJ's bare proj-string export of a CRS drops the datum relationship;
  `proj_crs_create_bound_crs_to_WGS84` before export carries it as
  `+towgs84`. Any port fed strings should be fed bound strings.
- A compound CRS exports with `+geoidgrids`/`+geoid_crs`; hand ports the
  horizontal member.
- `+wktext` appears in PROJ's export of EPSG:3857 and some ports reject it.
- Authority axis order (EPSG:4326 is lat/lon) bites every comparison;
  normalise for visualisation on the oracle side and say so.
- proj4rs is the backend of rwarp-wasm (hypertidy/rwarp) through a
  `CrsTransform` trait that also takes libproj natively and a
  caller-supplied closure; that trait is the shape the conclusions should
  be written toward.

## Environment

The `gdal-r-ci` Docker image chain (gdal-system -> gdal-r -> gdal-r-full
-> gdal-r-python -> gdal-r-python-extras) is available and is the place to
run this: it has a current GDAL and PROJ built from source with a known
version, Python with pyproj against that same libproj, R, and the Rust
toolchain in the extras image. Use the extras image. Record the PROJ
version it carries (`projinfo --version`, `pyproj.proj_version_str`) in
the fixtures. Build the georust `proj` crate against the image's libproj
(`PROJ_SYS_*`/pkg-config), not `bundled_proj`, so the oracle and the
binding share one PROJ. Keep the container's network off for the oracle
and harness runs (`--network none` after dependencies are fetched).

## Deliverables

- `capability-matrix.md`: layers by candidates, with evidence links.
- `fixtures/`: the CRS list, test points, and oracle output with the PROJ
  version and grid configuration recorded.
- `harness/`: the Rust crate, runnable with `cargo run -- --candidate
  proj4rs fixtures/crs.json`, output as JSON lines.
- `report.md`: findings, one paragraph per candidate, the wrapper-vs-
  structural list, and what changed from the TypeScript precedent.

Keep all content ASCII. Point at repos and issues, not people.
