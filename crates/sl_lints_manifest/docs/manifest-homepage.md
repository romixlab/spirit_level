## Why

crates.io and docs.rs show a crate's `repository` and `homepage` as its links. Without `repository`, a reader of crates.io cannot find the source, and the README's CI badge has no address to point at. When the project publishes a book, the book is the homepage a reader wants; a `homepage` that names something else, or nothing, sends them past it.

The keys are read where they are written once: `[workspace.package]` in a workspace, the root `[package]` for one crate. The book's address is `site_url` in `zensical.toml` or `mkdocs.yml`; without one, only `repository` is checked.

```toml
[workspace.package]
repository = "https://github.com/OWNER/REPO"
homepage = "https://book.example.org/"
```

## How to fix

Add `repository`. Set `homepage` to the book's `site_url`; `spl check --fix` writes it. Members inherit both with `repository.workspace = true` and `homepage.workspace = true`.
