# The crate map

A big workspace declares its structure in a crate map: the layers, which layers each may depend on, the layer of every crate, and a few properties per crate. The graph lints check the real cargo graph against it, edge by edge.

The map lives in `spirit_level.toml` as `[map]`, or in its own file named by `map-file`.

```toml
[map.layers.utils]
desc = "Tiny helpers with no workspace dependencies outside utils"
allow = ["utils"]

[map.layers.model]
allow = ["utils", "model"]

[map.layers.app]
allow = ["utils", "model", "app"]

[map.crates]
foo_ids = { layer = "utils" }
foo_core = { layer = "model", deps = ["foo_ids"] }
foo_gpu = { layer = "app", unsafe = true }
foo_proto = { layer = "model", generated = true }
foo_api = { layer = "model", stable = true }
foo_math = { layer = "utils", no_std = true, deterministic = true }
foo_proto_types = { layer = "utils", contract = true }
foo_cli = { layer = "app", cli = true }
foo_ui = { layer = "app", ui = true }
foo_theme = { layer = "app", theme = true }

[map.forbid]
foo_cli = ["foo_gpu", "wgpu"]       # the CLI never reaches these, directly or through others

[map.external]
exact_units = { git = "https://github.com/romixlab/exact_units", branch = "main", role = "utils" }

[map.third-party]
wgpu = ["foo_gpu"]                  # only these crates may depend on wgpu directly
```

## Keys

| Key | Meaning |
|---|---|
| `layers.<name>.allow` | layers its crates may depend on; list its own name when crates of one layer may see each other |
| `layers.<name>.desc` | what lives in the layer |
| `crates.<name>.layer` | the crate's layer (required) |
| `crates.<name>.deps` | the workspace crates it may depend on; absent: any its layer allows |
| `crates.<name>.desc` | its description, when the map is where it is written |
| `unsafe` | may allow `unsafe` code, in a `[lints]` table of its own that copies the workspace lints; every other crate forbids it |
| `generated` | generated code, checked in: exempt from size limits |
| `stable` | a published, stable API: semver-checked |
| `no_std` | builds without std |
| `wasm` | builds for `wasm32-unknown-unknown` |
| `deterministic` | promises bit-identical results on every platform |
| `contract` | holds the serde types that cross a process or node boundary; sized integers only ([sized-ints-in-contracts](lints/C0017.md)) |
| `cli` | its binaries are command-line tools the [CLI checks](cli.md) build and run; `false` leaves them out. Absent: a crate with a binary target that depends on `clap` is one |
| `ui` | a user interface crate, held to the [UI practices](ui.md); absent: a crate that depends on a UI toolkit is one, `false` exempts such a crate |
| `theme` | the theme crate: colours and sizes are written as values here and nowhere else in UI code |
| `forbid.<crate>` | crates (workspace or third-party) its closure must never reach |
| `external.<crate>` | a crate of another project taken by git: its URL, its branch (optional) and the layer that may use it; how it is pinned is `git-dependency-pins`' policy |
| `third-party.<crate>` | workspace crates that alone may depend on a heavy third-party crate |

Unknown keys are errors, like everywhere in the configuration; the next section is the one way around that. Dev-dependencies are exempt from the layer rule but not from `forbid`. To fix an edge that points the wrong way, move a crate; do not reorder the layers.

## A map another tool owns

A workspace whose own generator or linter reads the map keeps keys spl does not know beside spl's: a coverage cell, a release, a licence, whole tables of its own. Name those keys in `foreign`, and spl leaves them to their owner:

```toml
[map-file]
path = "crates.toml"
foreign = ["meta", "domains", "deferred", "layers.*.licence", "crates.*.cell", "crates.*.area", "crates.*.when"]
```

- Each entry is a dotted key path; `*` stands for any one key, a crate's or a layer's name. A path that ends at a table hands the whole table over.
- spl reads its own keys as everywhere else. A key that is neither spl's nor listed is an error, so a typo in an spl key (`depz` for `deps`) still stops the run, with the nearest key spl knows.
- An entry that names a key spl reads (`crates.*.deps`) is an error: a key has one owner.
- The owner checks its keys; spl never looks at them. `map-file = "crates.toml"` is the same table with no `foreign` entries.
