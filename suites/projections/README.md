# Projections suite

The second suite. Where the [CRS suite](../crs/README.md) asks whether an
engine can resolve and execute a CRS-to-CRS transformation, this one asks a
narrower question that every port has to answer: given a bare PROJ
projection string, does the engine compute the same projection as PROJ, over
the whole area where the projection is used?

Results: [results/projections/README.md](../../results/projections/README.md).

## How an entry is run

Each entry in `projections.json` is a PROJ string and a lon/lat window. The
oracle (`oracle/projections.py`) takes the graticule intersections in the
window and runs them through PROJ with `Transformer.from_pipeline`, so the
projection runs exactly as written: no CRS interpretation, no datum step, no
axis swap. It records PROJ's forward result for every point and PROJ's
inverse of that result.

Every engine is then given the same two definitions: the projection string
and a lon/lat definition on the same earth model (`geographic` below), so no
datum shift is involved. It computes the forward result for every point, and
its inverse of PROJ's forward result. The judging code
(`report/projections.py`) belongs to no engine and compares:

- forward: distance from PROJ's result, in projected units (metres for every
  entry here);
- inverse: angular distance from PROJ's inverse, in degrees, longitude scaled
  by cos(latitude);
- points where PROJ answers and the engine does not, and the reverse.

PROJ's own round trip (its inverse against the input point) is reported
separately in [proj-check.md](../../results/projections/proj-check.md), so a
problem in the reference shows up as a PROJ problem, not as a port's.

## projections.json

| field | meaning |
|---|---|
| `id` | name of the entry; one page per id |
| `proj` | the PROJ string, as given to PROJ and to every engine |
| `lon`, `lat`, `step` | window and graticule spacing in degrees (default whole world every 30 degrees) |
| `why` | optional: what the entry is probing, for example a PROJ commit |
| `known` | optional: `{engine: note}` for a difference that is understood; the key also covers engines whose name extends it (`proj4rs` covers `proj4rs-main` and `proj4rs-wasm`) |

Add an entry, run `ORACLE=1 ./run-projections.sh`, and read its page.

## reference.json (schema_version 1)

`environment`: `schema_version`, `generated`, `proj_version`, `proj_git_sha`
(set when PROJ was built from a checkout), `pyproj_version`, `defaults`.

`projections[]`: every field of the entry, plus `window`, `geographic` (the
lon/lat definition engines are given), `points` (`[lon, lat]`), `fwd`
(`[x, y]` or null where PROJ gave no finite result) and `inv` (PROJ's inverse
of `fwd`, or null). `error` replaces `fwd` and `inv` when PROJ refused the
string.

## Engine output

One JSON line per (projection, engine), from `engines/rust-projections` and
`engines/js`: `projection`, `engine`, `engine_version`, `supported`, and
either `error` or `fwd` (one `[x, y]` or null per point) and `inv` (one
`[lon, lat]` or null per PROJ forward result).
