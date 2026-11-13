# Writing a lint

A lint is declared once, beside its code, and everything else is made from that declaration: the configuration check, the book page, `explain`, the tier tables.

```rust
sl_analysis::lint!(CRATE_LINES, "crate-lines", "size", "a crate's src/ stays under its line limit",
    params: [("max-lines", Int, "20000", "code lines above which a crate is reported")],
    needs: [], fix: None);
```

- **id**: kebab-case, stable forever.
- **category**: the group in the book.
- **summary**: the rule as a statement, lower case, no full stop.
- **params**: name, kind (`Int`, `Float`, `Bool`, `Str`, `StrList`), default as a TOML literal, one doc line.
- **needs**: `Need::Resolve` (the full dependency graph), `Need::Map`, `Need::Tool("cargo-deny")`, `Need::Nightly`.
- **fix**: `Exact` only when the edit is the whole fix.

The documentation is `docs/<id>.md` in the lint's crate: `## Why`, an example where it helps, `## How to fix`.

An analysis implements one trait:

```rust
pub trait Analysis: Sync {
    fn name(&self) -> &'static str;
    fn lints(&self) -> &'static [&'static LintMeta];
    fn run(&self, cx: &Context<'_>);
}
```

`run` reads `cx.project` (cargo metadata, manifests, sources), `cx.params(id)` and `cx.map()`, and emits `Diagnostic`s; the engine sets their levels, applies exceptions and waivers, and reports. Tests are golden fixtures: a `pass/` and a `fail/` project per lint with an `expect.toml` of the findings.
