# sl_lints_docs

spirit_level's docs lints: the shape of a page as a reader meets it (runs of prose between visual breaks, reading time, heading length, sections, walls of code, images without alternative text, code blocks without a language) and the practices of a Zensical book (navigation entries that exist, no orphan page, one title per page, admonitions that render).

One analysis, `DOCS`, over the pages of the `[docs]` table and the book of `zensical.toml`; `report` gives `spl pages` the same numbers. Each lint's page is in `docs/<id>.md`; fixtures in `tests/fixtures/<id>/`.
