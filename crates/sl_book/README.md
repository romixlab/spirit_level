# sl_book

Writes the generated part of the book under `docs/`: `lints/<id>.md` for every lint (the rule, its category and source, whether `--fix` fixes it, what it needs, its level and reason in every tier, its parameters, then the lint's own documentation), `lints/index.md`, `tiers/<name>.md` and `tiers/index.md`, and the nav block of `zensical.toml` between its marker comments.

Everything comes from `LintMeta` and the shipped tier files, so a page cannot drift from the code: `spl book docs --check` (run by `just lint`) fails when a page is stale. The hand-written pages (the introduction, the configuration format, the crate map, waivers, output, the tools table) live beside them.
