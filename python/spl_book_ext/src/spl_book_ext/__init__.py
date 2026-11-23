"""A book's references in prose, as Python-Markdown extensions that Zensical loads from zensical.toml.

- `spl_book_ext.glossary`: `[[term]]` links a glossary term to its entry and shows its definition on hover.
- `spl_book_ext.code_refs`: `` [`Name`] `` links a public item of the book's crates and shows its signature and docs.
- `spl_book_ext.abbreviations`: an abbreviation of the project's table shows its long form on hover (`<abbr>`).
- `spl_book_ext.shell`: the `console` fence's lexer: commands, subcommands and flags in their own colours.
- `spl_book_ext.rust`: the `rust` fence's lexer: every macro call marked, not only the standard ones.

Both draw the same card (classes `spl-ref`, `spl-card`), styled by the stylesheet `spl book refs` writes. A card
opens on hover and on focus, so a tap or the keyboard opens it too. A name that resolves to nothing, or to several
things, raises an error that fails the build.
"""
