# PROJ ports and rewrites: a benchmark

PROJ is ported and rewritten again and again, in Rust, in JavaScript and as
WebAssembly. This repository is a shared benchmark for all of them: the same
inputs, PROJ's own answers, and judging code that belongs to no engine,
rerun every night against PROJ master. It is meant as a reference the ports
can point to when they plan, compare and negotiate, whether or not PROJ
itself adopts it. [Approach](docs/approach.md) explains the layers and what
is decided.

<!-- cards:start -->
<div class="cards" markdown="1">

<div class="card" markdown="1">

**[Projections](results/projections/README.md)**

8 engines on 54 projection setups, judged against PROJ 9.10.0, with a page and a map for each.

proj4rs main: **36 of 40** exact

</div>

<div class="card" markdown="1">

**[PROJ watch](results/tracker/README.md)**

Recent PROJ commits by projection, and whether proj4rs has each one. 35 of 167 PROJ projections are in proj4rs.

**1** ported projections changed upstream in the last 180 days

</div>

<div class="card" markdown="1">

**[CRS study](crs-study.md)**

Which Rust crate can take a CRS and run the transformation: 93 fixtures, 11 columns, PROJ 9.9.0.

best pure Rust: **proj-core, 33 of 93**

</div>

<div class="card" markdown="1">

**[Approach](docs/approach.md)**

Benchmark first, drafted proj4rs ports second, occasional throwaway spikes third.

proj4rs stays the model

</div>

</div>
<!-- cards:end -->

Engines in the projections suite: proj4rs (the crates.io release, main, and
main as wasm), proj-core, wbprojection, geodesy, proj4js and math.gl. To add
one, see [engines/README.md](engines/README.md); to add a projection, see
[suites/projections/README.md](suites/projections/README.md).

## Findings so far

As of 2026-10-21, 54 projection setups:

- **proj4rs main matches PROJ to the millimetre on 36 of the 40 setups it
  supports**, natively and as wasm. On ellipsoidal setups it is within about
  5 nm forward and 3e-11 deg inverse, since main adopted PROJ's newer
  meridional distance and authalic latitude series. Its latest crates.io
  release (0.2.1) supports 29 setups and is only "close" on laea and cea;
  ortho, gnom, omerc, tpers, nsper, cass, eqdc, robin, eqearth, poly and
  sinu are on main only.
- **proj4rs had not followed two PROJ fixes to spherical tmerc.** With `+R`
  and a non-zero `lat_0`, the inverse returned the wrong sign of latitude
  north of lat_0 (fixed in PROJ in 2020,
  [bc0c9f3e](https://github.com/OSGeo/PROJ/commit/bc0c9f3e96dc4a9829f1f3d1e1307b4dd5dd10de));
  the forward was unstable at the equator (PROJ
  [ca23885d](https://github.com/OSGeo/PROJ/commit/ca23885da621269333eb00c5010e2ff2ac3423f0),
  2026-02). Both are offered upstream in
  [3liz/proj4rs#61](https://github.com/3liz/proj4rs/pull/61)
  ([tmerc-sphere](results/projections/pages/tmerc-sphere.md)).
- **math.gl and proj4js agree with each other and differ from PROJ on
  ellipsoidal ortho by 33.7 km** ([ortho](results/projections/pages/ortho.md));
  both compute it on a sphere. math.gl 5.0.0-alpha.18 matches PROJ on 29
  setups.
- **PROJ's own round trip is checked too**
  ([proj-check.md](results/projections/proj-check.md)), so a problem in the
  reference is reported as PROJ's, not as a port's.

## How it runs

A nightly workflow (`.github/workflows/benchmark.yml`) builds PROJ master,
recomputes the projections reference, follows proj4rs main, reruns every
engine, and commits the regenerated pages. To run it locally (python3 with
pyproj, cargo, node, and wasm-pack for the proj4rs-wasm column):

```
engines/js/build-proj4rs-wasm.sh       # proj4rs main as a Node wasm module
ORACLE=1 ./run-projections.sh          # PROJ reference with the installed pyproj, every engine, pages
oracle/build-proj.sh master            # optional: PROJ and pyproj built from a PROJ ref, as the nightly does
tracker/checkout.sh && python3 tracker/watch.py --proj tracker/.cache/PROJ --proj4rs tracker/.cache/proj4rs
```

The CRS study has its own instructions in [crs-study.md](crs-study.md#reproduce).
