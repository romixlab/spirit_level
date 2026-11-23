# Sets

A set is a small group of lints by what they guard. The size tiers are built from sets: each tier extends the one below it and adds its own sets, and its own table only raises lints those sets turn on. A project extends a set on its own to take those lints before a whole bigger tier: `extends = ["small-workspace", "msrv", "deps-strict"]`. A set a tier already holds is applied once, so naming it again never lowers a level the tier raised.

A guard's lints that enter at a later tier sit in its `-strict` and `-paranoid` sets: `panics` comes with small, `panics-strict` with medium, `panics-paranoid` with big.

## In small

### waivers { #waivers }

Waivers and lint exceptions: valid, used, with a reason.

| Code | Lint | Level |
|---|---|---|
| [C0002](../lints/C0002.md) | allow-without-reason | `warn` |
| [C0012](../lints/C0012.md) | waiver-invalid | `deny` |
| [C0013](../lints/C0013.md) | waiver-unused | `warn` |

### panics { #panics }

Panics and lost errors: no bare unwrap, no exit, no dropped error cause.

| Code | Lint | Level |
|---|---|---|
| [C0032](../lints/C0032.md) | float-partial-cmp-unwrap | `warn` |
| [Clippy](../clippy/debug_assert_with_mut_call.md) | `debug_assert_with_mut_call` | `warn` |
| [Clippy](../clippy/exit.md) | `exit` | `warn` |
| [Clippy](../clippy/fallible_impl_from.md) | `fallible_impl_from` | `warn` |
| [Clippy](../clippy/get_unwrap.md) | `get_unwrap` | `warn` |
| [Clippy](../clippy/let_underscore_must_use.md) | `let_underscore_must_use` | `warn` |
| [Clippy](../clippy/map_err_ignore.md) | `map_err_ignore` | `warn` |
| [Clippy](../clippy/unwrap_used.md) | `unwrap_used` | `warn` |

### stop-gaps { #stop-gaps }

Stop-gaps in code: todo!, unimplemented!, dbg!, ignored tests without a reason.

| Code | Lint | Level |
|---|---|---|
| [C0008](../lints/C0008.md) | todo-macros | `warn` |
| [Clippy](../clippy/dbg_macro.md) | `dbg_macro` | `warn` |
| [Clippy](../clippy/ignore_without_reason.md) | `ignore_without_reason` | `warn` |
| [Clippy](../clippy/todo.md) | `todo` | `warn` |
| [Clippy](../clippy/unimplemented.md) | `unimplemented` | `warn` |

### unsafe { #unsafe }

Unsafe code: every unsafe block says why it is sound.

| Code | Lint | Level |
|---|---|---|
| [Clippy](../clippy/as_ptr_cast_mut.md) | `as_ptr_cast_mut` | `warn` |
| [Clippy](../clippy/mem_forget.md) | `mem_forget` | `warn` |
| [Clippy](../clippy/undocumented_unsafe_blocks.md) | `undocumented_unsafe_blocks` | `warn` |
| [Clippy](../clippy/unnecessary_safety_doc.md) | `unnecessary_safety_doc` | `warn` |

### state { #state }

Shared state: no static mut, statics with interior mutability named, no lock held across a wait.

| Code | Lint | Level |
|---|---|---|
| [T0001](../lints/T0001.md) | lock-held-across | `warn` |
| [T0004](../lints/T0004.md) | static-interior-mutability | `warn` |
| [T0005](../lints/T0005.md) | static-mut | `deny` |

### printing { #printing }

Library code does not print.

| Code | Lint | Level |
|---|---|---|
| [C0006](../lints/C0006.md) | print-in-library | `warn` |

### deps { #deps }

Dependencies: pure Rust, default features off, no pins or paths outside the workspace, advisories, unused ones.

| Code | Lint | Level |
|---|---|---|
| [M0001](../lints/M0001.md) | default-features-off | `warn` |
| [M0003](../lints/M0003.md) | git-dependency-pins | `warn` |
| [M0004](../lints/M0004.md) | no-sys-crates | `deny` |
| [M0005](../lints/M0005.md) | path-outside-workspace | `deny` |
| [M0027](../lints/M0027.md) | patch-outside-workspace | `deny` |
| [X0019](../lints/X0019.md) | deny-advisories | `warn` |
| [X0026](../lints/X0026.md) | machete-unused-deps | `warn` |

### docs { #docs }

API docs: crate docs, cargo doc without warnings.

| Code | Lint | Level |
|---|---|---|
| [M0006](../lints/M0006.md) | crate-docs | `warn` |
| [X0005](../lints/X0005.md) | cargo-doc | `warn` |

### book { #book }

The Zensical book: navigation, titles, admonitions, build stamp, deploy.

| Code | Lint | Level |
|---|---|---|
| [D0008](../lints/D0008.md) | nav-entries | `deny` |
| [D0009](../lints/D0009.md) | orphan-pages | `warn` |
| [D0010](../lints/D0010.md) | page-title | `warn` |
| [D0011](../lints/D0011.md) | admonition-syntax | `deny` |
| [D0017](../lints/D0017.md) | book-build-stamp | `warn` |
| [D0018](../lints/D0018.md) | readme-index-clash | `deny` |
| [D0019](../lints/D0019.md) | theme-feature-conflicts | `deny` |
| [D0020](../lints/D0020.md) | zensical-keys | `deny` |
| [D0022](../lints/D0022.md) | book-deploy | `warn` |
| [D0026](../lints/D0026.md) | glossary-terms | `warn` |

### reading { #reading }

Pages as a reader meets them: length, headings, figures, prose runs, links.

| Code | Lint | Level |
|---|---|---|
| [D0001](../lints/D0001.md) | prose-run-words | `warn` |
| [D0002](../lints/D0002.md) | page-reading-time | `warn` |
| [D0003](../lints/D0003.md) | heading-words | `warn` |
| [D0004](../lints/D0004.md) | page-headings | `warn` |
| [D0005](../lints/D0005.md) | code-wall | `warn` |
| [D0006](../lints/D0006.md) | image-alt | `warn` |
| [D0007](../lints/D0007.md) | code-block-language | `warn` |
| [D0021](../lints/D0021.md) | page-links | `deny` |
| [D0023](../lints/D0023.md) | page-figures | `warn` |
| [D0024](../lints/D0024.md) | paragraph-run-words | `warn` |
| [D0025](../lints/D0025.md) | prose-per-element | `warn` |

### prose { #prose }

Wording: terms, abbreviations, plain sentences, spelling, short CHANGELOG entries.

| Code | Lint | Level |
|---|---|---|
| [D0012](../lints/D0012.md) | abbreviation-first-use | `warn` |
| [D0013](../lints/D0013.md) | term-before-definition | `warn` |
| [D0014](../lints/D0014.md) | term-synonyms | `warn` |
| [D0015](../lints/D0015.md) | passive-voice | `warn` |
| [D0016](../lints/D0016.md) | reading-ease | `warn` |
| [P0009](../lints/P0009.md) | changelog-entry-length | `warn` |
| [X0032](../lints/X0032.md) | typos-misspelling | `warn` |

### repo { #repo }

The repo's files: README, CHANGELOG, AGENTS.md, justfile, pinned toolchain, tool versions.

| Code | Lint | Level |
|---|---|---|
| [M0014](../lints/M0014.md) | changelog-unreleased | `warn` |
| [M0015](../lints/M0015.md) | justfile-recipes | `warn` |
| [M0018](../lints/M0018.md) | repo-files | `warn` |
| [M0019](../lints/M0019.md) | toolchain-pinned | `warn` |
| [M0034](../lints/M0034.md) | readme-badges | `warn` |
| [M0036](../lints/M0036.md) | toolchain-overrides-pinned | `warn` |
| [P0001](../lints/P0001.md) | agent-file-lines | `warn` |
| [X0034](../lints/X0034.md) | tool-version | `deny` |

### build { #build }

Build profiles and build scripts: debug info, no LTO in test and deploy, no clock.

| Code | Lint | Level |
|---|---|---|
| [C0040](../lints/C0040.md) | build-script-clock | `warn` |
| [M0017](../lints/M0017.md) | profile-defaults | `warn` |
| [M0026](../lints/M0026.md) | profile-no-lto | `warn` |
| [M0037](../lints/M0037.md) | dev-debug-off | `warn` |

### manifest { #manifest }

Package metadata: description, licence, homepage, edition, README, the Clippy table.

| Code | Lint | Level |
|---|---|---|
| [M0007](../lints/M0007.md) | edition-latest | `warn` |
| [M0009](../lints/M0009.md) | package-description | `warn` |
| [M0010](../lints/M0010.md) | package-license | `warn` |
| [M0012](../lints/M0012.md) | readme-per-crate | `warn` |
| [M0025](../lints/M0025.md) | tier-clippy-set | `warn` |
| [M0033](../lints/M0033.md) | manifest-homepage | `warn` |

### cli { #cli }

Command-line tools: help, version, completion, colour, exit codes, secrets.

| Code | Lint | Level |
|---|---|---|
| [C0020](../lints/C0020.md) | cli-help-examples | `warn` |
| [C0021](../lints/C0021.md) | cli-help-text | `warn` |
| [C0022](../lints/C0022.md) | cli-help-clean | `warn` |
| [C0023](../lints/C0023.md) | cli-completion | `warn` |
| [C0024](../lints/C0024.md) | cli-plain-when-piped | `warn` |
| [C0025](../lints/C0025.md) | cli-color-flag | `warn` |
| [C0026](../lints/C0026.md) | cli-usage-error | `warn` |
| [C0027](../lints/C0027.md) | cli-version-line | `warn` |
| [C0028](../lints/C0028.md) | cli-json-output | `allow` |
| [C0029](../lints/C0029.md) | cli-standard-flags | `warn` |
| [C0037](../lints/C0037.md) | cli-secret-flags | `warn` |
| [C0038](../lints/C0038.md) | cli-no-abbreviations | `warn` |
| [C0039](../lints/C0039.md) | command-registry | `warn` |

### size { #size }

Function size and complexity.

| Code | Lint | Level |
|---|---|---|
| [S0002](../lints/S0002.md) | function-complexity | `warn` |
| [S0003](../lints/S0003.md) | function-lines | `warn` |

### units { #units }

Quantities and bounds in exact_units types.

| Code | Lint | Level |
|---|---|---|
| [C0033](../lints/C0033.md) | exact-units-quantity | `warn` |
| [C0035](../lints/C0035.md) | exact-units-bounds | `warn` |

## In small-workspace

### workspace { #workspace }

A workspace declared once at the root: inherited keys, lints and dependencies, one version each, no cycles, CI.

| Code | Lint | Level |
|---|---|---|
| [G0003](../lints/G0003.md) | crate-cycles | `warn` |
| [M0002](../lints/M0002.md) | duplicate-versions | `warn` |
| [M0020](../lints/M0020.md) | crate-dir-name | `warn` |
| [M0021](../lints/M0021.md) | macro-crate-suffix | `warn` |
| [M0022](../lints/M0022.md) | workspace-dependencies | `warn` |
| [M0023](../lints/M0023.md) | workspace-inheritance | `deny` |
| [M0024](../lints/M0024.md) | workspace-lints | `deny` |
| [M0032](../lints/M0032.md) | ci-workflow | `warn` |
| [X0031](../lints/X0031.md) | taplo-fmt | `warn` |

### state-strict { #state-strict }

Shared state beyond state: nested locks and thread-local state.

| Code | Lint | Level |
|---|---|---|
| [T0003](../lints/T0003.md) | nested-locks | `warn` |
| [T0006](../lints/T0006.md) | thread-local-state | `warn` |

## In medium

### clippy-pedantic { #clippy-pedantic }

Clippy's pedantic and cargo groups with the opt-outs that are noise, and the idioms beside them.

| Code | Lint | Level |
|---|---|---|
| [Clippy](../clippy/assigning_clones.md) | `assigning_clones` | `allow` |
| [Clippy](../clippy/cargo.md) | `cargo` | `warn` |
| [Clippy](../clippy/cargo_common_metadata.md) | `cargo_common_metadata` | `allow` |
| [Clippy](../clippy/cast_lossless.md) | `cast_lossless` | `allow` |
| [Clippy](../clippy/cast_possible_truncation.md) | `cast_possible_truncation` | `allow` |
| [Clippy](../clippy/cast_precision_loss.md) | `cast_precision_loss` | `allow` |
| [Clippy](../clippy/cast_sign_loss.md) | `cast_sign_loss` | `allow` |
| [Clippy](../clippy/default_trait_access.md) | `default_trait_access` | `allow` |
| [Clippy](../clippy/float_cmp.md) | `float_cmp` | `allow` |
| [Clippy](../clippy/implicit_hasher.md) | `implicit_hasher` | `allow` |
| [Clippy](../clippy/inline_always.md) | `inline_always` | `allow` |
| [Clippy](../clippy/items_after_statements.md) | `items_after_statements` | `allow` |
| [Clippy](../clippy/many_single_char_names.md) | `many_single_char_names` | `allow` |
| [Clippy](../clippy/map_unwrap_or.md) | `map_unwrap_or` | `allow` |
| [Clippy](../clippy/missing_errors_doc.md) | `missing_errors_doc` | `allow` |
| [Clippy](../clippy/multiple_crate_versions.md) | `multiple_crate_versions` | `allow` |
| [Clippy](../clippy/must_use_candidate.md) | `must_use_candidate` | `allow` |
| [Clippy](../clippy/needless_raw_string_hashes.md) | `needless_raw_string_hashes` | `allow` |
| [Clippy](../clippy/pedantic.md) | `pedantic` | `warn` |
| [Clippy](../clippy/redundant_closure_for_method_calls.md) | `redundant_closure_for_method_calls` | `allow` |
| [Clippy](../clippy/return_self_not_must_use.md) | `return_self_not_must_use` | `allow` |
| [Clippy](../clippy/self_named_module_files.md) | `self_named_module_files` | `warn` |
| [Clippy](../clippy/similar_names.md) | `similar_names` | `allow` |
| [Clippy](../clippy/str_to_string.md) | `str_to_string` | `warn` |
| [Clippy](../clippy/struct_excessive_bools.md) | `struct_excessive_bools` | `allow` |
| [Clippy](../clippy/struct_field_names.md) | `struct_field_names` | `allow` |
| [Clippy](../clippy/too_long_first_doc_paragraph.md) | `too_long_first_doc_paragraph` | `warn` |
| [Clippy](../clippy/too_many_lines.md) | `too_many_lines` | `allow` |
| [Clippy](../clippy/trivially_copy_pass_by_ref.md) | `trivially_copy_pass_by_ref` | `allow` |
| [Clippy](../clippy/unnecessary_debug_formatting.md) | `unnecessary_debug_formatting` | `allow` |
| [Clippy](../clippy/unreadable_literal.md) | `unreadable_literal` | `allow` |
| [Clippy](../clippy/use_self.md) | `use_self` | `warn` |
| [Clippy](../clippy/used_underscore_binding.md) | `used_underscore_binding` | `allow` |

### waivers-strict { #waivers-strict }

Lint exceptions as #[expect], which fails once the lint stops firing.

| Code | Lint | Level |
|---|---|---|
| [Clippy](../clippy/allow_attributes.md) | `allow_attributes` | `warn` |

### panics-strict { #panics-strict }

Panics in libraries: errors returned, panics documented, error enums kept small.

| Code | Lint | Level |
|---|---|---|
| [C0005](../lints/C0005.md) | panic-in-library | `warn` |
| [C0010](../lints/C0010.md) | error-enum-growth | `warn` |
| [Clippy](../clippy/missing_panics_doc.md) | `missing_panics_doc` | `warn` |

### printing-strict { #printing-strict }

No printing outside the binary's one place, Clippy's side of it.

| Code | Lint | Level |
|---|---|---|
| [Clippy](../clippy/print_stderr.md) | `print_stderr` | `warn` |
| [Clippy](../clippy/print_stdout.md) | `print_stdout` | `warn` |

### state-paranoid { #state-paranoid }

Where shared state concentrates: lock().unwrap() counted per crate.

| Code | Lint | Level |
|---|---|---|
| [T0002](../lints/T0002.md) | lock-unwrap | `warn` |

### licences { #licences }

Dependency licences against deny.toml.

| Code | Lint | Level |
|---|---|---|
| [X0021](../lints/X0021.md) | deny-licenses | `warn` |

### msrv { #msrv }

The MSRV declared and checked.

| Code | Lint | Level |
|---|---|---|
| [M0013](../lints/M0013.md) | rust-version-set | `warn` |
| [X0006](../lints/X0006.md) | cargo-msrv | `warn` |

### commits { #commits }

Commit messages: subject length, the Fixes: tag.

| Code | Lint | Level |
|---|---|---|
| [P0007](../lints/P0007.md) | commit-subject-length | `warn` |
| [P0017](../lints/P0017.md) | fixes-tag | `warn` |

### tests { #tests }

Test layout: test binaries, fuzz corpora replayed, test support gated, compile-fail tests for macros.

| Code | Lint | Level |
|---|---|---|
| [C0015](../lints/C0015.md) | test-support-gated | `warn` |
| [M0028](../lints/M0028.md) | test-binaries | `warn` |
| [M0029](../lints/M0029.md) | macro-compile-fail | `warn` |
| [M0031](../lints/M0031.md) | fuzz-corpus-test | `warn` |

### size-strict { #size-strict }

Module, crate and crates/ folder size; cognitive complexity.

| Code | Lint | Level |
|---|---|---|
| [S0001](../lints/S0001.md) | crate-lines | `warn` |
| [S0004](../lints/S0004.md) | module-lines | `warn` |
| [S0006](../lints/S0006.md) | crate-folder-size | `warn` |
| [Clippy](../clippy/cognitive_complexity.md) | `cognitive_complexity` | `warn` |

### structure { #structure }

Workspace structure: no module cycles or orphan files, small _core crates, libraries called as libraries, build info read once.

| Code | Lint | Level |
|---|---|---|
| [C0003](../lints/C0003.md) | cli-subprocess | `deny` |
| [C0011](../lints/C0011.md) | module-graph-scc | `warn` |
| [C0014](../lints/C0014.md) | build-info-env | `warn` |
| [C0016](../lints/C0016.md) | orphan-source-file | `warn` |
| [G0001](../lints/G0001.md) | core-crate-deps | `warn` |

### contracts { #contracts }

Types that cross a boundary have sized integers.

| Code | Lint | Level |
|---|---|---|
| [C0017](../lints/C0017.md) | sized-ints-in-contracts | `warn` |

### workspace-strict { #workspace-strict }

Internal crates are not published.

| Code | Lint | Level |
|---|---|---|
| [M0011](../lints/M0011.md) | publish-false | `warn` |

## In big

### panics-paranoid { #panics-paranoid }

Assertions that say what broke, one condition each; bounded reads.

| Code | Lint | Level |
|---|---|---|
| [C0019](../lints/C0019.md) | unbounded-read | `warn` |
| [C0030](../lints/C0030.md) | compound-assert | `warn` |
| [C0031](../lints/C0031.md) | assert-message | `warn` |

### stop-gaps-strict { #stop-gaps-strict }

Stop-gap comments and FIXMEs never land.

| Code | Lint | Level |
|---|---|---|
| [A0010](../lints/A0010.md) | stop-gap-comments | `deny` |
| [A0011](../lints/A0011.md) | fixme-before-land | `deny` |

### deps-strict { #deps-strict }

Dependency bans and sources against deny.toml; the workspace-hack crate kept current.

| Code | Lint | Level |
|---|---|---|
| [X0020](../lints/X0020.md) | deny-bans | `deny` |
| [X0022](../lints/X0022.md) | deny-sources | `deny` |
| [X0024](../lints/X0024.md) | hakari-generate | `warn` |
| [X0025](../lints/X0025.md) | hakari-manage-deps | `warn` |

### manifest-strict { #manifest-strict }

Every cargo feature documented.

| Code | Lint | Level |
|---|---|---|
| [M0008](../lints/M0008.md) | feature-docs | `warn` |

### repo-strict { #repo-strict }

Simple justfile recipes; AGENTS.md matches the lints.

| Code | Lint | Level |
|---|---|---|
| [M0016](../lints/M0016.md) | justfile-simple | `warn` |
| [P0003](../lints/P0003.md) | agent-rules-match-lints | `warn` |

### build-strict { #build-strict }

No_std and wasm crates build for their targets.

| Code | Lint | Level |
|---|---|---|
| [X0007](../lints/X0007.md) | cargo-no-std | `deny` |
| [X0008](../lints/X0008.md) | cargo-wasm | `deny` |

### tests-strict { #tests-strict }

Fixes come with tests; test files stay small.

| Code | Lint | Level |
|---|---|---|
| [P0018](../lints/P0018.md) | fix-without-test | `warn` |
| [S0005](../lints/S0005.md) | test-lines | `warn` |

### contracts-strict { #contracts-strict }

Contract schemas snapshotted; breaking changes found by cargo-semver-checks.

| Code | Lint | Level |
|---|---|---|
| [M0030](../lints/M0030.md) | contract-snapshot | `deny` |
| [X0028](../lints/X0028.md) | semver-checks-breaking | `deny` |

### layers { #layers }

The crate map: layers checked edge by edge, folders, licences, unsafe and third-party crates placed.

| Code | Lint | Level |
|---|---|---|
| [G0002](../lints/G0002.md) | core-crate-types | `deny` |
| [G0004](../lints/G0004.md) | crate-table | `deny` |
| [G0005](../lints/G0005.md) | external-form | `deny` |
| [G0006](../lints/G0006.md) | forbidden-closure | `deny` |
| [G0007](../lints/G0007.md) | layer-edges | `deny` |
| [G0008](../lints/G0008.md) | map-complete | `deny` |
| [G0009](../lints/G0009.md) | third-party-placement | `deny` |
| [G0010](../lints/G0010.md) | unsafe-allow-list | `deny` |
| [G0011](../lints/G0011.md) | crate-folder-layer | `deny` |
| [G0012](../lints/G0012.md) | map-licence | `deny` |

### api { #api }

A small public surface and no dead public items.

| Code | Lint | Level |
|---|---|---|
| [C0001](../lints/C0001.md) | pub-surface | `warn` |
| [C0009](../lints/C0009.md) | dead-pub | `warn` |

### determinism { #determinism }

Deterministic results: libm floats, fixed-seed hashing, exact float literals and comparisons.

| Code | Lint | Level |
|---|---|---|
| [C0004](../lints/C0004.md) | float-determinism | `deny` |
| [C0007](../lints/C0007.md) | random-state-hash | `deny` |
| [Clippy](../clippy/float_cmp_const.md) | `float_cmp_const` | `warn` |
| [Clippy](../clippy/imprecise_flops.md) | `imprecise_flops` | `warn` |
| [Clippy](../clippy/iter_over_hash_type.md) | `iter_over_hash_type` | `warn` |
| [Clippy](../clippy/lossy_float_literal.md) | `lossy_float_literal` | `warn` |

## Beside the tiers

### units-strict { #units-strict }

Units everywhere: indices, counts, sequence numbers and time spans in exact_units types, besides the quantities every tier checks.

| Code | Lint | Level |
|---|---|---|
| [C0034](../lints/C0034.md) | exact-units-plain-number | `warn` |

### units-paranoid { #units-paranoid }

Units inside function bodies: locals, literal arguments, casts, float maths, bare comparisons, raw arithmetic, scale constants and conversions by hand.

Extends [units-strict](#units-strict).

| Code | Lint | Level |
|---|---|---|
| [C0036](../lints/C0036.md) | exact-units-in-bodies | `warn` |
