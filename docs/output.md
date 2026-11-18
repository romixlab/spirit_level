# Output and exit codes

## Text

For people, findings are grouped by lint, denied groups first, then warnings, each by count:

<!-- spirit-level:check assets/sample_workspace --only workspace-lints -->
<pre class="sl-term"><code><span class="sl-term-c1 sl-term-b">deny</span> <span class="sl-term-c6">M0024</span> workspace-lints <span class="sl-term-d">×7</span>: the root declares [workspace.lints] and every member inherits them
  <span class="sl-term-c5 sl-term-b">help:</span> add `[lints]` with `workspace = true` to its Cargo.toml
  crates/station/Cargo.toml  station does not inherit the workspace lints
  crates/station_cli/Cargo.toml  station_cli does not inherit the workspace lints
  crates/station_core/Cargo.toml  station_core does not inherit the workspace lints
  crates/station_io/Cargo.toml  station_io does not inherit the workspace lints
  crates/station_net/Cargo.toml  station_net does not inherit the workspace lints
  <span class="sl-term-d">fix (--fix): add [lints] workspace = true (7 of 7)</span>
  <span class="sl-term-d">+2 more: `spl check --all` lists them (explain: `spl explain M0024`)</span>

<span class="sl-term-c1 sl-term-b">failed</span>: 1 lints ran on assets/sample_workspace (small-workspace): <span class="sl-term-c1 sl-term-b">7 denied</span></code></pre>
<!-- /spirit-level:check -->

- **Header:** level, code, id, count and the lint's rule.
- **Excerpt:** the source of the first place that points at a line, with 2 lines of context and its exact range marked with the finding's message. `--excerpts N` shows N per group; `0` turns them off.
- **Places:** up to 5, then how many more and the command that lists them. `--all` lists every place. An exact fix is named once per group, with how many places it fixes.
- **`--group-by crate|file`** groups by crate or file instead of lint.
- **Summary** last: the result, how many lints ran, and the counts.

Colour is off when the output is not a terminal, with `NO_COLOR`, or with `--color never`. `--show-allowed` also lists findings allowed by waivers and exceptions; `--timings` shows the time of each analysis.

## Progress

While `spl check` runs, a line on stderr says what runs now and how far the run is: the analyses done of those that run, then one line under it per analysis running, with the tools it starts and its time. The tool adapters (Clippy, cargo-deny, cargo-machete, typos and the others) take most of a run, so their lines stay longest:

```text
⠙ spl check: [================>       ] 14/20 analyses 8s
  ⠂ cargo-deny 1s
```

Before the analyses it names the step (loading the project, finding the tools), after them the waivers. When the run ends, spl erases the lines, and the report follows as it would without them: stdout never carries progress. `spl check --fix` shows the run after the fixes the same way; `spl pages` and `spl dependents` name their steps.

`--progress` decides when, for every command:

| Value | Progress |
|---|---|
| `auto` (default) | drawn when stderr is a terminal, except under `--format json`, in CI (`CI` set), with `NO_COLOR` set or with `TERM=dumb` |
| `always` | drawn at a terminal; elsewhere one line per step, `spl check: [3/20] clippy (cargo-clippy)` |
| `never` | none |

## JSON

`spl check --format json` prints one document:

```json
{
  "schema": "spirit_level.report/1",
  "root": "/path/to/project",
  "extends": ["big"],
  "diagnostics": [
    {
      "lint": "crate-lines",
      "code": "S0001",
      "level": "warn",
      "confidence": "certain",
      "message": "foo has 23412 code lines in src/",
      "help": "split it: ...",
      "spans": [{ "file": "crates/foo/src/lib.rs", "line": 0, "column": 0 }],
      "item": "crate:foo",
      "krate": "foo",
      "metric": { "name": "lines", "value": 23412.0, "limit": 20000.0 }
    }
  ],
  "ran": ["crate-lines", "..."],
  "not_run": [{ "lint": "deny-licenses", "code": "X0021", "level": "deny", "need": "tool:cargo-deny", "reason": "..." }],
  "skipped": [{ "lint": "stk-3", "code": "F0093", "level": "deny", "need": "elf", "reason": "no firmware binary" }],
  "timings": [{ "analysis": "size", "ms": 41 }]
}
```

JSON is complete and ungrouped: every finding, whatever `--all` and `--group-by` say. A span may carry `end_line` and `end_column` (the column after its last character); the text view reads the file for its excerpt, JSON carries no source. `rule` names an adapter's tool rule (`"clippy:needless_return"`). `allowed` (the reason) and `fix` (the edits) appear on a finding when they apply. Configuration errors print `{"schema": "spirit_level.config-errors/1", "errors": [{"file", "line", "message"}]}`.

## Confidence

`certain` findings rest only on the project's files. `estimate` findings rest on something read without type information; the message says what was assumed. `candidate` findings are places to look at and never fail a run.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | no denied finding (warnings allowed) |
| 1 | at least one denied finding |
| 2 | a configuration or usage error, or a project cargo cannot read |
| 3 | no denied finding, but an enabled lint could not run (a tool, toolchain, target, the crate map or the resolved graph is missing) |

A lint that cannot run fails the run at `warn` as well as at `deny`: a check that silently skips part of its work never passes green. `spl check --allow-warn-not-run` lets warn-level lints that cannot run through: they are listed under "not run" and in the summary, and the exit code follows the findings. A deny-level lint that cannot run still exits with 3. There is no configuration key for this; a project that never wants a tool turns its lints off.

## Skipped lints

A lint is skipped when the project has nothing of the kind it checks. A skipped lint never fails the run and is not "not run": the text report gives one line per reason, `skipped: no firmware binary: dma-1, err-3, mem-3, stk-3`, the summary counts them, and JSON lists them under `skipped`.

The lints that read the linked ELF (`--elf`) apply to the firmware binaries: the [firmware crates](embedded-migration.md#firmware-crates) that build a binary. They are skipped when the workspace has none, or when every one allows the lint, by its own [tiers](configuration.md#tiers-for-some-crates) and exceptions (`no firmware binary it is enabled on`). With a firmware binary the lint is enabled on, a run without its ELF fails with exit code 3, as any lint that cannot run.
