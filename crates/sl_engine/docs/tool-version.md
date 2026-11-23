## Why

An adapter reads the output of an external tool: cargo-deny's JSON, typos' JSON, the text of cargo-machete. Each parser is tested against one range of the tool's versions, written in `crates/sl_tools/tools.toml`. Another version may print another shape: findings go missing or come out wrong, and nothing else says so.

spl finds each tool on PATH and reads its version. A tool outside its tested range, or one that prints no version, still runs; this lint reports it once, naming the lints whose results are untested. It is `deny` from the small tier, so the run fails, also with `--allow-warn-not-run`. A missing tool is not a finding of this lint: the lints that need it do not run, and the run says how to install it.

PATH is searched in order, and the first match wins: an older package (a distribution's, `pacman`, `apt`) ahead of a newer one (`~/.cargo/bin`) shadows it, and the tool keeps running the one spl never finds. When that happens, this lint says so instead of pointing at an install: a later entry on PATH already holds an in-range version, so installing again changes nothing; removing the earlier one or reordering PATH does.

For example, with typos-cli 1.50.3 on PATH and the tested range `>=1.52, <1.53`:

```text
deny X0034 tool-version
  spirit_level.toml  typos 1.50.3 (/home/me/.cargo/bin/typos) is outside the tested range >=1.52, <1.53: the results of typos-misspelling are untested
    help: install typos-cli@1.52.2: `cargo binstall typos-cli@1.52.2` or `cargo install --locked typos-cli@1.52.2`
```

With typos-cli 1.52.2 further down PATH, shadowed by an out-of-range one:

```text
deny X0034 tool-version
  spirit_level.toml  typos 1.50.3 (/usr/bin/typos) is outside the tested range >=1.52, <1.53; typos 1.52.2 (/home/me/.cargo/bin/typos) later on PATH is in range: the earlier one shadows it: the results of typos-misspelling are untested
    help: remove /usr/bin/typos from PATH, or move its directory after /home/me/.cargo/bin's: typos 1.52.2 there is already in range; `spl tools needed` lists every tool
```

## How to fix

Install the version the help names, with whatever installs Rust tools here. `spl tools needed --spec --missing` prints every missing or untested crate as `crate@version`, one per line:

```console
$ spl tools needed --spec --missing | xargs -r cargo binstall -y
```

When the help names a second path instead, a working version is already on PATH, further down than the one spl found: remove the earlier one, or move its directory after the one the help names.

`spl tools needed` shows each tool, its tested range, the version found and, when shadowed, the in-range one further down PATH.
