"""Summarise results/results.jsonl: verdict grid (fixtures x candidates) and
per-candidate counts. Usage: python3 report/summarise.py results/results.jsonl [--md]"""
import json, sys
from collections import OrderedDict, Counter

ABBR = {"exact": "exact", "close": "close", "off": "OFF", "failed": "fail", "unresolved": "--",
        "n/a": "n/a", "refused-like-oracle": "refuse", "answered-where-oracle-refused": "ANSWER",
        "axis-swapped": "SWAP"}

def fmt(r):
    v = r["verdict"]
    s = ABBR.get(v, v)
    if v in ("close", "off") and r.get("diff_m") is not None:
        d = r["diff_m"]
        s += " " + (f"{d:.3g}m" if d < 1000 else f"{d/1000:.3g}km")
    return s

def main():
    path = sys.argv[1]
    md = "--md" in sys.argv
    rows = [json.loads(l) for l in open(path)]
    rows = [r for r in rows if not r.get("skipped")]
    cands = list(OrderedDict.fromkeys(r["candidate"] for r in rows))
    fx = list(OrderedDict.fromkeys(r["fixture"] for r in rows))
    grid = {(r["fixture"], r["candidate"]): r for r in rows}
    if md:
        print("| fixture | " + " | ".join(cands) + " |")
        print("|---|" + "---|" * len(cands))
        for f in fx:
            print(f"| {f} | " + " | ".join(fmt(grid[(f, c)]) if (f, c) in grid else "" for c in cands) + " |")
    else:
        w = 17
        print(f"{'fixture':30s}" + "".join(f"{c[:w-1]:{w}s}" for c in cands))
        for f in fx:
            print(f"{f[:29]:30s}" + "".join(f"{fmt(grid[(f, c)])[:w-1]:{w}s}" if (f, c) in grid else " " * w for c in cands))
    print()
    for c in cands:
        n = Counter(grid[(f, c)]["verdict"] for f in fx if (f, c) in grid)
        print(c, dict(n))

main()
