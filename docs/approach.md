# Approach

What this repository is for, how the work is layered, and what is decided.
Written 2026-10-21; the [front page](../README.md) has the current numbers.

## In one paragraph

PROJ is ported again and again (proj4rs, proj4js, math.gl, proj-core,
wbprojection and more), and every port drifts from PROJ in its own way.
Rather than merge codebases, this repository gives all of them one shared
benchmark: the same inputs, PROJ's own answers, and judging code that
belongs to no engine, rerun every night against PROJ master. Ports can
point to it when they plan, compare and negotiate, whether or not PROJ
itself adopts it. Work on proj4rs, the port that mirrors PROJ file by file,
sits on top of the benchmark in two further layers.

## Three layers

| layer | what | runs | kept? |
|---|---|---|---|
| 1. Benchmark | every engine scored against PROJ master on the projections suite, plus a watch of PROJ commits | nightly, automatically | yes, this repository |
| 2. Drafted ports | a proj4rs branch for a new or changed PROJ projection, with tests and its benchmark result | when the watch shows a change, or on request | yes, offered upstream as a PR once a person has read it |
| 3. Spikes | a throwaway branch that pre-develops a move proj4rs is likely to make later, to surface big issues early | occasionally | no, deleted once the findings are written up |

Everything that leaves this repository (a PR, an issue, a mailing list
note) is written and filed by a person. Fetching, building, running and
regenerating pages is automated.

## Decision: proj4rs stays the model

Decided 2026-10-21. proj4rs remains the port this work contributes to;
there is no separate port of our own. That holds while proj4rs keeps
mirroring PROJ and taking contributions, and is revisited if it stops.

Spikes (layer three) are how to look ahead without forking the effort.
Candidates: ellipsoidal `eqc` (EPSG method 1028, in PROJ since 2026-01),
and a CRS and pipeline layer above the projections. A spike is a branch in
a fork, run through the benchmark as an extra engine column, and written up
as a note to the proj4rs maintainers. It is not offered as a PR unless they
ask for that direction.

Tradeoff: a spike that is kept alive becomes a second port to maintain, so
each one is deleted once its findings are written up.

### Running a spike

1. Branch from proj4rs main in a fork.
2. Add a column for it in `engines/rust-projections/Cargo.toml`, as a git
   dependency renamed like `proj4rs-main` (for example
   `proj4rs-spike-eqc`), and register it in `src/engines.rs` with the
   `proj4rs_adapter!` macro.
3. `./run-projections.sh`, then read the grid and the pages it touches.
4. Write the findings up, then remove the column and delete the branch.

### Signals that proj4rs may stop being the right model

Planned for the benchmark; not yet tracked automatically.

- **Lag behind PROJ:** changed PROJ projections still unported, and for how long ([PROJ watch](../results/tracker/README.md)).
- **Release lag:** work on proj4rs main that is not in a crates.io release (the grid's `proj4rs` and `proj4rs-main` columns).
- **PR turnaround:** time from opening a PR to merging it, and the share that merge.
- **Accuracy trend:** exact and close counts per release, and any regression.
- **Spike friction:** how hard a spike was to fit into proj4rs's structure. A spike that keeps fighting the architecture is the earliest sign.
- **Alternatives catching up:** another column passing proj4rs on coverage or accuracy for the projections downstream users depend on.

## What is where

| | page |
|---|---|
| verdict grid, a page and a map per projection | [Projections](../results/projections/README.md) |
| PROJ's own round trip | [results/projections/proj-check.md](../results/projections/proj-check.md) |
| how the projections suite is judged, and how to add a projection | [suites/projections/README.md](../suites/projections/README.md) |
| PROJ commits by projection, ported or not | [PROJ watch](../results/tracker/README.md) |
| every engine, and how to add one | [engines/README.md](../engines/README.md) |
| the 2026-10-15 CRS study (resolve and execute, Rust crates, PROJ 9.9.0) | [CRS study](../crs-study.md) |
| how the site is built | [PUBLISHING.md](../PUBLISHING.md) |
