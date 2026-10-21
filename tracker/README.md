# Tracker

`watch.py` lists the projections in PROJ's `src/projections/`, which of them
proj4rs has, which the [projections suite](../suites/projections/README.md)
covers, and the PROJ commits that touched each one recently. Output:
[results/tracker/README.md](../results/tracker/README.md) and `watch.json`.

```
tracker/checkout.sh                          # shallow PROJ and proj4rs checkouts in tracker/.cache/
python3 tracker/watch.py --proj tracker/.cache/PROJ --proj4rs tracker/.cache/proj4rs --since "180 days ago"
```

A PROJ change to a ported projection is a prompt to check the suite page for
that projection and, if the ports disagree, to port the change. Nothing here
opens issues or pull requests anywhere; a person does that.
