# harness

One Rust binary with a cargo feature per candidate. It writes one JSON line
per (fixture, candidate), judged against `suites/crs/crs.json` by code that
belongs to none of the candidates (`src/judge.rs`). If a candidate's feature
is not built, the binary prints a single `{"skipped": true}` line for it and
carries on; it does not fail.

```
cargo run --release -- --candidate proj4rs ../../suites/crs/crs.json
cargo run --release -- --candidate all ../../suites/crs/crs.json
cargo run --release -- --list
cargo run --release -- -c proj-core -f pair-ed50-wgs84 ../../suites/crs/crs.json
```

In practice everything runs inside the extras image with the network off:
`../run-harness.sh`. That script builds the libproj columns and the pure-Rust
columns separately; the reason is under "Symbol clash" below.

## Candidates

| name | feature | what is fed |
|---|---|---|
| proj | c-proj | the definition as given, through `Proj::new` (resolve) and `Proj::new_known_crs` (execute; normalised for visualisation). The safe API is 2D only. |
| proj-sys | c-proj | the same libproj through raw FFI: 4D `proj_trans`, `proj_trans_get_last_used_operation`, and `proj_crs_create_bound_crs_to_WGS84` + `proj_as_proj_string` |
| proj4rs | c-proj4rs | PROJ strings as given; WKT through proj4wkt; authority codes and PROJJSON are refused |
| proj4rs+bound | c-proj4rs | the wrapper column: the bound-to-WGS84 PROJ strings that PROJ exported (`oracle.*_export.proj_bound`) |
| crs-definitions | c-crsdefs | EPSG codes looked up in the static table, executed by proj4rs. Non-authority fixtures are n/a. |
| geodesy | c-geodesy | PROJ strings through geodesy's own `parse_proj`, composed as `inv src \| dst` |
| geodesy+bound | c-geodesy | the wrapper column: PROJ's bound strings translated by the harness into a geodesy pipeline (ellipsoid, `+towgs84` as `cart \| helmert \| inv cart`, `+units` as `unitconvert`) |
| miniproj | c-miniproj | EPSG codes; the harness chains `src.projected_to_deg` and `dst.deg_to_projected` (no datum step exists) |
| proj-core | c-projcore | the definition as given through `proj_wkt::parse_crs` and `Transform::from_crs_defs` |
| proj-core+aoi | c-projcore | the same, with the source point passed as the area of interest |
| wbprojection | c-wbprojection | the definition as given: `Crs::from_epsg`, `from_proj_string`, `from_wkt`, then `transform_to` / `transform_to_3d` |

When a fixture's source is `OGC:CRS84`, candidates without an authority
lookup get `+proj=longlat +datum=WGS84 +no_defs` instead. The shared starting
point is not what is being tested.

## Output fields

| field | meaning |
|---|---|
| `resolved`, `resolve_error` | could it build both CRSs |
| `executed`, `exec_error` | did it return a finite forward result. NaN reported as success counts as not executed, with a note. |
| `fwd`, `oracle_fwd` | results in dst native units. Geographic results are in the CRS's angular unit, so EPSG:4807 is in grads. |
| `diff_m` | horizontal distance from the oracle, in metres. Geographic: degrees with longitude scaled by cos(lat), times 111320. Projected: native units times metres per unit. Geocentric: 3D distance. |
| `diff_swapped_m` | the same distance with the candidate's axes swapped |
| `dz_m` | difference in z |
| `inv`, `inv_diff_m` | the candidate's inverse of the oracle's forward result, compared with the oracle's inverse |
| `diff_vs_reference_m` | distance from the `compare_to` fixture's oracle result (for example, bare string vs authority code) |
| `operation` | the operation the candidate reports it used, where it reports one |
| `fed_src`, `fed_dst` | the definitions actually handed to the library |
| `verdict` | one of: `exact` (<1 mm), `close` (<1 m), `off`, `axis-swapped`, `failed`, `unresolved`, `n/a`, `refused-like-oracle` (the oracle refused too), `answered-where-oracle-refused` |

## Symbol clash (why there are two builds)

proj4rs's default `aeqd` feature compiles proj4rs-geodesic, a copy of
geodesic.c. It exports `geod_init`, `geod_direct` and the other `geod_*`
functions from the executable. Those definitions interpose on libproj's own,
so in a binary that links both, PROJ's aeqd returns NaN without setting an
error. `results/crs/combined-build.jsonl` shows this: proj and proj-sys fail
`method-aeqd` in the all-features build and are exact in the libproj-only
build.
