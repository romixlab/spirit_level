## Why

An adapter reads the output of an external tool: cargo-deny's JSON, typos' JSON, the text of cargo-machete. Each parser is tested against one range of the tool's versions, written in `crates/sl_tools/tools.toml`. Another version may print another shape: findings go missing or come out wrong, and nothing else says so.

spl finds each tool on PATH and reads its version. A tool outside its tested range, or one that prints no version, still runs; this lint reports it once, naming the lints whose results are untested. It is `deny` from the small tier, so the run fails, also with `--allow-warn-not-run`. A missing tool is not a finding of this lint: the lints that need it do not run, and the run says how to install it.

For example, with typos-cli 1.50.3 on PATH and the tested range `>=1.52, <1.53`:

```text
deny X0034 tool-version
  spirit_level.toml  typos 1.50.3 (/home/me/.cargo/bin/typos) is outside the tested range >=1.52, <1.53: the results of typos-misspelling are untested
    help: install typos-cli@1.52.2: `cargo binstall typos-cli@1.52.2` or `cargo install --locked typos-cli@1.52.2`
```

## How to fix

Install the version the help names, with whatever installs Rust tools here. `spl tools needed --spec --missing` prints every missing or untested crate as `crate@version`, one per line:

```sh
spl tools needed --spec --missing | xargs -r cargo binstall -y
```

`spl tools needed` shows each tool, its tested range and the version found.
