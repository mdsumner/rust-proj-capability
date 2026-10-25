"""Write the section cards on the front page.

Reads results/projections/summary.json (report/projections.py) and
results/tracker/watch.json (tracker/watch.py), and replaces the block
between the cards markers in README.md, so the numbers there follow the
nightly run. The block is HTML around markdown: it renders on GitHub Pages as
cards and on GitHub as plain links and text.

Usage (from the repository root):
  python3 report/front_page.py
"""

import json
import os
import re

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
README = os.path.join(ROOT, "README.md")
START, END = "<!-- cards:start -->", "<!-- cards:end -->"


def load(rel):
    path = os.path.join(ROOT, rel)
    if not os.path.exists(path):
        return None
    with open(path) as f:
        return json.load(f)


def card(link, title, text, number):
    # markdown="1" lets kramdown (Pages) read the markdown inside; GitHub
    # reads it because of the blank lines. Either way the links stay .md links.
    return (f'<div class="card" markdown="1">\n\n'
            f'**[{title}]({link})**\n\n'
            f'{text}\n\n'
            f'{number}\n\n'
            f'</div>')


def projections_card(summary):
    if summary is None:
        return card("results/projections/README.md", "Projections", "Every engine against PROJ, setup by setup.", "")
    rows = summary["results"]
    engines = summary["engines"]
    setups = len({r["projection"] for r in rows})
    env = summary["environment"]
    main = [r for r in rows if r["engine"] == "proj4rs-main" and r.get("supported")]
    exact = sum(r["verdict"] == "exact" for r in main)
    return card("results/projections/README.md", "Projections",
                f"{len(engines)} engines on {setups} projection setups, judged against "
                f"PROJ {env['proj_version']}, with a page and a map for each.",
                f"proj4rs main: **{exact} of {len(main)}** exact")


def watch_card(watch):
    if watch is None:
        return card("results/tracker/README.md", "PROJ watch", "PROJ projection changes, ported or not.", "")
    projs = watch["projections"]
    ported = [p for p in projs if p["proj4rs"]]
    changed = [p for p in ported if p["proj_commits"]]
    return card("results/tracker/README.md", "PROJ watch",
                f"Recent PROJ commits by projection, and whether proj4rs has each one. "
                f"{len(ported)} of {len(projs)} PROJ projections are in proj4rs.",
                f"**{len(changed)}** ported projections changed upstream {window(watch)}")


def window(watch):
    since = watch["environment"]["since"]
    m = re.fullmatch(r"(\d+) days ago", since)
    return f"in the last {m.group(1)} days" if m else f"since {since}"


def crs_card():
    # The CRS study is a fixed snapshot (2026-10-15, PROJ 9.9.0), so its numbers are too;
    # terrio-crs was added to it on 2026-10-25 against the same oracle.
    return card("crs-study.md", "CRS study",
                "Which Rust crate can take a CRS and run the transformation: "
                "93 fixtures, 13 columns, PROJ 9.9.0.",
                "best pure Rust: **terrio-crs, 71 of 93** (74 with its wbprojection fallback)")


def approach_card():
    return card("docs/approach.md", "Approach",
                "Benchmark first, drafted proj4rs ports second, occasional throwaway spikes third.",
                "proj4rs stays the model")


def main():
    summary = load("results/projections/summary.json")
    watch = load("results/tracker/watch.json")
    cards = [projections_card(summary), watch_card(watch), crs_card(), approach_card()]
    block = "\n".join([START, '<div class="cards" markdown="1">', "", "\n\n".join(cards), "", "</div>", END])
    with open(README) as f:
        text = f.read()
    pattern = re.compile(re.escape(START) + ".*?" + re.escape(END), re.S)
    if not pattern.search(text):
        raise SystemExit("README.md has no cards markers")
    new = pattern.sub(lambda m: block, text)
    if new != text:
        with open(README, "w") as f:
            f.write(new)
    print("wrote the README.md cards")


if __name__ == "__main__":
    main()
