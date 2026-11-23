<div class="sl-hero" markdown>
<img class="sl-hero--light" src="assets/logo/spirit_level-light.svg" alt="spirit_level">
<img class="sl-hero--dark" src="assets/logo/spirit_level-dark.svg" alt="spirit_level">
</div>

# spirit_level { .visually-hidden }

spirit_level is a linter for Rust projects of every size, from one crate to a workspace of hundreds. It checks what clippy does not: how a project is put together (manifests, workspace inheritance, the dependency graph, declared layers, crate sizes), how its code shares state, and the rules a team writes down and then forgets to check.

- **Tiers.** Lints come in TOML files shipped with the tool: [small](tiers/small.md) for one crate, [small-workspace](tiers/small-workspace.md), [medium](tiers/medium.md), [big](tiers/big.md). Each extends the one before it and is built from [sets](tiers/sets.md), small groups of lints a project can also take one at a time.
- **One file per project.** `spirit_level.toml` picks a tier (`extends = "big"`) and changes what it needs, every change with a reason, optionally with an expiry. Unknown keys and lint ids are errors.
- **Every lint documented.** Each [lint page](lints/index.md) is generated from the lint's own metadata: the rule, why, its parameters and its level in every tier.
- **For people and for CI.** Coloured text or a JSON report, exit codes, and `--fix` where a fix is exact.

```console
$ spl init --tier small-workspace
$ spl check
```
