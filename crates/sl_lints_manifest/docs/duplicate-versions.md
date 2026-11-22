## Why

Two versions of one crate compile twice and give two incompatible copies of its types (a `syn::Ident` from syn 2 is not one from syn 3). In a big graph they are the cheapest build time to win back.

## How to fix

`cargo tree -d -i NAME` shows who pulls each version. Align the version requirements, move a dependency to the version its siblings use, or `cargo update`. The graph is the one cargo-deny judges: with a `deny.toml` that names `[graph] targets`, only what is built for those targets; an optional dependency only a weak feature (`dep?/feat`) names is not in it. A crate reached only through dev-dependency edges is dropped too, cargo-deny's own default (its `[bans] multiple-versions-include-dev` adds it back, in deny.toml and here alike); `cargo tree -i NAME -e normal,build,dev` finds such a path. Features are resolved for each checked crate alone ([what each crate builds](../configuration.md#what-each-crate-builds)): a version only another crate's feature pulls in is not a duplicate of this crate's.

When two versions cannot be avoided yet, name them in `deny.toml`'s `[bans]`, the one list cargo-deny reads too: `skip` for a version, `skip-tree` for a crate and what it pulls in.

```toml
[bans]
skip = [{ crate = "syn@1.0.109", reason = "via geiger" }]
skip-tree = [{ crate = "hakari", reason = "its graph libraries pin older versions" }]
```

A crate left out by [`[[exclude]]`](../configuration.md#leaving-code-out) counts only through what the checked crates reach too.
