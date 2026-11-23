# Getting started

## Install

```bash
cargo install --locked --git https://github.com/romixlab/spirit_level spirit_level_cli
```

It installs the `spl` command. Shell completion: `source <(COMPLETE=bash spl)` in `~/.bashrc` (zsh and fish alike).

## Pick a tier

| Project | Tier |
|---|---|
| one crate | `small` |
| a few crates in a workspace | `small-workspace` |
| tens of crates, internal crates, an MSRV | `medium` |
| a big workspace with declared layers | `big` (needs a [crate map](map.md)) |

```bash
spl init --tier small-workspace   # writes spirit_level.toml
spl check                         # runs it
spl tiers                         # the ladder and add-ons, this project's marked
spl check --next                  # a look at the next tier up, with this project's overrides
spl check --at big                # the same at any ladder tier; nothing is written
spl check --tier big              # the shipped tier alone, without the project's file
```

`--next` and `--at` start with one line per lint that has findings: how many, its level there, and its level at the project's own tier (`off, new` or `warn, raised`). The findings follow.

## Read a finding

<!-- spirit-level:check assets/sample --only panic-in-library -->
<pre class="sl-term"><code><span class="sl-term-c1 sl-term-b">deny</span> <span class="sl-term-c6">C0005</span> panic-in-library <span class="sl-term-d">×2</span>: library code returns errors for what can fail and says why where it panics
    <span class="sl-term-c12 sl-term-b">--&gt; </span>src/lib.rs:13:58
     <span class="sl-term-c12 sl-term-b">|</span>
  <span class="sl-term-c12 sl-term-b">11</span> <span class="sl-term-c12 sl-term-b">|</span> /// Parses a frame of three bytes
  <span class="sl-term-c12 sl-term-b">12</span> <span class="sl-term-c12 sl-term-b">|</span> pub fn parse(bytes: &amp;[u8]) -&gt; Frame {
  <span class="sl-term-c12 sl-term-b">13</span> <span class="sl-term-c12 sl-term-b">|</span>     let value = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
     <span class="sl-term-c12 sl-term-b">|</span>                                                          <span class="sl-term-c9 sl-term-b">^^^^^^^^^</span> <span class="sl-term-c9 sl-term-b">.unwrap() in library code (2 in telemetry)</span>
  <span class="sl-term-c12 sl-term-b">14</span> <span class="sl-term-c12 sl-term-b">|</span>     Frame { channel: bytes[0], value }
  <span class="sl-term-c12 sl-term-b">15</span> <span class="sl-term-c12 sl-term-b">|</span> }
     <span class="sl-term-c12 sl-term-b">|</span>
  <span class="sl-term-c5 sl-term-b">help:</span> return an error for what can fail; where it cannot, say why: `.expect("why")`, `unreachable!("why")`, or a waiver
  src/lib.rs:13:58  .unwrap() in library code (2 in telemetry) <span class="sl-term-d">(panics 2, limit 0)</span>
  src/lib.rs:19:5  todo! in library code (2 in telemetry) <span class="sl-term-d">(panics 2, limit 0)</span>

<span class="sl-term-c1 sl-term-b">failed</span>: 1 lints ran on assets/sample (medium): <span class="sl-term-c1 sl-term-b">2 denied</span></code></pre>
<!-- /spirit-level:check -->

`spl explain panic-in-library` prints the lint's page. `spl check --fix` applies the exact fixes and checks again.

## Change what does not fit

Every change says why ([configuration](configuration.md)):

```toml
extends = "small-workspace"

[lints.duplicate-versions]
level = "allow"
reason = "syn 2 and 3 until clap moves; checked monthly"
expires = "2027-01-31"
```

A single finding is allowed where it is, with a [waiver](waivers.md).
