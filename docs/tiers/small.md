# small

One crate: metadata, docs, pure-Rust dependencies, build profiles, the repo's rule files, valid waivers.

Use it with `extends = "small"` in `spirit_level.toml`.

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
| [M0001](../lints/M0001.md) | default-features-off | `warn` | dependencies are added with default-features = false and only the features in use | small |
| [M0003](../lints/M0003.md) | git-dependency-pins | `warn` | git dependencies follow a branch; Cargo.lock fixes the commit | small |
| [M0004](../lints/M0004.md) | no-sys-crates | `deny` | pure-Rust crates only: no bundled C, no -sys bindings without a written reason | small |
| [M0005](../lints/M0005.md) | path-outside-workspace | `deny` | a path into a sibling checkout builds differently on every PC | small |
| [M0006](../lints/M0006.md) | crate-docs | `warn` | rustdoc's front page of the crate is its crate-level docs | small |
| [M0007](../lints/M0007.md) | edition-latest | `warn` | new code starts on the current edition | small |
| [M0009](../lints/M0009.md) | package-description | `warn` | the crate's one-line description is written once, in Cargo.toml; tables and READMEs are made from it | small |
| [M0010](../lints/M0010.md) | package-license | `warn` | a crate without a licence cannot be reused or published | small |
| [M0012](../lints/M0012.md) | readme-per-crate | `warn` | every crate says what it is and how to use it | small |
| [M0014](../lints/M0014.md) | changelog-unreleased | `warn` | work lands under [Unreleased]; the release step turns it into a version section | small |
| [M0015](../lints/M0015.md) | justfile-recipes | `warn` | every repo answers to the same recipes: default, test, lint, install, deploy | small |
| [M0017](../lints/M0017.md) | profile-defaults | `warn` | split debug info and an incremental deploy profile: measured large build-time wins | small |
| [M0018](../lints/M0018.md) | repo-files | `warn` | every work repo carries README, AGENTS, CLAUDE, FEATURES and CHANGELOG | small |
| [M0025](../lints/M0025.md) | tier-clippy-set | `warn` | Clippy runs in the editor and CI from the project's own lint table; the tier says what belongs in it | small |
| [M0026](../lints/M0026.md) | profile-no-lto | `warn` | LTO in the loop's profiles turns a link of a second into minutes; it belongs in a release profile | small |
| [M0027](../lints/M0027.md) | patch-outside-workspace | `deny` | a committed patch into a sibling checkout builds differently on every machine; it lives in an ignored .cargo/config.toml | small |
| [M0033](../lints/M0033.md) | manifest-homepage | `warn` | crates.io and docs.rs link to repository and homepage; when there is a book, it is the homepage a reader wants | small |
| [M0034](../lints/M0034.md) | readme-badges | `warn` | the top of the README tells a visitor whether the project passes and where its book, crate and API docs are | small |
| [P0001](../lints/P0001.md) | agent-file-lines | `warn` | agents read AGENTS.md whole every session; a long one is skimmed, not followed | small |
| [P0009](../lints/P0009.md) | changelog-entry-length | `warn` | a CHANGELOG is a list: one short line per change | small |
| [S0002](../lints/S0002.md) | function-complexity | `warn` | more paths than can be tested or held in one's head | small |
| [S0003](../lints/S0003.md) | function-lines | `warn` | a function longer than a screen is changed without being read whole | small |
| [T0001](../lints/T0001.md) | lock-held-across | `warn` | a guard across .await or I/O stalls every other caller for as long as the slowest peer | small |
| [T0004](../lints/T0004.md) | static-interior-mutability | `warn` | every piece of mutable state has one owner; a global lock says why | small |
| [T0005](../lints/T0005.md) | static-mut | `deny` | global mutable state with no synchronisation; there is always a better owner | small |
| [X0005](../lints/X0005.md) | cargo-doc | `warn` | broken intra-doc links and tags render wrong on docs.rs and in the IDE | small |
| [X0019](../lints/X0019.md) | deny-advisories | `warn` | a dependency with a known vulnerability is a risk taken unseen; needs no deny.toml | small |
| [X0026](../lints/X0026.md) | machete-unused-deps | `warn` | an unused dependency costs build time, audit and lockfile churn for nothing | small |
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
# Tier 1: one crate. The house basics every Rust repo meets from its first commit.
[tier]
description = "One crate: metadata, docs, pure-Rust dependencies, build profiles, the repo's rule files, valid waivers"

[lints.waiver-invalid]
level = "deny"
reason = "a waiver without a reason, for an unknown lint, or expired hides a finding without saying why"

[lints.waiver-unused]
level = "warn"
reason = "a waiver that allows nothing is left over from changed code"

[lints.tool-version]
level = "deny"
reason = "an external tool outside its tested range may print a shape the adapter misreads: its results cannot be trusted"

[lints.package-description]
level = "warn"
reason = "the crate's one-line description is written once, in Cargo.toml; tables and READMEs are made from it"

[lints.package-license]
level = "warn"
reason = "a crate without a licence cannot be reused or published"

[lints.edition-latest]
level = "warn"
reason = "new code starts on the current edition"

[lints.readme-per-crate]
level = "warn"
reason = "every crate says what it is and how to use it"

[lints.crate-docs]
level = "warn"
reason = "rustdoc's front page of the crate is its crate-level docs"

[lints.default-features-off]
level = "warn"
reason = "dependencies are added with default-features = false and only the features in use"

[lints.no-sys-crates]
level = "deny"
reason = "pure-Rust crates only: no bundled C, no -sys bindings without a written reason"

[lints.git-dependency-pins]
level = "warn"
reason = "git dependencies follow a branch; Cargo.lock fixes the commit"

[lints.path-outside-workspace]
level = "deny"
reason = "a path into a sibling checkout builds differently on every PC"

[lints.profile-defaults]
level = "warn"
reason = "split debug info and an incremental deploy profile: measured large build-time wins"

[lints.profile-no-lto]
level = "warn"
reason = "LTO in the loop's profiles turns a link of a second into minutes; it belongs in a release profile"

[lints.patch-outside-workspace]
level = "deny"
reason = "a committed patch into a sibling checkout builds differently on every machine; it lives in an ignored .cargo/config.toml"

[lints.repo-files]
level = "warn"
reason = "every work repo carries README, AGENTS, CLAUDE, FEATURES and CHANGELOG"

[lints.changelog-unreleased]
level = "warn"
reason = "work lands under [Unreleased]; the release step turns it into a version section"

[lints.justfile-recipes]
level = "warn"
reason = "every repo answers to the same recipes: default, test, lint, install, deploy"

[lints.manifest-homepage]
level = "warn"
reason = "crates.io and docs.rs link to repository and homepage; when there is a book, it is the homepage a reader wants"

[lints.readme-badges]
level = "warn"
reason = "the top of the README tells a visitor whether the project passes and where its book, crate and API docs are"

[lints.function-lines]
level = "warn"
reason = "a function longer than a screen is changed without being read whole"

[lints.function-complexity]
level = "warn"
reason = "more paths than can be tested or held in one's head"

[lints.static-mut]
level = "deny"
reason = "global mutable state with no synchronisation; there is always a better owner"

[lints.static-interior-mutability]
level = "warn"
reason = "every piece of mutable state has one owner; a global lock says why"

[lints.lock-held-across]
level = "warn"
reason = "a guard across .await or I/O stalls every other caller for as long as the slowest peer"

[lints.todo-macros]
level = "warn"
reason = "no stop-gaps: a todo!() is a panic waiting in shipped code"

[lints.allow-without-reason]
level = "warn"
reason = "an exception to a lint says why, or it outlives its reason"

[lints.print-in-library]
level = "warn"
reason = "printing is the binary's choice"

[lints.changelog-entry-length]
level = "warn"
reason = "a CHANGELOG is a list: one short line per change"

[lints.agent-file-lines]
level = "warn"
reason = "agents read AGENTS.md whole every session; a long one is skimmed, not followed"

[lints.typos-misspelling]
level = "warn"
reason = "a misspelled identifier is copied to every call site; typos has almost no false positives"

[lints.machete-unused-deps]
level = "warn"
reason = "an unused dependency costs build time, audit and lockfile churn for nothing"

[lints.deny-advisories]
level = "warn"
reason = "a dependency with a known vulnerability is a risk taken unseen; needs no deny.toml"

[lints.cargo-doc]
level = "warn"
reason = "broken intra-doc links and tags render wrong on docs.rs and in the IDE"

[lints.tier-clippy-set]
level = "warn"
reason = "Clippy runs in the editor and CI from the project's own lint table; the tier says what belongs in it"

# CLIs (the crates the map marks cli, or with a binary and clap): built and run with help, version and
# completion probes; docs/cli.md has the catalogue

[lints.cli-help-examples]
level = "warn"
reason = "a reader learns a command from its examples first; most existing CLIs lack them on many leaves"

[lints.cli-help-text]
level = "warn"
reason = "-h is the map of a tool; an entry without a line leaves the reader guessing"

[lints.cli-help-clean]
level = "warn"
reason = "rustdoc source in help hides the tool's description"

[lints.cli-completion]
level = "warn"
reason = "completion makes a tool discoverable at the prompt; older CLIs add it when touched"

[lints.cli-plain-when-piped]
level = "warn"
reason = "colour escapes in a pipe break grep, diffs and scripts"

[lints.cli-color-flag]
level = "warn"
reason = "--color settles what automatic colour guesses wrong, the same way in every tool; older CLIs add it when touched"

[lints.cli-usage-error]
level = "warn"
reason = "a caller tells a wrong call from a failure by exit code 2"

[lints.cli-version-line]
level = "warn"
reason = "the SHA and commit time say which build runs; older CLIs print other formats"

[lints.cli-standard-flags]
level = "warn"
reason = "short flags are habits across tools; a different meaning turns a habit into a mistake"

[lints.cli-json-output]
level = "allow"
reason = "an estimate: help does not say which commands print data, so it fires on every setup command until a project lists them in exempt"

[lints.cli-secret-flags]
level = "warn"
reason = "a candidate: a flag's name is a guess at its purpose, so a human reads each hit; the shell history and the process list both see a value passed on the command line"

[lints.cli-no-abbreviations]
level = "warn"
reason = "a caller's shorthand for one subcommand can silently point at a different one once a sibling is added"

[lints.command-registry]
level = "warn"
reason = "an estimate: past a handful of commands, one central list is the line every new command edits and every parallel branch conflicts on"

[lints.float-partial-cmp-unwrap]
level = "warn"
reason = "partial_cmp(..).unwrap() panics on the first NaN; total_cmp orders every float"

[lints.build-script-clock]
level = "warn"
reason = "a clock read makes the build unreproducible; a hardcoded .git path breaks in a worktree checkout"

# The Clippy set every tier expects in [workspace.lints.clippy] (or [lints.clippy]); tier-clippy-set checks it.

[clippy.lints]
unwrap_used = { level = "warn", reason = "a panic says what failed (expect with a message), or the error is returned" }
debug_assert_with_mut_call = { level = "warn", reason = "a debug_assert! that calls a &mut method makes debug and release builds behave differently" }
get_unwrap = { level = "warn", reason = "`.get(i).unwrap()` is indexing with a worse message: index, or handle the None" }
todo = { level = "warn", reason = "no stop-gaps: a todo!() is a panic waiting in shipped code" }
unimplemented = { level = "warn", reason = "no stop-gaps: an unimplemented!() is a panic waiting in shipped code" }
dbg_macro = { level = "warn", reason = "debugging output left behind" }
exit = { level = "warn", reason = "process::exit skips every destructor; main returns instead" }
mem_forget = { level = "warn", reason = "forget leaks the value and skips Drop; ManuallyDrop says it on purpose" }
let_underscore_must_use = { level = "warn", reason = "`let _ =` on a must_use value drops an error unseen" }
map_err_ignore = { level = "warn", reason = "`map_err(|_| ...)` throws the cause away; wrap it as the source" }
fallible_impl_from = { level = "warn", reason = "a From that can panic is a TryFrom" }
undocumented_unsafe_blocks = { level = "warn", reason = "every unsafe block says in a SAFETY: comment why it is sound" }
unnecessary_safety_doc = { level = "warn", reason = "a # Safety section on a safe function sends readers looking for a contract that is not there" }
as_ptr_cast_mut = { level = "warn", reason = "`as_ptr() as *mut` writes through a shared borrow: undefined behaviour" }
ignore_without_reason = { level = "warn", reason = "an ignored test says why, or nobody knows when it may run again" }

[clippy.config]
allow-unwrap-in-tests = { value = true, reason = "a test that unwraps fails at the unwrap, which is what a test should do" }

# Docs as the reader meets them (the docs lints). The budgets warn: they fire on most repos with long docs, and a
# project raises a budget, marks its reference pages ([docs] reference) or turns a lint to deny once its pages fit.

[lints.prose-run-words]
level = "warn"
reason = "a long stretch of prose with no heading, figure, table or code block is skimmed, not read"

[lints.page-reading-time]
level = "warn"
reason = "a guide longer than one sitting is left half read"

[lints.heading-words]
level = "warn"
reason = "headings are scanned to find a section; a sentence is read slowly"

[lints.page-headings]
level = "warn"
reason = "a long page without sections gives the reader no map"

[lints.code-wall]
level = "warn"
reason = "code without prose leaves the reader to guess what it shows"

[lints.image-alt]
level = "warn"
reason = "an image without alternative text tells a screen reader, a search engine and a broken link nothing"

[lints.code-block-language]
level = "warn"
reason = "the language selects the highlighting and tells the reader what the block is"

[lints.nav-entries]
level = "deny"
reason = "a navigation entry that names no page renders as a link that leads nowhere, and the build does not fail"

[lints.orphan-pages]
level = "warn"
reason = "a page nobody reaches is published, unread and left to go stale"

[lints.page-links]
level = "deny"
reason = "a link to a page that moved or never existed, or to the built site, leads a reader nowhere"

[lints.page-title]
level = "warn"
reason = "a page without a title is shown under its file name"

[lints.admonition-syntax]
level = "deny"
reason = "an unknown type loses its colour and a body indented less than four spaces renders outside the box"

[lints.abbreviation-first-use]
level = "warn"
reason = "a reader arrives on any page and cannot look up an abbreviation the page never spells out"

[lints.term-before-definition]
level = "warn"
reason = "a term used before its definition is guessed at, and a wrong guess is read into every sentence until then"

[lints.term-synonyms]
level = "warn"
reason = "a reader takes two names for two things and looks for a difference that is not there"

[lints.passive-voice]
level = "warn"
reason = "a passive sentence hides who acts; a page of them reads as things that happen on their own"

[lints.reading-ease]
level = "warn"
reason = "long sentences of long words are read slowly and understood less"

[lints.book-build-stamp]
level = "warn"
reason = "without a version, commit and age in the footer, a reader cannot tell which build they are looking at"

[lints.readme-index-clash]
level = "deny"
reason = "Zensical does not say which of README.md and index.md it builds when a folder holds both"

[lints.theme-feature-conflicts]
level = "deny"
reason = "Zensical does not support the conflicting features together, and builds neither error nor warn about it"

[lints.zensical-keys]
level = "deny"
reason = "Zensical ignores a key it does not know, silently; a typo in zensical.toml never fails the build"

[lints.book-deploy]
level = "warn"
reason = "a book published by hand falls behind main, and a broken page goes live; one workflow builds it strict and deploys it"

# Units in types (exact_units)

[lints.exact-units-quantity]
level = "warn"
reason = "a unit in a name is checked by no one; in the type the compiler checks every use. Names only, so a warning"

[lints.exact-units-bounds]
level = "warn"
reason = "a range the code enforces belongs in the type, where every value is checked when it is made"
```
