# spl_book_ext

Markdown extensions for a Zensical book's references in prose:

- `spl_book_ext.glossary`: `[[term]]` links a term of the book's glossary to its entry and shows its definition on hover; `[[shown words|term]]` shows other words. The glossary page's `<!-- glossary -->` line becomes every entry. An abbreviation's `long` form is shown on every use when the `abbr` extension is on.
- `spl_book_ext.abbreviations`: each abbreviation of the `abbreviation-first-use` lint's `long-forms` table shows its long form on hover, through Python-Markdown's `abbr` extension, which must be on. `spl book refs` writes the table from `spirit_level.toml`.
- `spl_book_ext.code_refs`: `` [`Name`] `` names a public item of the book's crates as rustdoc's intra-doc links do (`fn@`, `type@` and `cfg(...)@` pick one of items sharing a path); its card shows the signature and the docs, the link opens its lines on the forge. `` [`file.rs`] `` and `` [`file.rs:section`] `` name a file or a marked section of one; the card shows its code.

An unknown term, item, file or section fails the build and names the page and line. Install the package at the release tag of the `spl` in use. `spl book refs` checks the glossary and writes the code index and the cards' stylesheet and script before each build; spirit_level's book page "References in prose" has the settings.

Tests: `uv run --directory python/spl_book_ext pytest`.
