## Why

Zensical treats a navigation entry that does not resolve to a page as a link to an external site. A misspelled or moved page in `nav` does not fail the build: the navigation shows a link that leads nowhere.

The lint checks every entry of `nav` in `zensical.toml` that is not a URL: it must name a file in the docs folder.

## How to fix

Correct the path. Paths in `nav` are relative to the docs folder (`docs_dir`, `docs` by default). Drop the entry if the page is gone.
