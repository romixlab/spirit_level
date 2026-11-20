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
| [C0037](../lints/C0037.md) | cli-secret-flags | `warn` | a candidate: a flag's name is a guess at its purpose, so a human reads each hit; the shell history and the process list both see a value passed on the command line | small |
| [C0038](../lints/C0038.md) | cli-no-abbreviations | `warn` | a caller's shorthand for one subcommand can silently point at a different one once a sibling is added | small |
| [C0039](../lints/C0039.md) | command-registry | `warn` | an estimate: past a handful of commands, one central list is the line every new command edits and every parallel branch conflicts on | small |
| [C0040](../lints/C0040.md) | build-script-clock | `warn` | a clock read makes the build unreproducible; a hardcoded .git path breaks in a worktree checkout | small |
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
| [D0012](../lints/D0012.md) | abbreviation-first-use | `warn` | a reader arrives on any page and cannot look up an abbreviation the page never spells out | small |
| [D0013](../lints/D0013.md) | term-before-definition | `warn` | a term used before its definition is guessed at, and a wrong guess is read into every sentence until then | small |
| [D0014](../lints/D0014.md) | term-synonyms | `warn` | a reader takes two names for two things and looks for a difference that is not there | small |
| [D0015](../lints/D0015.md) | passive-voice | `warn` | a passive sentence hides who acts; a page of them reads as things that happen on their own | small |
| [D0016](../lints/D0016.md) | reading-ease | `warn` | long sentences of long words are read slowly and understood less | small |
| [D0017](../lints/D0017.md) | book-build-stamp | `warn` | without a version, commit and age in the footer, a reader cannot tell which build they are looking at | small |
| [D0018](../lints/D0018.md) | readme-index-clash | `deny` | Zensical does not say which of README.md and index.md it builds when a folder holds both | small |
| [D0019](../lints/D0019.md) | theme-feature-conflicts | `deny` | Zensical does not support the conflicting features together, and builds neither error nor warn about it | small |
| [D0020](../lints/D0020.md) | zensical-keys | `deny` | Zensical ignores a key it does not know, silently; a typo in zensical.toml never fails the build | small |
| [D0021](../lints/D0021.md) | page-links | `deny` | a link to a page that moved or never existed, or to the built site, leads a reader nowhere | small |
| [D0022](../lints/D0022.md) | book-deploy | `warn` | a book published by hand falls behind main, and a broken page goes live; one workflow builds it strict and deploys it | small |
| [D0023](../lints/D0023.md) | page-figures | `warn` | a guide that explains a mechanism in words alone is skimmed; a figure shows it whole | small |
| [D0024](../lints/D0024.md) | paragraph-run-words | `warn` | short paragraphs under many headings are still a wall of text when nothing else sits between them | small |
| [D0025](../lints/D0025.md) | prose-per-element | `warn` | a page of sentences with little else between them is skimmed; the reader needs something to look at | small |
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
| [M0019](../lints/M0019.md) | toolchain-pinned | `warn` | every PC and the gate build with the same compiler: one exact release or dated nightly, the pair spl ships | small |
| [M0020](../lints/M0020.md) | crate-dir-name | `warn` | flat crates/<name>: the folder is the crate | small-workspace |
| [M0021](../lints/M0021.md) | macro-crate-suffix | `warn` | _macros and _derive mean proc macros and nothing else | small-workspace |
| [M0022](../lints/M0022.md) | workspace-dependencies | `warn` | one version per dependency per workspace, written once in [workspace.dependencies] | small-workspace |
| [M0023](../lints/M0023.md) | workspace-inheritance | `deny` | edition, rust-version, licence and repository are declared once in [workspace.package] | small-workspace |
| [M0024](../lints/M0024.md) | workspace-lints | `deny` | every crate inherits [workspace.lints], so one table is the policy | small-workspace |
| [M0025](../lints/M0025.md) | tier-clippy-set | `warn` | Clippy runs in the editor and CI from the project's own lint table; the tier says what belongs in it | small |
| [M0026](../lints/M0026.md) | profile-no-lto | `warn` | LTO in the loop's profiles turns a link of a second into minutes; it belongs in a release profile | small |
| [M0027](../lints/M0027.md) | patch-outside-workspace | `deny` | a committed patch into a sibling checkout builds differently on every machine; it lives in an ignored .cargo/config.toml | small |
| [M0032](../lints/M0032.md) | ci-workflow | `warn` | checks that only run on one PC get skipped; CI runs the repo's own checks on every push and pull request | small-workspace |
| [M0033](../lints/M0033.md) | manifest-homepage | `warn` | crates.io and docs.rs link to repository and homepage; when there is a book, it is the homepage a reader wants | small |
| [M0034](../lints/M0034.md) | readme-badges | `warn` | the top of the README tells a visitor whether the project passes and where its book, crate and API docs are | small |
| [M0036](../lints/M0036.md) | toolchain-overrides-pinned | `warn` | a `cargo +nightly` in a recipe or CI bypasses the pinned toolchain | small |
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
| [X0034](../lints/X0034.md) | tool-version | `deny` | an external tool outside its tested range may print a shape the adapter misreads: its results cannot be trusted | small |

## Clippy set (with what it extends)

The project turns these on in its own `[workspace.lints.clippy]` (`allow` rows are accepted opt-outs, pasted with their reason); [tier-clippy-set](../lints/M0025.md) checks it.

| Clippy lint | Level | Reason | Set in |
|---|---|---|---|
| [`as_ptr_cast_mut`](../clippy/as_ptr_cast_mut.md) | `warn` | `as_ptr() as *mut` writes through a shared borrow: undefined behaviour | small |
| [`dbg_macro`](../clippy/dbg_macro.md) | `warn` | debugging output left behind | small |
| [`debug_assert_with_mut_call`](../clippy/debug_assert_with_mut_call.md) | `warn` | a debug_assert! that calls a &mut method makes debug and release builds behave differently | small |
| [`exit`](../clippy/exit.md) | `warn` | process::exit skips every destructor; main returns instead | small |
| [`fallible_impl_from`](../clippy/fallible_impl_from.md) | `warn` | a From that can panic is a TryFrom | small |
| [`get_unwrap`](../clippy/get_unwrap.md) | `warn` | `.get(i).unwrap()` is indexing with a worse message: index, or handle the None | small |
| [`ignore_without_reason`](../clippy/ignore_without_reason.md) | `warn` | an ignored test says why, or nobody knows when it may run again | small |
| [`let_underscore_must_use`](../clippy/let_underscore_must_use.md) | `warn` | `let _ =` on a must_use value drops an error unseen | small |
| [`map_err_ignore`](../clippy/map_err_ignore.md) | `warn` | `map_err(\|_\| ...)` throws the cause away; wrap it as the source | small |
| [`mem_forget`](../clippy/mem_forget.md) | `warn` | forget leaks the value and skips Drop; ManuallyDrop says it on purpose | small |
| [`todo`](../clippy/todo.md) | `warn` | no stop-gaps: a todo!() is a panic waiting in shipped code | small |
| [`undocumented_unsafe_blocks`](../clippy/undocumented_unsafe_blocks.md) | `warn` | every unsafe block says in a SAFETY: comment why it is sound | small |
| [`unimplemented`](../clippy/unimplemented.md) | `warn` | no stop-gaps: an unimplemented!() is a panic waiting in shipped code | small |
| [`unnecessary_safety_doc`](../clippy/unnecessary_safety_doc.md) | `warn` | a # Safety section on a safe function sends readers looking for a contract that is not there | small |
| [`unwrap_used`](../clippy/unwrap_used.md) | `warn` | a panic says what failed (expect with a message), or the error is returned | small |

`clippy.toml` (an integer is a limit the project may lower):

| Setting | Value | Reason | Set in |
|---|---|---|---|
| [`allow-unwrap-in-tests`](../clippy/allow-unwrap-in-tests.md) | `true` | a test that unwraps fails at the unwrap, which is what a test should do | small |

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

[lints.ci-workflow]
level = "warn"
reason = "checks that only run on one PC get skipped; CI runs the repo's own checks on every push and pull request"
```
