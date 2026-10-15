# PROJ ports and rewrites, from Rust: a capability map

A coordinate library has to do two separate things:

- **resolve:** turn what you typed into a complete parameter set;
- **execute:** run those parameters.

PROJ's ports and rewrites fail the two independently. This repository
measures both for the Rust landscape. The same 93 fixtures are run through 11
candidate columns and judged against PROJ 9.9.0 itself, with network off and
no grids installed. The brief is in [docs/brief.md](docs/brief.md).

## Headline findings

- **The datum step is the expensive gap, and binding to WGS84 only partly
  closes it.**
  - The bare PROJ string for EPSG:2056 is 164 m off and the bound string is
    exact
    ([form-2056-proj-bare](fixtures/catalog/form-2056-proj-bare.md)).
  - PROJ declines to bind EPSG:27700, ED50 and GDA94, leaving ports 109 m,
    172 m and 1.5 m off.
- **Operation selection is no longer PROJ-only.** `proj-core` carries an EPSG
  registry and matches PROJ where every hub port is off
  ([pair-4283-7844](fixtures/catalog/pair-4283-7844.md)). It cannot transform
  custom CRSs.
- **`proj4rs` executes what it ports to 0.000 mm.** The exception is `eqc`,
  where PROJ 9.9 has moved to the ellipsoid
  ([method-eqc](fixtures/catalog/method-eqc.md)).
- **Several rewrites are silently wrong** by hundreds of metres to thousands
  of kilometres. The cases, with fixtures, are in
  [report.md](report.md#findings).
- **`proj4rs` (default features) and libproj cannot share a native binary.**
  The `geod_*` symbols clash and PROJ's aeqd returns NaN
  ([harness/README.md](harness/README.md#symbol-clash-why-there-are-two-builds)).

## Read

| | |
|---|---|
| [report.md](report.md) | findings, what each candidate is for, wrapper vs structural, changes from the TypeScript precedent |
| [capability-matrix.md](capability-matrix.md) | 15 layers by 8 candidates, with evidence |
| [results/per-fixture.md](results/per-fixture.md) | verdict grid: every fixture by every candidate |
| [fixtures/FIXTURES.md](fixtures/FIXTURES.md) | fixture catalog: a page per fixture with inputs, PROJ's answer and each candidate's result |
| [fixtures/README.md](fixtures/README.md) | oracle configuration and data schema |
| [harness/README.md](harness/README.md) | how candidates are fed and judged |
| [platform/README.md](platform/README.md) | wasm32, binary size, thread safety |

## Candidates

PROJ 9.9.0 (oracle) and the following crates:

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

Everything runs in `ghcr.io/hypertidy/gdal-r-python-extras`:
`./make-proj-nogrids.sh` and a one-time `cargo fetch`, then
`./regenerate.sh`. Details are in [report.md](report.md#reproducing) and
[PUBLISHING.md](PUBLISHING.md).
