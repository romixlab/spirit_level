# Configuration

A project's configuration is `spirit_level.toml` at the workspace root. It extends tiers and changes lints, each change with a reason.

```toml
extends = "big"                      # a shipped tier, a list, or a path to a shared .toml file
map-file = "crates.toml"             # the crate map, or inline as [map]; a table when another tool owns part of it

[lints.crate-lines]                  # an override
level = "deny"                       # allow, warn or deny
max-lines = 15000                    # a parameter of the lint
reason = "this repo splits early"    # required
expires = "2027-06-30"               # optional: a date, or a release such as "v0.5.0"

[[crate-tiers]]                      # tiers that hold some crates beside the project's own
tiers = ["embedded"]
crates = ["foo_fw"]
reason = "the firmware; the host crates are not judged as firmware"

[[exceptions]]                       # an override for some crates or paths only
lint = "crate-lines"
crates = ["foo_sdk_types"]           # and/or paths = ["crates/foo/src/generated/**"]
max-lines = 120000
reason = "generated protocol types"
expires = "v0.4.0"

[clippy.lints]                       # the Clippy set expected in [workspace.lints.clippy]
print_stdout = { level = "allow", reason = "a CLI-only workspace" }

[clippy.config]                      # settings expected in clippy.toml
cognitive-complexity-threshold = { value = 20, reason = "this team splits early" }

[dependents]                         # where the repos that build on this library are checked out
scan = ["~/git"]

[docs]                               # the pages the docs lints read: Markdown, or a built site
reference = ["docs/lints/**"]        # pages that are looked up, not read through

[cli]                                # how the CLI lints build and run the tools
timeout-secs = 20
```

## Rules

- **extends** is a tier name (`spl tiers` lists them), a list (`["small", "embedded"]`, later ones win), or a path to a `.toml` file of the same form, relative to the file that names it.
- **Every entry needs a `reason`**, in tier files too. An entry without one is an error.
- **Levels:** `allow` (off), `warn` (reported), `deny` (reported, fails the run). A lint that no extended tier enables needs a `level` when a project first sets it.
- **Parameters** are the lint's own keys, listed on its page with their type and default. An unknown parameter or a value of the wrong type is an error.
- **Expiries:** a date (expired from that day on) or a release (expired once the project's version, from `[workspace.package]` or `[package]`, reaches it). An expired override or exception is an error: fix what it allowed and remove it, or renew it with a new reason and expiry.
- **Exceptions** apply to findings about the listed crates or whose file matches a path pattern (`*` within a segment, `**` across segments). They change the level, parameters, or both. Their parameters apply to the crates they name.
- **Unknown keys fail.** A typo in a key, a lint id or a tier name stops the run with the file and line, before any lint runs. Map keys another tool owns: [map-file foreign](map.md#a-map-another-tool-owns).
- **Codes or ids.** A lint is named by its id or its code: `[lints.S0001]` is `[lints.crate-lines]`, in exceptions (`lint = "S0001"`), waivers and `--only` too.

## Tiers for some crates

`extends` holds every crate. A workspace that mixes kinds of crates (firmware beside host tools, an egui app beside its CLI) puts the tiers of one kind on those crates only, so the others are not judged by rules that are not theirs:

```toml
extends = ["big", "ai"]

[[crate-tiers]]
tiers = ["embedded"]
crates = ["foo_fw", "foo_boot"]
reason = "the firmware crates"

[[crate-tiers]]
tiers = ["ui", "egui"]
crates = ["foo_app"]
reason = "the egui app"
```

With a crate map, the map key `tiers` says the same per crate: `foo_fw = { layer = "app", tiers = ["embedded"] }` ([The crate map](map.md)). Both may be used; a crate's tiers are the map's, then the entries', in order.

- For a finding about such a crate, levels and parameters are resolved as if the project file extended its tiers after its own `extends`. The project's `[lints]` and `[[exceptions]]` still apply on top.
- A finding about any other crate, or about no crate, keeps the project's level. A lint that only a crate's tiers enable runs, and its findings elsewhere are allowed.
- A tier is a shipped name or a path to a `.toml` file, as in `extends`. An entry needs `tiers`, `crates` and a `reason`; an unknown tier is an error.
- `spl config` lists each set of crates with the lints whose level differs for them.

## Tables beside the lints

- **The Clippy set** is what the project turns on in its own `[workspace.lints.clippy]` and `clippy.toml`: a lint or group (`pedantic`) at `warn` or `deny`, or at `allow` as an accepted opt-out that the help pastes with its reason. Tiers declare it; later layers override it entry by entry. [tier-clippy-set](lints/M0025.md) checks the project's files against it; spirit_level never passes Clippy flags of its own.
- **Dependents** turn on `spl dependents` for a library: [Dependents](dependents.md). Only a project file or a shared file sets them, never a tier.
- **Docs** say which Markdown files are pages, which of them are reference pages, and their language: [Docs as the reader meets them](reading.md#which-pages). Only a project file or a shared file sets them, never a tier.
- **The `[cli]` table** sets how the CLI lints build and probe the tools: [CLI checks](cli.md). Only a project file or a shared file sets it, never a tier.

## Seeing the result

`spl config` prints every lint that is on, the level, where it was set and the reason, then every exception with its scope, reason and expiry.
