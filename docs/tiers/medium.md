# medium

A medium workspace: internal crates unpublished, MSRV declared, toolchain pinned, dependency rules strict.

Extends [small-workspace](small-workspace.md): everything below is on top of it.

Use it with `extends = "medium"` in `spirit_level.toml`.

## Lints enabled (with what it extends)

| Code | Lint | Level | Reason | Set in |
|---|---|---|---|---|
| [C0002](../lints/C0002.md) | allow-without-reason | `warn` | an exception to a lint says why, or it outlives its reason | small |
| [C0003](../lints/C0003.md) | cli-subprocess | `deny` | components call each other as libraries, not through their CLIs | medium |
| [C0005](../lints/C0005.md) | panic-in-library | `warn` | a library returns errors; its caller decides what is fatal | medium |
| [C0006](../lints/C0006.md) | print-in-library | `warn` | printing is the binary's choice | small |
| [C0008](../lints/C0008.md) | todo-macros | `warn` | no stop-gaps: a todo!() is a panic waiting in shipped code | small |
| [C0010](../lints/C0010.md) | error-enum-growth | `warn` | a flat error enum grows a variant per failure and loses the source errors; wrap them by layer | medium |
| [C0011](../lints/C0011.md) | module-graph-scc | `warn` | a module cycle holding most of a crate can't be split into crates; cut it while it is small | medium |
| [C0012](../lints/C0012.md) | waiver-invalid | `deny` | a waiver without a reason, for an unknown lint, or expired hides a finding without saying why | small |
| [C0013](../lints/C0013.md) | waiver-unused | `warn` | a waiver that allows nothing is left over from changed code | small |
| [C0014](../lints/C0014.md) | build-info-env | `warn` | build info compiled into libraries recompiles them on every commit; the binary sets it once | medium |
| [C0015](../lints/C0015.md) | test-support-gated | `warn` | fakes and test helpers never ship: cfg(test), a test-support feature from dev-dependencies, or a test crate | medium |
| [C0016](../lints/C0016.md) | orphan-source-file | `warn` | a file no mod reaches is never compiled or tested, yet reads like live code | medium |
| [C0017](../lints/C0017.md) | sized-ints-in-contracts | `warn` | a contract is read on every party; usize is 32 bits on some of them | medium |
| [G0001](../lints/G0001.md) | core-crate-deps | `warn` | a _core crate holds shared types and drags nothing in | medium |
| [G0003](../lints/G0003.md) | crate-cycles | `deny` | crates in a cycle cannot be built or tested apart | medium |
| [M0001](../lints/M0001.md) | default-features-off | `warn` | dependencies are added with default-features = false and only the features in use | small |
| [M0002](../lints/M0002.md) | duplicate-versions | `warn` | two versions of a crate double its build time and split its types | small-workspace |
| [M0003](../lints/M0003.md) | git-dependency-pins | `deny` | git dependencies follow a branch; a pin hides a break until it is old | medium |
| [M0004](../lints/M0004.md) | no-sys-crates | `deny` | pure-Rust crates only: no bundled C, no -sys bindings without a written reason | small |
| [M0005](../lints/M0005.md) | path-outside-workspace | `deny` | a path into a sibling checkout builds differently on every PC | small |
| [M0006](../lints/M0006.md) | crate-docs | `warn` | rustdoc's front page of the crate is its crate-level docs | small |
| [M0007](../lints/M0007.md) | edition-latest | `warn` | new code starts on the current edition | small |
| [M0009](../lints/M0009.md) | package-description | `warn` | the crate's one-line description is written once, in Cargo.toml; tables and READMEs are made from it | small |
| [M0010](../lints/M0010.md) | package-license | `warn` | a crate without a licence cannot be reused or published | small |
| [M0011](../lints/M0011.md) | publish-false | `warn` | internal crates are a build unit, not an API; only listed crates are published | medium |
| [M0012](../lints/M0012.md) | readme-per-crate | `deny` | every crate of a workspace says what it is and what it may depend on | small-workspace |
| [M0013](../lints/M0013.md) | rust-version-set | `warn` | the MSRV is declared so it can be checked | medium |
| [M0014](../lints/M0014.md) | changelog-unreleased | `warn` | work lands under [Unreleased]; the release step turns it into a version section | small |
| [M0015](../lints/M0015.md) | justfile-recipes | `warn` | every repo answers to the same recipes: default, test, lint, install, deploy | small |
| [M0017](../lints/M0017.md) | profile-defaults | `warn` | split debug info and an incremental deploy profile: measured large build-time wins | small |
| [M0018](../lints/M0018.md) | repo-files | `warn` | every work repo carries README, AGENTS, CLAUDE, FEATURES and CHANGELOG | small |
| [M0019](../lints/M0019.md) | toolchain-pinned | `warn` | every PC and the gate build with the same compiler | medium |
| [M0020](../lints/M0020.md) | crate-dir-name | `warn` | flat crates/<name>: the folder is the crate | small-workspace |
| [M0021](../lints/M0021.md) | macro-crate-suffix | `warn` | _macros and _derive mean proc macros and nothing else | small-workspace |
| [M0022](../lints/M0022.md) | workspace-dependencies | `deny` | one version per dependency per workspace, written once in [workspace.dependencies] | medium |
| [M0023](../lints/M0023.md) | workspace-inheritance | `deny` | edition, rust-version, licence and repository are declared once in [workspace.package] | small-workspace |
| [M0024](../lints/M0024.md) | workspace-lints | `deny` | every crate inherits [workspace.lints], so one table is the policy | small-workspace |
| [M0025](../lints/M0025.md) | tier-clippy-set | `deny` | the Clippy set is the lint policy of every crate; a gap in it is a gap in the policy | medium |
| [M0026](../lints/M0026.md) | profile-no-lto | `warn` | LTO in the loop's profiles turns a link of a second into minutes; it belongs in a release profile | small |
| [M0027](../lints/M0027.md) | patch-outside-workspace | `deny` | a committed patch into a sibling checkout builds differently on every machine; it lives in an ignored .cargo/config.toml | small |
| [M0028](../lints/M0028.md) | test-binaries | `warn` | each integration-test binary links the library again; one tests/it binary links it once | medium |
| [M0029](../lints/M0029.md) | macro-compile-fail | `warn` | a proc macro must refuse bad input with a clear error, and only compile-fail cases show that | medium |
| [P0001](../lints/P0001.md) | agent-file-lines | `warn` | agents read AGENTS.md whole every session; a long one is skimmed, not followed | small |
| [P0007](../lints/P0007.md) | commit-subject-length | `warn` | the subject is what one-line logs and reviews show | medium |
| [P0009](../lints/P0009.md) | changelog-entry-length | `warn` | a CHANGELOG is a list: one short line per change | small |
| [P0017](../lints/P0017.md) | fixes-tag | `warn` | a Fixes: trailer that points nowhere is worse than none; it fires only where one is written | medium |
| [S0001](../lints/S0001.md) | crate-lines | `warn` | split a crate before it passes ~20,000 lines: one crate is one serial compile step | medium |
| [S0002](../lints/S0002.md) | function-complexity | `warn` | more paths than can be tested or held in one's head | small |
| [S0003](../lints/S0003.md) | function-lines | `warn` | a function longer than a screen is changed without being read whole | small |
| [S0004](../lints/S0004.md) | module-lines | `warn` | a file of thousands of lines holds several subjects and collects every edit | medium |
| [T0001](../lints/T0001.md) | lock-held-across | `warn` | a guard across .await or I/O stalls every other caller for as long as the slowest peer | small |
| [T0002](../lints/T0002.md) | lock-unwrap | `warn` | the count of lock().unwrap() per crate shows where shared state concentrates | medium |
| [T0003](../lints/T0003.md) | nested-locks | `warn` | two locks at once need one lock order everywhere | small-workspace |
| [T0004](../lints/T0004.md) | static-interior-mutability | `warn` | every piece of mutable state has one owner; a global lock says why | small |
| [T0005](../lints/T0005.md) | static-mut | `deny` | global mutable state with no synchronisation; there is always a better owner | small |
| [T0006](../lints/T0006.md) | thread-local-state | `warn` | thread-local state is a hidden side channel between functions | small-workspace |
| [X0005](../lints/X0005.md) | cargo-doc | `deny` | the docs of internal crates are the map of the code; they build without a warning | medium |
| [X0006](../lints/X0006.md) | cargo-msrv | `warn` | the declared rust-version is built, so it stays true | medium |
| [X0019](../lints/X0019.md) | deny-advisories | `deny` | a vulnerable dependency is updated, or its advisory ignored with a reason in deny.toml | medium |
| [X0021](../lints/X0021.md) | deny-licenses | `warn` | every linked licence is one the project accepts, listed once in deny.toml | medium |
| [X0026](../lints/X0026.md) | machete-unused-deps | `warn` | an unused dependency costs build time, audit and lockfile churn for nothing | small |
| [X0031](../lints/X0031.md) | taplo-fmt | `warn` | manifests are edited by people and by tools; one TOML formatting keeps their diffs about the change | small-workspace |
| [X0032](../lints/X0032.md) | typos-misspelling | `warn` | a misspelled identifier is copied to every call site; typos has almost no false positives | small |

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
| `get_unwrap` | `warn` | `.get(i).unwrap()` is indexing with a worse message: index, or handle the None | small |
| `let_underscore_must_use` | `warn` | `let _ =` on a must_use value drops an error unseen | small |
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
| `cognitive-complexity-threshold` | `20` | set on samples: 17 read fine, 21 and 25 were too long; spl's function-complexity (cyclomatic, 15) counts paths | medium |
| `max-fn-params-bools` | `1` | two bool parameters are a call site nobody can read; an enum or a struct says what each means | medium |

## The file

```toml
# Tier 3: a medium workspace (wire_weaver's size): internal crates, MSRV, pinned toolchain.
extends = "small-workspace"

[tier]
description = "A medium workspace: internal crates unpublished, MSRV declared, toolchain pinned, dependency rules strict"

[lints.rust-version-set]
level = "warn"
reason = "the MSRV is declared so it can be checked"

[lints.publish-false]
level = "warn"
reason = "internal crates are a build unit, not an API; only listed crates are published"

[lints.toolchain-pinned]
level = "warn"
reason = "every PC and the gate build with the same compiler"

[lints.workspace-dependencies]
level = "deny"
reason = "one version per dependency per workspace, written once in [workspace.dependencies]"

[lints.git-dependency-pins]
level = "deny"
reason = "git dependencies follow a branch; a pin hides a break until it is old"

[lints.module-lines]
level = "warn"
reason = "a file of thousands of lines holds several subjects and collects every edit"

[lints.crate-lines]
level = "warn"
reason = "split a crate before it passes ~20,000 lines: one crate is one serial compile step"

[lints.lock-unwrap]
level = "warn"
reason = "the count of lock().unwrap() per crate shows where shared state concentrates"

[lints.crate-cycles]
level = "deny"
reason = "crates in a cycle cannot be built or tested apart"

[lints.core-crate-deps]
level = "warn"
reason = "a _core crate holds shared types and drags nothing in"

[lints.panic-in-library]
level = "warn"
reason = "a library returns errors; its caller decides what is fatal"

[lints.cli-subprocess]
level = "deny"
reason = "components call each other as libraries, not through their CLIs"

[lints.commit-subject-length]
level = "warn"
reason = "the subject is what one-line logs and reviews show"

[lints.fixes-tag]
level = "warn"
reason = "a Fixes: trailer that points nowhere is worse than none; it fires only where one is written"

[lints.module-graph-scc]
level = "warn"
reason = "a module cycle holding most of a crate can't be split into crates; cut it while it is small"


[lints.error-enum-growth]
level = "warn"
reason = "a flat error enum grows a variant per failure and loses the source errors; wrap them by layer"

[lints.deny-advisories]
level = "deny"
reason = "a vulnerable dependency is updated, or its advisory ignored with a reason in deny.toml"

[lints.deny-licenses]
level = "warn"
reason = "every linked licence is one the project accepts, listed once in deny.toml"

[lints.cargo-msrv]
level = "warn"
reason = "the declared rust-version is built, so it stays true"

[lints.cargo-doc]
level = "deny"
reason = "the docs of internal crates are the map of the code; they build without a warning"

[lints.tier-clippy-set]
level = "deny"
reason = "the Clippy set is the lint policy of every crate; a gap in it is a gap in the policy"

[lints.test-binaries]
level = "warn"
reason = "each integration-test binary links the library again; one tests/it binary links it once"

[lints.build-info-env]
level = "warn"
reason = "build info compiled into libraries recompiles them on every commit; the binary sets it once"

[lints.test-support-gated]
level = "warn"
reason = "fakes and test helpers never ship: cfg(test), a test-support feature from dev-dependencies, or a test crate"

[lints.macro-compile-fail]
level = "warn"
reason = "a proc macro must refuse bad input with a clear error, and only compile-fail cases show that"

[lints.orphan-source-file]
level = "warn"
reason = "a file no mod reaches is never compiled or tested, yet reads like live code"

[lints.sized-ints-in-contracts]
level = "warn"
reason = "a contract is read on every party; usize is 32 bits on some of them"

# On top of small's Clippy set: pedantic and cargo, with the opt-outs rerun found to be noise (each with its reason),
# and the restriction and nursery lints a workspace of this size wants.

[clippy.lints]
pedantic = { level = "warn", reason = "the stricter idioms; the opt-outs below are the ones that are noise for most code" }
cargo = { level = "warn", reason = "manifest hygiene: feature names, wildcard versions" }
allow_attributes = { level = "warn", reason = "#[expect] fails once the lint stops firing, so an exception cannot outlive its cause" }
print_stdout = { level = "warn", reason = "printing is the binary's choice, written in one place" }
print_stderr = { level = "warn", reason = "printing is the binary's choice, written in one place" }
str_to_string = { level = "warn", reason = "to_owned says what happens: a copy of a borrowed str" }
use_self = { level = "warn", reason = "Self follows a rename" }
self_named_module_files = { level = "warn", reason = "one module layout, mod.rs, across the workspace" }
too_long_first_doc_paragraph = { level = "warn", reason = "the first doc paragraph is the summary rustdoc lists; one short sentence" }
cognitive_complexity = { level = "warn", reason = "a function too tangled to follow in one read; complements spl's cyclomatic function-complexity" }
# opt-outs of pedantic
assigning_clones = { level = "allow", reason = "clone_into reads backwards" }
cast_lossless = { level = "allow", reason = "a widening `as` cast loses nothing, and From is longer for no gain" }
cast_possible_truncation = { level = "allow", reason = "too noisy: most casts are of values known to fit" }
cast_precision_loss = { level = "allow", reason = "too noisy: integer to float for display and statistics is the common case" }
cast_sign_loss = { level = "allow", reason = "too noisy: most casts are of values known to be non-negative" }
default_trait_access = { level = "allow", reason = "Default::default() is clear where the type is written beside it" }
float_cmp = { level = "allow", reason = "exact float comparisons are usually intended; float_cmp_const stays on in big" }
implicit_hasher = { level = "allow", reason = "a BuildHasher type parameter on public functions is not worth it" }
inline_always = { level = "allow", reason = "#[inline(always)] is written where it was measured" }
items_after_statements = { level = "allow", reason = "a helper item next to its only use reads better" }
many_single_char_names = { level = "allow", reason = "r, g, b and x, y, z are fine" }
map_unwrap_or = { level = "allow", reason = "`.map(f).unwrap_or(x)` often reads better than map_or" }
missing_errors_doc = { level = "allow", reason = "the error type documents its cases" }
missing_panics_doc = { level = "allow", reason = "too noisy: most panics are invariants, not inputs" }
must_use_candidate = { level = "allow", reason = "thousands of hits; must_use goes where ignoring the value is a bug" }
needless_raw_string_hashes = { level = "allow", reason = "r#\"...\"# keeps a string editable when a quote is added" }
redundant_closure_for_method_calls = { level = "allow", reason = "a closure is often clearer than a path to a method" }
return_self_not_must_use = { level = "allow", reason = "thousands of hits on builders" }
similar_names = { level = "allow", reason = "too many false positives: encoder and encoded are different things" }
struct_excessive_bools = { level = "allow", reason = "a struct of flags is often just that" }
struct_field_names = { level = "allow", reason = "a field named after its struct is often the clearest name" }
trivially_copy_pass_by_ref = { level = "allow", reason = "passing small Copy types by reference is harmless" }
unnecessary_debug_formatting = { level = "allow", reason = "paths printed with {:?} quote themselves, which helps with spaces" }
unreadable_literal = { level = "allow", reason = "addresses and magic numbers are written as their sources write them" }
used_underscore_binding = { level = "allow", reason = "an underscore binding used in one cfg is common" }
too_many_lines = { level = "allow", reason = "spl's function-lines measures function length, with a limit per crate and expiring exceptions" }
# opt-outs of cargo
cargo_common_metadata = { level = "allow", reason = "keywords and categories matter only for published crates; package-description covers the rest" }
multiple_crate_versions = { level = "allow", reason = "spl's duplicate-versions checks this, with a reasoned allow list" }

[clippy.config]
cognitive-complexity-threshold = { value = 20, reason = "set on samples: 17 read fine, 21 and 25 were too long; spl's function-complexity (cyclomatic, 15) counts paths" }
allow-print-in-tests = { value = true, reason = "a test may print what it saw" }
max-fn-params-bools = { value = 1, reason = "two bool parameters are a call site nobody can read; an enum or a struct says what each means" }
```
