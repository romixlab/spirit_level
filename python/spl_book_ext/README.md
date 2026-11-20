# spl_book_ext

Markdown extensions for a Zensical book's references in prose:

- `spl_book_ext.glossary`: `[[term]]` links a term of the book's glossary to its entry and shows its definition on hover; `[[shown words|term]]` shows other words. The glossary page's `<!-- glossary -->` line becomes every entry. An abbreviation's `long` form is shown on every use when the `abbr` extension is on.
- `spl_book_ext.code_refs`: `` [`Name`] `` names a public item of the book's crates as rustdoc's intra-doc links do; its card shows the signature and the first doc paragraph, the link opens its lines on the forge.

An unknown term or item fails the build. `spl book refs` checks the glossary and writes the code index and the cards' stylesheet and script before each build; spirit_level's book page "References in prose" has the settings.

Tests: `uv run --directory python/spl_book_ext pytest`.
