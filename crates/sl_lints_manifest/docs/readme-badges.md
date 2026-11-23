## Why

The top of the README is where a visitor decides whether the project is alive and where to read on. A row of badges answers both at a glance: the CI status of the default branch, the published version, the API docs, the book. A badge that links somewhere else, or a project that has a book nobody can find from its README, wastes that place.

What the README must carry follows from what the repo has, read from its own files:

| The repo has | Badge links to | Image |
|---|---|---|
| a book with `site_url` in `zensical.toml` or `mkdocs.yml` | the `site_url` | any |
| the CI workflow (parameter `workflow`) and a GitHub `repository` | `https://github.com/OWNER/REPO/actions/workflows/ci.yml` | that page's `badge.svg` |
| a published crate (`publish` not false) | `https://crates.io/crates/NAME` | any |
| a published crate with a library | `https://docs.rs/NAME` | any |
| its own `spirit_level.toml` (parameter `badge` not empty) | the book's page of the tier it extends (`https://spl.vhrd.tech/tiers/medium/`, parameter `book`) | the file `badge`, `.github/spl-badge.svg` |

The spirit_level badge is the one [`spl badge`](../badge.md) writes. The committed file must hold exactly what `spl badge` writes for the configuration: a badge that no longer shows the tier or how strict the configuration is, after a change to `spirit_level.toml` or a new release of spirit_level, is a finding too.

A badge is a linked image: `[![alt](image)](link)` in Markdown, or `<a href="link"><img src="image"></a>` in HTML. A link may go deeper than the address in the table (`https://docs.rs/NAME/latest/NAME/`) or carry a query (`?query=branch%3Amain`). Badges in code blocks and HTML comments do not count.

```markdown
[![CI](https://github.com/OWNER/REPO/actions/workflows/ci.yml/badge.svg)](https://github.com/OWNER/REPO/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/NAME)](https://crates.io/crates/NAME)
[![docs.rs](https://img.shields.io/docsrs/NAME)](https://docs.rs/NAME)
[![Book](https://img.shields.io/badge/book-book.example.org-blue)](https://book.example.org/)
[![spirit level](.github/spl-badge.svg)](https://spl.vhrd.tech/tiers/medium/)
```

## How to fix

Add the badge the finding names. `spl check --fix` inserts it after the README's existing badges, in their form (Markdown or HTML). Without any badges, each one goes in a block of its own before the first paragraph, after a title, a logo or an HTML header; put them on consecutive lines to show them in one row. A CI badge that shows another image is changed by hand to show the workflow's `badge.svg`. `--fix` also writes the spirit_level badge where it is missing or stale; `spl badge` does the same alone. Set `badge = ""` for a repo that shows no spirit_level badge.
