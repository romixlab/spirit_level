# External tools

spirit_level reuses established Rust lint tools instead of reimplementing them. Each tool sits behind an adapter: an analysis like any other, whose lints carry the tool's prefix (`clippy-`, `deny-`, `machete-`, ...) and are configured, overridden, excepted and waived in `spirit_level.toml` like native ones.

## How tools are run

- **Library or executable.** A tool with a library on stable Rust that does the whole job is linked in, always compiled; it runs only when one of its lints is on. The others run as executables, and the adapter reads their machine-readable output (JSON where the tool has it, its stable text where not).
- **Found on PATH.** spl installs nothing. Each tool is found on PATH and its version read, so it comes from whatever installs tools here: `cargo binstall`, `cargo install`, a distribution's package, mise.
- **Missing tools.** A lint whose executable is not installed is reported as not run with the reason and the `crate@version` that installs it. The run then exits with code 3, at `warn` as at `deny` ([exit codes](output.md#exit-codes)).
- **Tested versions.** `crates/sl_tools/tools.toml` gives each tool the range of versions its parser is tested against. A tool outside that range still runs; `tool-version` (deny from the small tier) reports its results as untested and fails the run.
- **What a project needs.** `spl tools needed` lists what the project's enabled lints need: their tools, the toolchain of each `rust-version` for `cargo-msrv`, and the targets of `cargo-no-std` and `cargo-wasm` when the map marks crates. Install them before `check` in CI, so no lint is left not run.

## Library or executable

Cost of linking: a scratch project depending only on the library (`default-features = false`), built cold with `cargo build --release -j 4`. Crates are the unique packages in `cargo tree -e normal,build`. C is any `-sys` crate with a native build, `cc`, `cmake`, `ring` or `aws-lc-sys` in that tree. For scale, spirit_level's own base (cargo_metadata, toml_edit, syn, serde_json) is 26 crates and 12 s.

| Tool | Used as | Why | Cost of linking (crates, s, C) |
|---|---|---|---|
| clippy | executable | its lints are rustc plugins: `rustc_private`, nightly only | not linkable on stable |
| rustfmt | executable | `rustfmt_nightly` is `rustc_private` too | not linkable on stable |
| cargo-deny | executable | the library pulls C into every build, for checks that run once per CI job | 143, 63 s, ring, zstd-sys |
| cargo-audit (rustsec) | executable | fetching the advisory database needs the default features, which build C | 249, 97 s, aws-lc-sys, cmake; without fetch 54, 11 s |
| cargo-machete | executable | binary only, no library target | none |
| cargo-shear | executable | its library does not build with a fresh lock (ra-ap-rustc_lexer) | 120 crates, fails to build |
| typos | executable | the `typos` library is only the tokenizer; config, file walk and ignore rules live in typos-cli | typos + dict 22, 14 s; typos-cli 129, 37 s |
| taplo | executable | the config and file-walk layer (taplo-common) needs a tokio runtime | taplo 53, 16 s; taplo-common 158, 41 s |
| cargo-semver-checks | executable | heavy, builds C, and its library report has no per-check detail | 308, 115 s, ring |
| cargo-udeps | executable | needs nightly, and links cargo itself (libgit2, curl) | not linkable on stable |
| hakari | library | guppy and hakari are libraries first; no C | 86, 20 s, none |
| geiger | library | a small library that counts `unsafe` with syn | 41, 11 s, none |

## Lint mapping

A tool's findings map to spirit_level lints in one of two ways, chosen so that a level, an exception or an expiry in `spirit_level.toml` means something:

- **One lint per family.** When the tool sorts its findings into a few groups a project sets apart (clippy's groups, cargo-deny's checks, cargo-audit's kinds), each group is a lint.
- **One lint per tool.** When every finding is the same kind of problem (an unused dependency, a misspelling, an unformatted file), the tool is one lint.

In both cases the item (the fingerprint waivers and baselines use) carries the tool's own code: `clippy::needless_range_loop@src/lib.rs:for i in 0..n`, `licenses:unlicensed:foo@1.0.0`. Which of a tool's own checks run stays the tool's configuration (`[lints.clippy]`, `deny.toml`, `_typos.toml`, `taplo.toml`); spirit_level decides how much a finding weighs.

## clippy

- **Command:** `cargo clippy --message-format json` plus the `args` parameter of `clippy-build` (default `--workspace --all-targets`). The groups come from `clippy-driver -W help`.
- **Output:** cargo's JSON messages; `compiler-message` objects carry the rustc diagnostic with its `code` (`clippy::lint_name`) and spans.
- **Exit codes:** 0 built (warnings included), 101 a build error. Findings come from the messages, not the code.
- **Lints:** `clippy-correctness`, `-suspicious`, `-style`, `-complexity`, `-perf`, `-pedantic`, `-nursery`, `-cargo`, `-restriction` by group; `clippy-build` for rustc's own warnings and errors. No `-W` flags are passed, so the project's `[lints.clippy]` decides what runs.

## rustfmt

- **Command:** `cargo fmt --all --check --message-format short`.
- **Output:** one absolute path per line for each file rustfmt would change; parse errors on stderr.
- **Exit codes:** 0 formatted, 1 a file to change or a parse error.
- **Lints:** `rustfmt-unformatted`, one finding per file.

## cargo-deny

- **Command:** `cargo deny --format json --color never check <check>`, once per enabled check; `--offline` when the `fetch` parameter is false.
- **Output:** JSON lines on stderr. `"type": "diagnostic"` objects carry `code`, `severity`, `message`, `labels` and `graphs` (the crate and the chain that pulls it in); `"type": "log"` objects are cargo-deny's log.
- **Exit codes:** a bit set of the failed checks: 1 advisories, 2 bans, 4 licenses, 8 sources. A failure without diagnostics is a failed run (`run:cargo-deny`).
- **Lints:** `deny-advisories`, `deny-bans`, `deny-licenses`, `deny-sources`. `deny-advisories` works without a `deny.toml`; the others need the project's lists in it.

## cargo-audit

- **Command:** `cargo audit --json`, with `--no-fetch` when the `fetch` parameter is false.
- **Output:** one JSON document: `vulnerabilities.list` (each with `advisory`, `versions`, `package`) and `warnings`, a map from kind (`unmaintained`, `unsound`, `yanked`) to the same shape.
- **Exit codes:** 0 no vulnerability, 1 a vulnerability; errors on stderr.
- **Lints:** `audit-vulnerabilities`, `audit-unmaintained`, `audit-unsound`, `audit-yanked`. It overlaps `deny-advisories`; a project uses one of them.

## cargo-machete

- **Command:** `cargo machete --skip-target-dir .`
- **Output:** per crate with unused dependencies a line `<crate> -- <path>/Cargo.toml:`, then one tab-indented line per dependency.
- **Exit codes:** 0 none, 1 found some, 2 an error.
- **Lints:** `machete-unused-deps`, one finding per crate and dependency. A false positive goes into the crate's `[package.metadata.cargo-machete] ignored`.

## cargo-shear

- **Command:** `cargo shear --format json --color never`.
- **Output:** one JSON document: `summary` and `findings`, each with a `code` (`shear/unused_dependency`, `shear/misplaced_dependency`, ...), `severity`, `message`, `file` and a byte `location`.
- **Exit codes:** 0 none, 1 found some, 2 an error.
- **Lints:** `shear-unused-deps` (also unused workspace dependencies), `shear-misplaced-deps` (a dependency only tests use). An alternative to cargo-machete that also sees misplaced dependencies.

## typos

- **Command:** `typos --format json` in the project root.
- **Output:** JSON lines; `"type": "typo"` objects carry `path`, `line_num`, `byte_offset` in the line, `typo` and `corrections`. It reads `typos.toml` (`_typos.toml`, `.typos.toml`) and `.gitignore`.
- **Exit codes:** 0 clean, 2 typos found, others an error.
- **Lints:** `typos-misspelling`, one finding per place; an exact fix when typos offers one correction.

## taplo

- **Command:** `taplo fmt --check --colors never` on the TOML files `git ls-files` lists (taplo's own walk enters ignored directories). The fix is `taplo fmt --stdin-filepath <file> -`, so `taplo.toml`'s per-path rules apply.
- **Output:** log lines; an unformatted file is `ERROR taplo:format_files: the file is not properly formatted path="<path>"`, a parse error an `error: invalid TOML` report with a `┌─ <path>:<line>:<column>` line.
- **Exit codes:** 0 clean, 1 a file not formatted or not parsing.
- **Lints:** `taplo-fmt`, one finding per file, with an exact fix.

## cargo-semver-checks

- **Command:** `cargo semver-checks -p <crate> --color never` for each crate the map marks `stable = true` (or the `packages` parameter); `--baseline-rev` or `--baseline-root` from the parameters, else the latest release on crates.io.
- **Output:** text only. Each failed check is a line `--- failure <check>: <what> ---`, its places follow under `Failed in:` (`<item> in <file>:<line>`), and a `Summary` line names the bump the changes need.
- **Exit codes:** 0 the version bump covers every change, 1 otherwise.
- **Lints:** `semver-checks-breaking`, one finding per check and place; the item carries the check's name.

## cargo-udeps

- **Command:** `cargo +<pinned nightly> udeps --output json --workspace --all-targets`; the nightly is `toolchain-pinned`'s `nightly` parameter.
- **Output:** one JSON document: `success` and `unused_deps`, a map from package id to its `manifest_path` and unused `normal`, `development` and `build` dependencies.
- **Exit codes:** 0 none, 1 found some or a failed build (stderr).
- **Lints:** `udeps-unused-deps`. It needs the pinned nightly installed; without it the lint is not run.

## hakari (linked)

- **Library:** guppy reads the full `cargo metadata`; hakari computes the workspace-hack from `.config/hakari.toml` (or `.guppy/hakari.toml`), as `cargo hakari` does.
- **Equivalent commands:** `cargo hakari generate --diff` and `cargo hakari manage-deps --dry-run`.
- **Lints:** `hakari-generate` (the generated section of the workspace-hack's `Cargo.toml` is current; exact fix writes it) and `hakari-manage-deps` (every managed member depends on the workspace-hack, and no other crate does). Always linked in.

## geiger (linked)

- **Library:** every package the members reach through normal and build edges is scanned, each `.rs` file outside tests parsed with syn. A package whose library root says `#![forbid(unsafe_code)]` and counts nothing is skipped.
- **Lints:** `geiger-unsafe-deps`, one finding per dependency with more unsafe uses than `max-uses` that is not in `allow`. Always linked in.

## Cargo checks

Builds that cargo does itself, reported from `--message-format json`. Other toolchains and targets build into `target/spirit-level/<check>/`, so they never invalidate the normal build. A toolchain or target that is not installed makes the lint not run, with the `rustup` command that adds it; rustup is never allowed to install one by itself.

- **`cargo-msrv`:** `cargo check` with the toolchain of each crate's `rust-version`, per version.
- **`cargo-no-std`:** `cargo check -p <crate> --target <t> --no-default-features` for each crate the map marks `no_std = true`, on each of the `targets` (default `thumbv7em-none-eabihf`); `args` replaces `--no-default-features`.
- **`cargo-wasm`:** `cargo check -p <crate> --target wasm32-unknown-unknown` for each crate the map marks `wasm = true`.
- **`cargo-doc`:** `cargo doc`; every rustdoc warning is a finding, as `RUSTDOCFLAGS=-D warnings` would fail.

## Installing the tools

`spl tools needed` shows each tool the project needs: its tested range, the version found and where, whether it is in range, whether it runs on nightly, a later PATH entry that already has it in range, and the lints that run it. It exits with 1 while something needed is missing or outside its tested range. Its other outputs feed an installer:

| Flag | Prints | For |
|---|---|---|
| `--spec` | one `crate@version` per line | `xargs -r cargo binstall -y`, `xargs -r -n1 cargo install --locked` |
| `--components` | one rustup component per line | `xargs -r rustup component add` |
| `--toolchains` | one toolchain per line (`cargo-msrv`) | `xargs -r -n1 rustup toolchain install --profile minimal` |
| `--targets` | one target per line (`cargo-no-std`, `cargo-wasm`) | `xargs -r rustup target add` |
| `--json` | every record: crate, version, tested range, path, version found, state, shadow, lints | scripts and packagers |

`--missing` keeps only what is missing, outside its tested range or prints no version. `--all` lists every tool that has an adapter, without reading a project. For example, in CI:

```console
$ spl tools needed --spec --missing | xargs -r cargo binstall -y
$ spl tools needed --components --missing | xargs -r rustup component add
$ spl check
```

The version in `--spec` is the one the parser is tested at. A tool's version is the first word shaped like one in the output of its `version-args` (default `--version`); `cargo-udeps` runs on the pinned nightly, which `rustup toolchain install nightly-2026-11-19 --profile minimal` adds (`spl tools needed` names the one a project pins).

### A tool shadowed on PATH

PATH is searched in order, and the first match wins. An earlier directory holding an out-of-range copy (a distribution package ahead of `~/.cargo/bin`) hides an in-range one further down: `--spec` and `--components` never list such a tool, since installing it again changes nothing. The table's `shadowed by` column, the JSON `shadow` field and the `tool-version` finding's help all name the later path and version instead, and say to remove the earlier one from PATH or move its directory after the later one's.
