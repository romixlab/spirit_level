# small-workspace

A small workspace: shared package keys, lints and dependencies declared once at the root and inherited.

Extends [small](small.md): everything below is on top of it.

Use it with `extends = "small-workspace"` in `spirit_level.toml`.

## Lints enabled (with what it extends)

| Code | Lint | Level | Reason | Set in |
|---|---|---|---|---|
| [C0002](../lints/C0002.md) | allow-without-reason | `warn` | an exception to a lint says why, or it outlives its reason | small |
| [C0006](../lints/C0006.md) | print-in-library | `warn` | printing is the binary's choice | small |
| [C0008](../lints/C0008.md) | todo-macros | `warn` | no stop-gaps: a todo!() is a panic waiting in shipped code | small |
| [C0012](../lints/C0012.md) | waiver-invalid | `deny` | a waiver without a reason, for an unknown lint, or expired hides a finding without saying why | small |
| [C0013](../lints/C0013.md) | waiver-unused | `warn` | a waiver that allows nothing is left over from changed code | small |
| [C0020](../lints/C0020.md) | cli-help-examples | `warn` | a reader learns a command from its examples first; most existing CLIs lack them on many leaves | small |
| [C0021](../lints/C0021.md) | cli-help-text | `warn` | -h is the map of a tool; an entry without a line leaves the reader guessing | small |
| [C0022](../lints/C0022.md) | cli-help-clean | `warn` | rustdoc source in help hides the tool's description | small |
| [C0023](../lints/C0023.md) | cli-completion | `warn` | completion makes a tool discoverable at the prompt; older CLIs add it when touched | small |
| [C0024](../lints/C0024.md) | cli-plain-when-piped | `warn` | colour escapes in a pipe break grep, diffs and scripts | small |
| [C0025](../lints/C0025.md) | cli-color-flag | `warn` | --color settles what automatic colour guesses wrong, the same way in every tool; older CLIs add it when touched | small |
| [C0026](../lints/C0026.md) | cli-usage-error | `warn` | a caller tells a wrong call from a failure by exit code 2 | small |
| [C0027](../lints/C0027.md) | cli-version-line | `warn` | the SHA and commit time say which build runs; older CLIs print other formats | small |
| [C0029](../lints/C0029.md) | cli-standard-flags | `warn` | short flags are habits across tools; a different meaning turns a habit into a mistake | small |
| [C0032](../lints/C0032.md) | float-partial-cmp-unwrap | `warn` | partial_cmp(..).unwrap() panics on the first NaN; total_cmp orders every float | small |
| [C0033](../lints/C0033.md) | exact-units-quantity | `warn` | a unit in a name is checked by no one; in the type the compiler checks every use. Names only, so a warning | small |
| [C0035](../lints/C0035.md) | exact-units-bounds | `warn` | a range the code enforces belongs in the type, where every value is checked when it is made | small |
| [D0001](../lints/D0001.md) | prose-run-words | `warn` | a long stretch of prose with no heading, figure, table or code block is skimmed, not read | small |
| [D0002](../lints/D0002.md) | page-reading-time | `warn` | a guide longer than one sitting is left half read | small |
| [D0003](../lints/D0003.md) | heading-words | `warn` | headings are scanned to find a section; a sentence is read slowly | small |
| [D0004](../lints/D0004.md) | page-headings | `warn` | a long page without sections gives the reader no map | small |
| [D0005](../lints/D0005.md) | code-wall | `warn` | code without prose leaves the reader to guess what it shows | small |
| [D0006](../lints/D0006.md) | image-alt | `warn` | an image without alternative text tells a screen reader, a search engine and a broken link nothing | small |
| [D0007](../lints/D0007.md) | code-block-language | `warn` | the language selects the highlighting and tells the reader what the block is | small |
| [D0008](../lints/D0008.md) | nav-entries | `deny` | a navigation entry that names no page renders as a link that leads nowhere, and the build does not fail | small |
| [D0009](../lints/D0009.md) | orphan-pages | `warn` | a page nobody reaches is published, unread and left to go stale | small |
| [D0010](../lints/D0010.md) | page-title | `warn` | a page without a title is shown under its file name | small |
| [D0011](../lints/D0011.md) | admonition-syntax | `deny` | an unknown type loses its colour and a body indented less than four spaces renders outside the box | small |
| [G0003](../lints/G0003.md) | crate-cycles | `warn` | crates in a cycle cannot be built or tested apart | small-workspace |
| [M0001](../lints/M0001.md) | default-features-off | `warn` | dependencies are added with default-features = false and only the features in use | small |
| [M0002](../lints/M0002.md) | duplicate-versions | `warn` | two versions of a crate double its build time and split its types | small-workspace |
| [M0003](../lints/M0003.md) | git-dependency-pins | `warn` | git dependencies follow a branch; Cargo.lock fixes the commit | small |
| [M0004](../lints/M0004.md) | no-sys-crates | `deny` | pure-Rust crates only: no bundled C, no -sys bindings without a written reason | small |
| [M0005](../lints/M0005.md) | path-outside-workspace | `deny` | a path into a sibling checkout builds differently on every PC | small |
| [M0006](../lints/M0006.md) | crate-docs | `warn` | rustdoc's front page of the crate is its crate-level docs | small |
| [M0007](../lints/M0007.md) | edition-latest | `warn` | new code starts on the current edition | small |
| [M0009](../lints/M0009.md) | package-description | `warn` | the crate's one-line description is written once, in Cargo.toml; tables and READMEs are made from it | small |
| [M0010](../lints/M0010.md) | package-license | `warn` | a crate without a licence cannot be reused or published | small |
| [M0012](../lints/M0012.md) | readme-per-crate | `deny` | every crate of a workspace says what it is and what it may depend on | small-workspace |
| [M0014](../lints/M0014.md) | changelog-unreleased | `warn` | work lands under [Unreleased]; the release step turns it into a version section | small |
| [M0015](../lints/M0015.md) | justfile-recipes | `warn` | every repo answers to the same recipes: default, test, lint, install, deploy | small |
| [M0017](../lints/M0017.md) | profile-defaults | `warn` | split debug info and an incremental deploy profile: measured large build-time wins | small |
| [M0018](../lints/M0018.md) | repo-files | `warn` | every work repo carries README, AGENTS, CLAUDE, FEATURES and CHANGELOG | small |
| [M0020](../lints/M0020.md) | crate-dir-name | `warn` | flat crates/<name>: the folder is the crate | small-workspace |
| [M0021](../lints/M0021.md) | macro-crate-suffix | `warn` | _macros and _derive mean proc macros and nothing else | small-workspace |
| [M0022](../lints/M0022.md) | workspace-dependencies | `warn` | one version per dependency per workspace, written once in [workspace.dependencies] | small-workspace |
| [M0023](../lints/M0023.md) | workspace-inheritance | `deny` | edition, rust-version, licence and repository are declared once in [workspace.package] | small-workspace |
| [M0024](../lints/M0024.md) | workspace-lints | `deny` | every crate inherits [workspace.lints], so one table is the policy | small-workspace |
| [M0025](../lints/M0025.md) | tier-clippy-set | `warn` | Clippy runs in the editor and CI from the project's own lint table; the tier says what belongs in it | small |
| [M0026](../lints/M0026.md) | profile-no-lto | `warn` | LTO in the loop's profiles turns a link of a second into minutes; it belongs in a release profile | small |
| [M0027](../lints/M0027.md) | patch-outside-workspace | `deny` | a committed patch into a sibling checkout builds differently on every machine; it lives in an ignored .cargo/config.toml | small |
| [P0001](../lints/P0001.md) | agent-file-lines | `warn` | agents read AGENTS.md whole every session; a long one is skimmed, not followed | small |
| [P0009](../lints/P0009.md) | changelog-entry-length | `warn` | a CHANGELOG is a list: one short line per change | small |
| [S0002](../lints/S0002.md) | function-complexity | `warn` | more paths than can be tested or held in one's head | small |
| [S0003](../lints/S0003.md) | function-lines | `warn` | a function longer than a screen is changed without being read whole | small |
| [T0001](../lints/T0001.md) | lock-held-across | `warn` | a guard across .await or I/O stalls every other caller for as long as the slowest peer | small |
| [T0003](../lints/T0003.md) | nested-locks | `warn` | two locks at once need one lock order everywhere | small-workspace |
| [T0004](../lints/T0004.md) | static-interior-mutability | `warn` | every piece of mutable state has one owner; a global lock says why | small |
| [T0005](../lints/T0005.md) | static-mut | `deny` | global mutable state with no synchronisation; there is always a better owner | small |
| [T0006](../lints/T0006.md) | thread-local-state | `warn` | thread-local state is a hidden side channel between functions | small-workspace |
| [X0005](../lints/X0005.md) | cargo-doc | `warn` | broken intra-doc links and tags render wrong on docs.rs and in the IDE | small |
| [X0019](../lints/X0019.md) | deny-advisories | `warn` | a dependency with a known vulnerability is a risk taken unseen; needs no deny.toml | small |
| [X0026](../lints/X0026.md) | machete-unused-deps | `warn` | an unused dependency costs build time, audit and lockfile churn for nothing | small |
| [X0031](../lints/X0031.md) | taplo-fmt | `warn` | manifests are edited by people and by tools; one TOML formatting keeps their diffs about the change | small-workspace |
| [X0032](../lints/X0032.md) | typos-misspelling | `warn` | a misspelled identifier is copied to every call site; typos has almost no false positives | small |

## Clippy set (with what it extends)

The project turns these on in its own `[workspace.lints.clippy]` (`allow` rows are accepted opt-outs, pasted with their reason); [tier-clippy-set](../lints/M0025.md) checks it.

| Clippy lint | Level | Reason | Set in |
|---|---|---|---|
| `as_ptr_cast_mut` | `warn` | `as_ptr() as *mut` writes through a shared borrow: undefined behaviour | small |
| `dbg_macro` | `warn` | debugging output left behind | small |
| `debug_assert_with_mut_call` | `warn` | a debug_assert! that calls a &mut method makes debug and release builds behave differently | small |
| `exit` | `warn` | process::exit skips every destructor; main returns instead | small |
| `fallible_impl_from` | `warn` | a From that can panic is a TryFrom | small |
| `get_unwrap` | `warn` | `.get(i).unwrap()` is indexing with a worse message: index, or handle the None | small |
| `ignore_without_reason` | `warn` | an ignored test says why, or nobody knows when it may run again | small |
| `let_underscore_must_use` | `warn` | `let _ =` on a must_use value drops an error unseen | small |
| `map_err_ignore` | `warn` | `map_err(\|_\| ...)` throws the cause away; wrap it as the source | small |
| `mem_forget` | `warn` | forget leaks the value and skips Drop; ManuallyDrop says it on purpose | small |
| `todo` | `warn` | no stop-gaps: a todo!() is a panic waiting in shipped code | small |
| `undocumented_unsafe_blocks` | `warn` | every unsafe block says in a SAFETY: comment why it is sound | small |
| `unimplemented` | `warn` | no stop-gaps: an unimplemented!() is a panic waiting in shipped code | small |
| `unnecessary_safety_doc` | `warn` | a # Safety section on a safe function sends readers looking for a contract that is not there | small |
| `unwrap_used` | `warn` | a panic says what failed (expect with a message), or the error is returned | small |

`clippy.toml` (an integer is a limit the project may lower):

| Setting | Value | Reason | Set in |
|---|---|---|---|
| `allow-unwrap-in-tests` | `true` | a test that unwraps fails at the unwrap, which is what a test should do | small |

## The file

```toml
# Tier 2: a few crates in one workspace. Everything shared is declared once at the root.
extends = "small"

[tier]
description = "A small workspace: shared package keys, lints and dependencies declared once at the root and inherited"

[lints.workspace-inheritance]
level = "deny"
reason = "edition, rust-version, licence and repository are declared once in [workspace.package]"

[lints.workspace-lints]
level = "deny"
reason = "every crate inherits [workspace.lints], so one table is the policy"

[lints.workspace-dependencies]
level = "warn"
reason = "one version per dependency per workspace, written once in [workspace.dependencies]"

[lints.duplicate-versions]
level = "warn"
reason = "two versions of a crate double its build time and split its types"

[lints.crate-dir-name]
level = "warn"
reason = "flat crates/<name>: the folder is the crate"

[lints.macro-crate-suffix]
level = "warn"
reason = "_macros and _derive mean proc macros and nothing else"

[lints.readme-per-crate]
level = "deny"
reason = "every crate of a workspace says what it is and what it may depend on"

[lints.nested-locks]
level = "warn"
reason = "two locks at once need one lock order everywhere"

[lints.thread-local-state]
level = "warn"
reason = "thread-local state is a hidden side channel between functions"

[lints.crate-cycles]
level = "warn"
reason = "crates in a cycle cannot be built or tested apart"

[lints.taplo-fmt]
level = "warn"
reason = "manifests are edited by people and by tools; one TOML formatting keeps their diffs about the change"
```
