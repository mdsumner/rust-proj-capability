# Publishing notes

For whoever renders this tree as web pages (for example GitHub Pages).
Everything needed is in the repository; nothing below needs the author. The
CRS suite pages are final as of 2026-10-15. The projections suite and the
PROJ watch are regenerated nightly by `.github/workflows/benchmark.yml` and
committed to main, so the site follows them without a separate build step.

## Site map, in reading order

| page | kind | what it is |
|---|---|---|
| [README.md](README.md) | hand-written | landing page: what this is, where to look, findings so far |
| [docs/approach.md](docs/approach.md) | hand-written | the three layers (benchmark, drafted ports, spikes), the proj4rs decision, what is where |
| [results/projections/README.md](results/projections/README.md) | generated nightly | projections suite: verdict grid, projections by engines |
| `results/projections/pages/*.md` | generated nightly | one page per projection: metrics per engine and a map (`results/projections/maps/*.svg`) |
| [results/projections/proj-check.md](results/projections/proj-check.md) | generated nightly | PROJ's own round trip per projection |
| [results/tracker/README.md](results/tracker/README.md) | generated nightly | PROJ change watch |
| [suites/projections/README.md](suites/projections/README.md) | hand-written | how the projections suite is run and judged, and its schema |
| [engines/README.md](engines/README.md) | hand-written | every engine, and how to add one |
| [tracker/README.md](tracker/README.md) | hand-written | the PROJ change watch |
| [crs-study.md](crs-study.md) | hand-written | CRS study landing page: the Rust crate guide and links to its pages |
| [report.md](report.md) | hand-written | findings, one paragraph per candidate, wrapper-vs-structural, the rwarp `CrsTransform` angle, changes from the TypeScript precedent |
| [capability-matrix.md](capability-matrix.md) | hand-written | 15 layers by 8 candidates, summary grid and evidence |
| [results/crs/per-fixture.md](results/crs/per-fixture.md) | generated | verdict grid: 93 fixtures by 11 candidate columns |
| [suites/crs/FIXTURES.md](suites/crs/FIXTURES.md) | generated | fixture catalog: one row per fixture |
| `suites/crs/catalog/*.md` (93 pages) | generated | one page per fixture: inputs, what PROJ did, every candidate's result |
| [suites/crs/README.md](suites/crs/README.md) | hand-written | oracle configuration and the JSON schema |
| [results/crs/README.md](results/crs/README.md) | generated | result files and verdict counts |
| [engines/rust-crs/README.md](engines/rust-crs/README.md) | hand-written | candidates, output fields, the symbol clash |
| [platform/README.md](platform/README.md) | hand-written | wasm32, size and thread-safety checks |
| [docs/brief.md](docs/brief.md) | hand-written | the original brief |

The navigation bar (`_layouts/default.html`) has five sections: Home /
Approach / Projections / PROJ watch / CRS study. Pages in the Projections
and CRS study sections get a second row of links to that section's pages;
which section a page belongs to is decided from its path in the layout.

## Data and code (link to them, do not render as pages)

| file | size | what |
|---|---|---|
| `suites/crs/crs.json` | 0.3 MB | the oracle output (schema in suites/crs/README.md) |
| `suites/crs/fixtures-src.json` | small | the fixture list |
| `results/crs/results.jsonl` | 0.8 MB | one JSON object per fixture and candidate |
| `results/crs/combined-build.jsonl`, `results/crs/summary.txt`, `results/crs/stderr.txt` | small | evidence and plain-text views |
| `platform/*.txt`, `platform/*.sh` | small | E7 raw output and scripts |
| `suites/projections/projections.json`, `reference.json` | 0.5 MB | the projections catalogue and PROJ's reference points |
| `results/projections/summary.json`, `results/tracker/watch.json` | small | judged metrics per (projection, engine); the watch data |
| `engines/` (Rust and JavaScript), `oracle/*.py`, `report/*.py`, `tracker/*.py`, `*.sh` | | code |

## Rendering facts

- **Markdown flavour.** GitHub-flavoured: pipe tables and fenced code. The
  fixture pages use `<details><summary>` HTML blocks to fold long WKT and
  PROJJSON.
- **Links.** All links are relative and point at `.md` files. All 104 pages
  were checked and none is broken.
  - GitHub Pages' default Jekyll rewrites `.md` links to `.html`
    (jekyll-relative-links).
  - Another generator must do the same rewrite.
- **Front matter.** There is none. GitHub Pages renders markdown without it
  (jekyll-optional-front-matter) and takes titles from the first heading.
  Every page starts with a `#` heading.
- **Source folder.** Serve from the repository root, not `/docs`: `/docs` only
  holds the brief.
- **Tables are wide.** The verdict grid and the fixture pages have up to 12
  columns. A theme should let tables scroll horizontally rather than wrap.
- **Encoding.** All content is ASCII. Non-ASCII from libraries (degree signs)
  is escaped as `\u00b0` in JSON and as `&#176;` in pages.
- **Generated pages** start with an HTML comment
  `<!-- generated by report/make_docs.py ... -->`. Regenerate them; do not
  edit them.
- **Exclude** from any upload what `.gitignore` lists: build directories
  (`.target*`), the cargo cache, `.home/` and `.proj-nogrids/` (a 10 MB copy
  of proj.db).

## Regenerating

`./regenerate.sh` reruns the oracle, the harness and the page generator in
the extras image, with the network off. The prerequisites, listed at the top
of that script, are done once.

The hand-written pages quote numbers (verdict counts, distances). After a
rerun with new library versions, check them against
[results/crs/README.md](results/crs/README.md) and the fixture pages.
