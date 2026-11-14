# big

A big workspace: a crate map with layers checked edge by edge, size limits with expiring exceptions, strict conformity.

Extends [medium](medium.md): everything below is on top of it.

Use it with `extends = "big"` in `spirit_level.toml`.

## Lints enabled (with what it extends)

| Code | Lint | Level | Reason | Set in |
|---|---|---|---|---|
| [A0010](../lints/A0010.md) | stop-gap-comments | `deny` | no stop-gaps: a TODO or a 'for now' names its tracker id or goes | big |
| [C0001](../lints/C0001.md) | pub-surface | `warn` | in a big workspace pub is the easy way across crates; a library that is all surface keeps nothing apart | big |
| [C0002](../lints/C0002.md) | allow-without-reason | `deny` | an exception to a lint says why, or it outlives its reason | big |
| [C0003](../lints/C0003.md) | cli-subprocess | `deny` | components call each other as libraries, not through their CLIs | medium |
| [C0004](../lints/C0004.md) | float-determinism | `deny` | crates the map marks deterministic compute the same bits on every platform | big |
| [C0005](../lints/C0005.md) | panic-in-library | `warn` | a library returns errors; its caller decides what is fatal | medium |
| [C0006](../lints/C0006.md) | print-in-library | `warn` | printing is the binary's choice | small |
| [C0007](../lints/C0007.md) | random-state-hash | `deny` | crates the map marks deterministic iterate in the same order every run | big |
| [C0008](../lints/C0008.md) | todo-macros | `deny` | no stop-gaps: a todo!() is a panic waiting in shipped code | big |
| [C0009](../lints/C0009.md) | dead-pub | `warn` | rustc can't see pub items no other crate uses; in a big workspace they pile up (candidates, never fail) | big |
| [C0010](../lints/C0010.md) | error-enum-growth | `warn` | a flat error enum grows a variant per failure and loses the source errors; wrap them by layer | medium |
| [C0011](../lints/C0011.md) | module-graph-scc | `warn` | a module cycle holding most of a crate can't be split into crates; cut it while it is small | medium |
| [C0012](../lints/C0012.md) | waiver-invalid | `deny` | a waiver without a reason, for an unknown lint, or expired hides a finding without saying why | small |
| [C0013](../lints/C0013.md) | waiver-unused | `deny` | every waiver allows something, or it goes | big |
| [G0001](../lints/G0001.md) | core-crate-deps | `deny` | a _core crate holds shared types and drags nothing in | big |
| [G0002](../lints/G0002.md) | core-crate-types | `deny` | _core crates hold only types: no I/O, threads or runtimes | big |
| [G0003](../lints/G0003.md) | crate-cycles | `deny` | crates in a cycle cannot be built or tested apart | medium |
| [G0004](../lints/G0004.md) | crate-table | `deny` | the crate table is generated from the manifests and the map, never written by hand | big |
| [G0005](../lints/G0005.md) | external-form | `deny` | outside projects come by git on a branch, listed once, used where their role allows | big |
| [G0006](../lints/G0006.md) | forbidden-closure | `deny` | some crates must never end up in a binary, however far away they arrive | big |
| [G0007](../lints/G0007.md) | layer-edges | `deny` | layers are declared and checked edge by edge; move a crate, never reorder layers | big |
| [G0008](../lints/G0008.md) | map-complete | `deny` | the crate map is the source of truth: every crate in it, nothing gone | big |
| [G0009](../lints/G0009.md) | third-party-placement | `deny` | heavy dependencies sit in one crate each | big |
| [G0010](../lints/G0010.md) | unsafe-allow-list | `deny` | unsafe is forbidden except in the crates the map lists | big |
| [M0001](../lints/M0001.md) | default-features-off | `deny` | feature creep in a big graph costs every build | big |
| [M0002](../lints/M0002.md) | duplicate-versions | `deny` | two versions of a crate double its build time and split its types | big |
| [M0003](../lints/M0003.md) | git-dependency-pins | `deny` | git dependencies follow a branch; a pin hides a break until it is old | medium |
| [M0004](../lints/M0004.md) | no-sys-crates | `deny` | pure-Rust crates only: no bundled C, no -sys bindings without a written reason | small |
| [M0005](../lints/M0005.md) | path-outside-workspace | `deny` | a path into a sibling checkout builds differently on every PC | small |
| [M0006](../lints/M0006.md) | crate-docs | `deny` | rustdoc's front page of the crate is its crate-level docs | big |
| [M0007](../lints/M0007.md) | edition-latest | `warn` | new code starts on the current edition | small |
| [M0008](../lints/M0008.md) | feature-docs | `warn` | a feature without a comment cannot be chosen by a reader | big |
| [M0009](../lints/M0009.md) | package-description | `deny` | the generated crate table and READMEs are made from it | big |
| [M0010](../lints/M0010.md) | package-license | `warn` | a crate without a licence cannot be reused or published | small |
| [M0011](../lints/M0011.md) | publish-false | `warn` | internal crates are a build unit, not an API; only listed crates are published | medium |
| [M0012](../lints/M0012.md) | readme-per-crate | `deny` | every crate of a workspace says what it is and what it may depend on | small-workspace |
| [M0013](../lints/M0013.md) | rust-version-set | `deny` | the MSRV is declared so it can be checked | big |
| [M0014](../lints/M0014.md) | changelog-unreleased | `warn` | work lands under [Unreleased]; the release step turns it into a version section | small |
| [M0015](../lints/M0015.md) | justfile-recipes | `warn` | every repo answers to the same recipes: default, test, lint, install, deploy | small |
| [M0016](../lints/M0016.md) | justfile-simple | `warn` | recipes call programs; logic lives in Rust where it is tested | big |
| [M0017](../lints/M0017.md) | profile-defaults | `warn` | split debug info and an incremental deploy profile: measured large build-time wins | small |
| [M0018](../lints/M0018.md) | repo-files | `warn` | every work repo carries README, AGENTS, CLAUDE, FEATURES and CHANGELOG | small |
| [M0019](../lints/M0019.md) | toolchain-pinned | `deny` | every PC and the gate build with the same compiler | big |
| [M0020](../lints/M0020.md) | crate-dir-name | `warn` | flat crates/<name>: the folder is the crate | small-workspace |
| [M0021](../lints/M0021.md) | macro-crate-suffix | `warn` | _macros and _derive mean proc macros and nothing else | small-workspace |
| [M0022](../lints/M0022.md) | workspace-dependencies | `deny` | one version per dependency per workspace, written once in [workspace.dependencies] | medium |
| [M0023](../lints/M0023.md) | workspace-inheritance | `deny` | edition, rust-version, licence and repository are declared once in [workspace.package] | small-workspace |
| [M0024](../lints/M0024.md) | workspace-lints | `deny` | every crate inherits [workspace.lints], so one table is the policy | small-workspace |
| [M0025](../lints/M0025.md) | tier-clippy-set | `deny` | the Clippy set is the lint policy of every crate; a gap in it is a gap in the policy | medium |
| [P0001](../lints/P0001.md) | agent-file-lines | `warn` | agents read AGENTS.md whole every session; a long one is skimmed, not followed | small |
| [P0003](../lints/P0003.md) | agent-rules-match-lints | `warn` | limits AGENTS.md states are the limits the lints check | big |
| [P0007](../lints/P0007.md) | commit-subject-length | `warn` | the subject is what one-line logs and reviews show | medium |
| [P0009](../lints/P0009.md) | changelog-entry-length | `warn` | a CHANGELOG is a list: one short line per change | small |
| [S0001](../lints/S0001.md) | crate-lines | `warn` | split a crate before it passes ~20,000 lines: one crate is one serial compile step | medium |
| [S0002](../lints/S0002.md) | function-complexity | `warn` | more paths than can be tested or held in one's head | small |
| [S0003](../lints/S0003.md) | function-lines | `warn` | a function longer than a screen is changed without being read whole | small |
| [S0004](../lints/S0004.md) | module-lines | `warn` | a file of thousands of lines holds several subjects and collects every edit | medium |
| [S0005](../lints/S0005.md) | test-lines | `warn` | test code builds and links too | big |
| [T0001](../lints/T0001.md) | lock-held-across | `deny` | a guard across .await or I/O stalls every other caller for as long as the slowest peer | big |
| [T0002](../lints/T0002.md) | lock-unwrap | `warn` | the count of lock().unwrap() per crate shows where shared state concentrates | medium |
| [T0003](../lints/T0003.md) | nested-locks | `warn` | two locks at once need one lock order everywhere | small-workspace |
| [T0004](../lints/T0004.md) | static-interior-mutability | `deny` | every piece of mutable state has one owner; a global lock says why on the item | big |
| [T0005](../lints/T0005.md) | static-mut | `deny` | global mutable state with no synchronisation; there is always a better owner | small |
| [T0006](../lints/T0006.md) | thread-local-state | `warn` | thread-local state is a hidden side channel between functions | small-workspace |
| [X0005](../lints/X0005.md) | cargo-doc | `deny` | the docs of internal crates are the map of the code; they build without a warning | medium |
| [X0006](../lints/X0006.md) | cargo-msrv | `deny` | the declared rust-version is built, so it stays true | big |
| [X0007](../lints/X0007.md) | cargo-no-std | `deny` | a crate the map marks no_std is built for a target without std, or the promise breaks unseen | big |
| [X0008](../lints/X0008.md) | cargo-wasm | `deny` | a crate the map marks wasm is built for wasm32, or the promise breaks unseen | big |
| [X0019](../lints/X0019.md) | deny-advisories | `deny` | a vulnerable dependency is updated, or its advisory ignored with a reason in deny.toml | medium |
| [X0020](../lints/X0020.md) | deny-bans | `deny` | banned crates (OpenSSL, a second runtime) and duplicate versions stay out of a big graph | big |
| [X0021](../lints/X0021.md) | deny-licenses | `deny` | every linked licence is one the project accepts, listed once in deny.toml | big |
| [X0022](../lints/X0022.md) | deny-sources | `deny` | crates come only from the registries and git hosts deny.toml trusts | big |
| [X0024](../lints/X0024.md) | hakari-generate | `warn` | a workspace-hack builds each shared dependency once; warn, as the lint needs spl built with the hakari feature | big |
| [X0025](../lints/X0025.md) | hakari-manage-deps | `warn` | the workspace-hack helps only the members that depend on it; warn, as the lint needs the hakari feature | big |
| [X0026](../lints/X0026.md) | machete-unused-deps | `deny` | an unused dependency in a big graph costs every build; false positives are listed in the manifest | big |
| [X0028](../lints/X0028.md) | semver-checks-breaking | `deny` | a crate the map marks stable changes its API only with the version bump it needs | big |
| [X0031](../lints/X0031.md) | taplo-fmt | `deny` | manifests are edited by people and by tools; one TOML formatting keeps their diffs about the change | big |
| [X0032](../lints/X0032.md) | typos-misspelling | `deny` | names are searched across hundreds of crates; a misspelling hides from every search | big |

## Clippy set (with what it extends)

The project turns these on in its own `[workspace.lints.clippy]` (`allow` rows are accepted opt-outs, pasted with their reason); [tier-clippy-set](../lints/M0025.md) checks it.

| Clippy lint | Level | Reason | Set in |
|---|---|---|---|
| `cargo` | `warn` | manifest hygiene: feature names, wildcard versions | medium |
| `pedantic` | `warn` | the stricter idioms; the opt-outs below are the ones that are noise for most code | medium |
| `allow_attributes` | `warn` | #[expect] fails once the lint stops firing, so an exception cannot outlive its cause | medium |
| `as_ptr_cast_mut` | `warn` | `as_ptr() as *mut` writes through a shared borrow: undefined behaviour | small |
| `cognitive_complexity` | `warn` | a function too tangled to follow in one read; complements spl's cyclomatic function-complexity | medium |
| `dbg_macro` | `warn` | debugging output left behind | small |
| `exit` | `warn` | process::exit skips every destructor; main returns instead | small |
| `fallible_impl_from` | `warn` | a From that can panic is a TryFrom | small |
| `float_cmp_const` | `warn` | comparing a float to a constant exactly is almost always a bug | big |
| `get_unwrap` | `warn` | `.get(i).unwrap()` is indexing with a worse message: index, or handle the None | small |
| `imprecise_flops` | `warn` | hypot, ln_1p and friends are more precise than the formula written out | big |
| `iter_over_hash_type` | `warn` | iteration over a hash map changes order from run to run; random-state-hash reports the type | big |
| `let_underscore_must_use` | `warn` | `let _ =` on a must_use value drops an error unseen | small |
| `lossy_float_literal` | `warn` | a float literal that does not fit its type silently becomes another number | big |
| `map_err_ignore` | `warn` | `map_err(\|_\| ...)` throws the cause away; wrap it as the source | small |
| `mem_forget` | `warn` | forget leaks the value and skips Drop; ManuallyDrop says it on purpose | small |
| `print_stderr` | `warn` | printing is the binary's choice, written in one place | medium |
| `print_stdout` | `warn` | printing is the binary's choice, written in one place | medium |
| `self_named_module_files` | `warn` | one module layout, mod.rs, across the workspace | medium |
| `str_to_string` | `warn` | to_owned says what happens: a copy of a borrowed str | medium |
| `todo` | `warn` | no stop-gaps: a todo!() is a panic waiting in shipped code | small |
| `too_long_first_doc_paragraph` | `warn` | the first doc paragraph is the summary rustdoc lists; one short sentence | medium |
| `undocumented_unsafe_blocks` | `warn` | every unsafe block says in a SAFETY: comment why it is sound | small |
| `unimplemented` | `warn` | no stop-gaps: an unimplemented!() is a panic waiting in shipped code | small |
| `unnecessary_safety_doc` | `warn` | a # Safety section on a safe function sends readers looking for a contract that is not there | small |
| `unwrap_used` | `warn` | a panic says what failed (expect with a message), or the error is returned | small |
| `use_self` | `warn` | Self follows a rename | medium |
| `assigning_clones` | `allow` | clone_into reads backwards | medium |
| `cargo_common_metadata` | `allow` | keywords and categories matter only for published crates; package-description covers the rest | medium |
| `cast_lossless` | `allow` | a widening `as` cast loses nothing, and From is longer for no gain | medium |
| `cast_possible_truncation` | `allow` | too noisy: most casts are of values known to fit | medium |
| `cast_precision_loss` | `allow` | too noisy: integer to float for display and statistics is the common case | medium |
| `cast_sign_loss` | `allow` | too noisy: most casts are of values known to be non-negative | medium |
| `default_trait_access` | `allow` | Default::default() is clear where the type is written beside it | medium |
| `float_cmp` | `allow` | exact float comparisons are usually intended; float_cmp_const stays on in big | medium |
| `implicit_hasher` | `allow` | a BuildHasher type parameter on public functions is not worth it | medium |
| `inline_always` | `allow` | #[inline(always)] is written where it was measured | medium |
| `items_after_statements` | `allow` | a helper item next to its only use reads better | medium |
| `many_single_char_names` | `allow` | r, g, b and x, y, z are fine | medium |
| `map_unwrap_or` | `allow` | `.map(f).unwrap_or(x)` often reads better than map_or | medium |
| `missing_errors_doc` | `allow` | the error type documents its cases | medium |
| `missing_panics_doc` | `allow` | too noisy: most panics are invariants, not inputs | medium |
| `multiple_crate_versions` | `allow` | spl's duplicate-versions checks this, with a reasoned allow list | medium |
| `must_use_candidate` | `allow` | thousands of hits; must_use goes where ignoring the value is a bug | medium |
| `needless_raw_string_hashes` | `allow` | r#"..."# keeps a string editable when a quote is added | medium |
| `redundant_closure_for_method_calls` | `allow` | a closure is often clearer than a path to a method | medium |
| `return_self_not_must_use` | `allow` | thousands of hits on builders | medium |
| `similar_names` | `allow` | too many false positives: encoder and encoded are different things | medium |
| `struct_excessive_bools` | `allow` | a struct of flags is often just that | medium |
| `struct_field_names` | `allow` | a field named after its struct is often the clearest name | medium |
| `too_many_lines` | `allow` | spl's function-lines measures function length, with a limit per crate and expiring exceptions | medium |
| `trivially_copy_pass_by_ref` | `allow` | passing small Copy types by reference is harmless | medium |
| `unnecessary_debug_formatting` | `allow` | paths printed with {:?} quote themselves, which helps with spaces | medium |
| `unreadable_literal` | `allow` | addresses and magic numbers are written as their sources write them | medium |
| `used_underscore_binding` | `allow` | an underscore binding used in one cfg is common | medium |

`clippy.toml` (an integer is a limit the project may lower):

| Setting | Value | Reason | Set in |
|---|---|---|---|
| `allow-print-in-tests` | `true` | a test may print what it saw | medium |
| `allow-unwrap-in-tests` | `true` | a test that unwraps fails at the unwrap, which is what a test should do | small |
| `cognitive-complexity-threshold` | `25` | Clippy's default; spl's function-complexity (cyclomatic, 15) is the tighter limit | medium |

## The file

```toml
# Tier 4: a big workspace (rerun, zed, bevy, intent): layers declared and checked, sizes watched, everything strict.
extends = "medium"

[tier]
description = "A big workspace: a crate map with layers checked edge by edge, size limits with expiring exceptions, strict conformity"

[lints.waiver-unused]
level = "deny"
reason = "every waiver allows something, or it goes"

[lints.package-description]
level = "deny"
reason = "the generated crate table and READMEs are made from it"

[lints.crate-docs]
level = "deny"
reason = "rustdoc's front page of the crate is its crate-level docs"

[lints.default-features-off]
level = "deny"
reason = "feature creep in a big graph costs every build"

[lints.duplicate-versions]
level = "deny"
reason = "two versions of a crate double its build time and split its types"

[lints.rust-version-set]
level = "deny"
reason = "the MSRV is declared so it can be checked"

[lints.toolchain-pinned]
level = "deny"
reason = "every PC and the gate build with the same compiler"

[lints.feature-docs]
level = "warn"
reason = "a feature without a comment cannot be chosen by a reader"

[lints.justfile-simple]
level = "warn"
reason = "recipes call programs; logic lives in Rust where it is tested"

[lints.test-lines]
level = "warn"
reason = "test code builds and links too"

[lints.static-interior-mutability]
level = "deny"
reason = "every piece of mutable state has one owner; a global lock says why on the item"

[lints.lock-held-across]
level = "deny"
reason = "a guard across .await or I/O stalls every other caller for as long as the slowest peer"

[lints.map-complete]
level = "deny"
reason = "the crate map is the source of truth: every crate in it, nothing gone"

[lints.layer-edges]
level = "deny"
reason = "layers are declared and checked edge by edge; move a crate, never reorder layers"

[lints.forbidden-closure]
level = "deny"
reason = "some crates must never end up in a binary, however far away they arrive"

[lints.external-form]
level = "deny"
reason = "outside projects come by git on a branch, listed once, used where their role allows"

[lints.third-party-placement]
level = "deny"
reason = "heavy dependencies sit in one crate each"

[lints.core-crate-deps]
level = "deny"
reason = "a _core crate holds shared types and drags nothing in"

[lints.core-crate-types]
level = "deny"
reason = "_core crates hold only types: no I/O, threads or runtimes"

[lints.crate-table]
level = "deny"
reason = "the crate table is generated from the manifests and the map, never written by hand"

[lints.unsafe-allow-list]
level = "deny"
reason = "unsafe is forbidden except in the crates the map lists"

[lints.todo-macros]
level = "deny"
reason = "no stop-gaps: a todo!() is a panic waiting in shipped code"

[lints.allow-without-reason]
level = "deny"
reason = "an exception to a lint says why, or it outlives its reason"

[lints.float-determinism]
level = "deny"
reason = "crates the map marks deterministic compute the same bits on every platform"

[lints.random-state-hash]
level = "deny"
reason = "crates the map marks deterministic iterate in the same order every run"

[lints.stop-gap-comments]
level = "deny"
reason = "no stop-gaps: a TODO or a 'for now' names its tracker id or goes"

[lints.agent-rules-match-lints]
level = "warn"
reason = "limits AGENTS.md states are the limits the lints check"

[lints.dead-pub]
level = "warn"
reason = "rustc can't see pub items no other crate uses; in a big workspace they pile up (candidates, never fail)"

[lints.pub-surface]
level = "warn"
reason = "in a big workspace pub is the easy way across crates; a library that is all surface keeps nothing apart"

[lints.deny-bans]
level = "deny"
reason = "banned crates (OpenSSL, a second runtime) and duplicate versions stay out of a big graph"

[lints.deny-licenses]
level = "deny"
reason = "every linked licence is one the project accepts, listed once in deny.toml"

[lints.deny-sources]
level = "deny"
reason = "crates come only from the registries and git hosts deny.toml trusts"

[lints.machete-unused-deps]
level = "deny"
reason = "an unused dependency in a big graph costs every build; false positives are listed in the manifest"

[lints.typos-misspelling]
level = "deny"
reason = "names are searched across hundreds of crates; a misspelling hides from every search"

[lints.taplo-fmt]
level = "deny"
reason = "manifests are edited by people and by tools; one TOML formatting keeps their diffs about the change"

[lints.cargo-msrv]
level = "deny"
reason = "the declared rust-version is built, so it stays true"

[lints.cargo-no-std]
level = "deny"
reason = "a crate the map marks no_std is built for a target without std, or the promise breaks unseen"

[lints.cargo-wasm]
level = "deny"
reason = "a crate the map marks wasm is built for wasm32, or the promise breaks unseen"

[lints.semver-checks-breaking]
level = "deny"
reason = "a crate the map marks stable changes its API only with the version bump it needs"

[lints.hakari-generate]
level = "warn"
reason = "a workspace-hack builds each shared dependency once; warn, as the lint needs spl built with the hakari feature"

[lints.hakari-manage-deps]
level = "warn"
reason = "the workspace-hack helps only the members that depend on it; warn, as the lint needs the hakari feature"

# On top of medium's Clippy set: the restriction and nursery lints a big codebase wants.

[clippy.lints]
iter_over_hash_type = { level = "warn", reason = "iteration over a hash map changes order from run to run; random-state-hash reports the type" }
float_cmp_const = { level = "warn", reason = "comparing a float to a constant exactly is almost always a bug" }
lossy_float_literal = { level = "warn", reason = "a float literal that does not fit its type silently becomes another number" }
imprecise_flops = { level = "warn", reason = "hypot, ln_1p and friends are more precise than the formula written out" }
```
