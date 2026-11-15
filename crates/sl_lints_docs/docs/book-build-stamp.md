## Why

A reader of a built book cannot tell which version of the project they are looking at, or how stale the page is: two tabs of the same URL opened months apart look the same. `exact_units`' book answers this in its footer: the crate's version, the git commit it was built from (`-dirty` when the tree had changes) and the build time in UTC, written into a generated `overrides/partials/stamp.html` that `overrides/partials/copyright.html` includes. A small script turns the build time into the age the reader actually cares about, coarser as it grows ("built 40 s ago", "12 min ago", "3 h ago", "5 days ago"), with the absolute UTC time kept beside it.

The lint looks for this practice: an override partial (or the built site) that carries a build stamp, and a script that shows the stamp's age from the reader's own clock, not only its absolute time.

## How to fix

Give the theme a `custom_dir` (`[project.theme] custom_dir = "overrides"` in `zensical.toml`), add an override partial your footer includes (`partial`, `partials/copyright.html` by default) that pulls in a generated stamp with the crate's version, commit and build time, and add a script, inline or as an `extra_javascript` file, that renders the build's age from `Date.now()`, coarser as it grows. See the pattern in `exact_units` (`docs_ext/eu_docs_ext/stamp.py`, `overrides/partials/copyright.html`).
