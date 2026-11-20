//! Manifest and repo-file lints. One analysis, [`MANIFEST`], reports them all: they share nothing heavier than
//! the manifests cargo metadata already read.

mod bins;
mod ci;
mod clippy;
mod contract;
mod deps;
mod face;
mod fuzz;
mod macros;
mod package;
mod patch;
mod repo;
mod speed;
mod toolchain;
mod workspace;

use sl_analysis::{Analysis, Context, lint};
use sl_core::{LintMeta, Need, Span};
use sl_model::Package;

lint!(PACKAGE_DESCRIPTION, "M0009", "package-description", "manifest", "every crate has a description in its Cargo.toml",
    params: [], needs: [], fix: None);
lint!(PACKAGE_LICENSE, "M0010", "package-license", "manifest", "every crate states its licence",
    params: [], needs: [], fix: None);
lint!(EDITION_LATEST, "M0007", "edition-latest", "manifest", "every crate is on the current edition",
    params: [("edition", Str, "\"2024\"", "the edition every crate is on")], needs: [], fix: None);
lint!(RUST_VERSION_SET, "M0013", "rust-version-set", "manifest", "every crate declares its minimum Rust version",
    params: [], needs: [], fix: None);
lint!(PUBLISH_FALSE, "M0011", "publish-false", "manifest", "workspace crates are publish = false unless listed as published",
    params: [("published", StrList, "[]", "crates that are published (stable API)")], needs: [], fix: None);
lint!(README_PER_CRATE, "M0012", "readme-per-crate", "manifest", "every crate has a README",
    params: [("file", Str, "\"README.md\"", "the README's file name in the crate directory")], needs: [], fix: None);
lint!(CRATE_DOCS, "M0006", "crate-docs", "manifest", "every crate root opens with crate-level documentation",
    params: [], needs: [], fix: None);
lint!(WORKSPACE_INHERITANCE, "M0023", "workspace-inheritance", "workspace", "members inherit the shared package keys from [workspace.package]",
    params: [("keys", StrList, "[\"edition\", \"rust-version\", \"license\", \"repository\"]", "the keys every member inherits")],
    needs: [], fix: Exact);
lint!(WORKSPACE_LINTS, "M0024", "workspace-lints", "workspace", "the root declares [workspace.lints] and every member inherits them",
    params: [], needs: [], fix: Exact);
lint!(WORKSPACE_DEPENDENCIES, "M0022", "workspace-dependencies", "workspace", "members take every dependency from [workspace.dependencies]",
    params: [("members", Bool, "true", "also workspace members (path dependencies)")], needs: [], fix: None);
lint!(DEFAULT_FEATURES_OFF, "M0001", "default-features-off", "dependencies", "third-party dependencies are declared with default-features = false",
    params: [("allow", StrList, "[]", "crates whose default features are wanted")], needs: [], fix: None);
lint!(NO_SYS_CRATES, "M0004", "no-sys-crates", "dependencies", "the dependency graph has no -sys crates and no C build steps",
    params: [("allow", StrList, "[]", "crates allowed anyway, each named with its reason in AGENTS.md")],
    needs: [Need::Resolve], fix: None);
lint!(DUPLICATE_VERSIONS, "M0002", "duplicate-versions", "dependencies", "no crate is in the dependency graph in two versions",
    params: [], needs: [Need::Resolve], fix: None);
lint!(GIT_DEPENDENCY_PINS, "M0003", "git-dependency-pins", "dependencies", "git dependencies are pinned the way the project's policy says: a branch by default",
    params: [
        ("policy", Choice["branch", "default-branch", "dev-branch", "tag", "release", "any"], "\"branch\"", "how a git dependency on another project's repo is pinned"),
        ("own", StrList, "[]", "git URL prefixes of the project's own repos (`github.com/org`, a host)"),
        ("own-policy", Choice["branch", "default-branch", "dev-branch", "tag", "release", "any"], "\"branch\"", "how a git dependency on one of the project's own repos is pinned"),
        ("dev-branch", Str, "\"dev\"", "the branch the `dev-branch` policy follows"),
    ],
    needs: [], fix: None);
lint!(PATH_OUTSIDE_WORKSPACE, "M0005", "path-outside-workspace", "dependencies", "path dependencies stay inside the workspace",
    params: [], needs: [], fix: None);
lint!(PROFILE_DEFAULTS, "M0017", "profile-defaults", "repo", "the dev profile splits debug info and a deploy profile builds incrementally",
    params: [("deploy-profile", Str, "\"deploy\"", "the profile `just install` builds with")], needs: [], fix: None);
lint!(DEV_DEBUG_OFF, "M0037", "dev-debug-off", "repo", "the dev profile builds with debug info off; a run turns it on with CARGO_PROFILE_DEV_DEBUG",
    params: [("allow", StrList, "[]", "other `profile.dev.debug` values accepted besides false/0/\"none\", each with its reason on the line above in spirit_level.toml")],
    needs: [], fix: Exact);
lint!(TOOLCHAIN_PINNED, "M0019", "toolchain-pinned", "repo", "every rust-toolchain.toml pins the expected toolchain exactly: the stable release or the dated nightly",
    params: [
        ("stable", Str, "\"1.100.0\"", "the stable release every toolchain file pins (`X.Y.Z`); each spl release ships the current one"),
        ("nightly", Str, "\"nightly-2026-11-19\"", "the nightly a toolchain file pins where it needs one (`nightly-YYYY-MM-DD`)"),
        ("skip", StrList, "[\"**/tests/fixtures/**\", \"**/tests/projects/**\"]", "globs of toolchain files not checked: test projects"),
    ],
    needs: [], fix: Exact);
lint!(TOOLCHAIN_OVERRIDES_PINNED, "M0036", "toolchain-overrides-pinned", "repo", "justfiles and CI workflows choose the pinned toolchain: `cargo +nightly-2026-11-19`, never `+nightly` or `+stable`",
    params: [("skip", StrList, "[\"**/tests/fixtures/**\", \"**/tests/projects/**\"]", "globs of justfiles and workflows not checked: test projects")],
    needs: [], fix: Exact);
lint!(REPO_FILES, "M0018", "repo-files", "repo", "the repo carries its rule files",
    params: [("files", StrList, "[\"README.md\", \"AGENTS.md\", \"CLAUDE.md\", \"FEATURES.md\", \"CHANGELOG.md\"]", "files that must exist at the root")],
    needs: [], fix: None);
lint!(CHANGELOG_UNRELEASED, "M0014", "changelog-unreleased", "repo", "CHANGELOG.md has an Unreleased section",
    params: [], needs: [], fix: None);
lint!(JUSTFILE_RECIPES, "M0015", "justfile-recipes", "repo", "the justfile has the standard recipes",
    params: [("recipes", StrList, "[\"default\", \"test\", \"lint\", \"install\", \"deploy\"]", "recipes that must exist")],
    needs: [], fix: None);
lint!(JUSTFILE_SIMPLE, "M0016", "justfile-simple", "repo", "justfile recipes only call programs: a few lines, no inline scripts",
    params: [("max-lines", Int, "8", "command lines a recipe may have")], needs: [], fix: None);
lint!(FEATURE_DOCS, "M0008", "feature-docs", "manifest", "every cargo feature has a comment saying what it turns on",
    params: [("skip", StrList, "[\"default\"]", "features that need no comment")], needs: [], fix: None);
lint!(CRATE_DIR_NAME, "M0020", "crate-dir-name", "workspace", "a member's directory is named after its crate",
    params: [], needs: [], fix: None);
lint!(MACRO_CRATE_SUFFIX, "M0021", "macro-crate-suffix", "workspace", "proc-macro crates and only they end in _macros or _derive",
    params: [], needs: [], fix: None);
lint!(TIER_CLIPPY_SET, "M0025", "tier-clippy-set", "manifest", "the project's [lints.clippy] and clippy.toml turn on the Clippy set its tiers expect, every opt-out with a reason",
    params: [], needs: [], fix: None);
lint!(PROFILE_NO_LTO, "M0026", "profile-no-lto", "repo", "the profiles of the edit-build-test loop and the deploy profile build without LTO",
    params: [("profiles", StrList, "[\"dev\", \"test\", \"deploy\"]", "profiles that build without LTO, inherited settings included")],
    needs: [], fix: None);
lint!(PATCH_OUTSIDE_WORKSPACE, "M0027", "patch-outside-workspace", "dependencies", "no committed [patch] or [replace] override takes a path outside the workspace",
    params: [], needs: [], fix: None);
lint!(TEST_BINARIES, "M0028", "test-binaries", "repo", "a crate builds its integration tests as one binary",
    params: [("max-binaries", Int, "1", "integration-test binaries a crate may have")], needs: [], fix: None);
lint!(MACRO_COMPILE_FAIL, "M0029", "macro-compile-fail", "workspace", "every proc-macro crate has compile-fail tests",
    params: [("harnesses", StrList, "[\"trybuild\", \"ui_test\"]", "crates that run compile-fail cases")], needs: [], fix: None);
lint!(CONTRACT_SNAPSHOT, "M0030", "contract-snapshot", "workspace", "a contract crate exports its version and keeps every version's schema snapshot in git, unchanged once in the base",
    params: [
        ("const", Str, "\"VERSION\"", "the integer constant of the library root that holds the contract's version"),
        ("snapshot", Str, "\"schema/v{version}.json\"", "the snapshot of a version, in the crate's directory; `{version}` is the number"),
        ("base", Str, "\"main\"", "the branch work merges into; a snapshot it has must not change"),
    ],
    needs: [Need::Map], fix: None);
lint!(FUZZ_CORPUS_TEST, "M0031", "fuzz-corpus-test", "repo", "every fuzz target has a corpus tracked in git and a test that replays it",
    params: [], needs: [], fix: None);
lint!(CI_WORKFLOW, "M0032", "ci-workflow", "repo", "the repo has a CI workflow and it runs the repo's checks",
    params: [("path", Str, "\".github/workflows/ci.yml\"", "the CI workflow, relative to the root")], needs: [], fix: None);
lint!(MANIFEST_HOMEPAGE, "M0033", "manifest-homepage", "manifest", "the shared package keys set repository, and homepage is the book's site_url when it has one",
    params: [], needs: [], fix: Exact);
lint!(README_BADGES, "M0034", "readme-badges", "repo", "the README carries a badge for the book, CI and every published crate, each linking to its target",
    params: [("file", Str, "\"README.md\"", "the README, relative to the root"),
             ("workflow", Str, "\".github/workflows/ci.yml\"", "the CI workflow whose status badge the README shows")],
    needs: [], fix: Exact);
lint!(BINARY_NAME_CHARS, "M0035", "binary-name-chars", "manifest", "every binary's name, explicit or implicit, uses only an allowed set of characters",
    params: [("chars", Str, "\"a-z0-9\"", "characters a binary name may use: literals and a-z-style ranges")],
    needs: [], fix: None);

/// The manifest analysis
pub struct Manifest;

/// The registered instance
pub static MANIFEST: Manifest = Manifest;

static LINTS: &[&LintMeta] = &[
    &PACKAGE_DESCRIPTION,
    &PACKAGE_LICENSE,
    &EDITION_LATEST,
    &RUST_VERSION_SET,
    &PUBLISH_FALSE,
    &README_PER_CRATE,
    &CRATE_DOCS,
    &WORKSPACE_INHERITANCE,
    &WORKSPACE_LINTS,
    &WORKSPACE_DEPENDENCIES,
    &DEFAULT_FEATURES_OFF,
    &NO_SYS_CRATES,
    &DUPLICATE_VERSIONS,
    &GIT_DEPENDENCY_PINS,
    &PATH_OUTSIDE_WORKSPACE,
    &PROFILE_DEFAULTS,
    &DEV_DEBUG_OFF,
    &TOOLCHAIN_PINNED,
    &TOOLCHAIN_OVERRIDES_PINNED,
    &REPO_FILES,
    &CHANGELOG_UNRELEASED,
    &JUSTFILE_RECIPES,
    &JUSTFILE_SIMPLE,
    &FEATURE_DOCS,
    &CRATE_DIR_NAME,
    &MACRO_CRATE_SUFFIX,
    &TIER_CLIPPY_SET,
    &PROFILE_NO_LTO,
    &PATCH_OUTSIDE_WORKSPACE,
    &TEST_BINARIES,
    &MACRO_COMPILE_FAIL,
    &CONTRACT_SNAPSHOT,
    &FUZZ_CORPUS_TEST,
    &CI_WORKFLOW,
    &MANIFEST_HOMEPAGE,
    &README_BADGES,
    &BINARY_NAME_CHARS,
];

impl Analysis for Manifest {
    fn name(&self) -> &'static str {
        "manifest"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        package::run(cx);
        bins::run(cx);
        workspace::run(cx);
        deps::run(cx);
        repo::run(cx);
        toolchain::run(cx);
        clippy::run(cx);
        speed::run(cx);
        patch::run(cx);
        macros::run(cx);
        contract::run(cx);
        fuzz::run(cx);
        ci::run(cx);
        face::run(cx);
    }
}

/// The key of a member's manifest; when the key is missing, the nearest table above it, else the whole file
fn at(pkg: &Package, path: &[&str]) -> Span {
    (1..=path.len())
        .rev()
        .map(|n| pkg.manifest.span(&path[..n]))
        .find(|s| s.line > 0)
        .unwrap_or_else(|| Span::new(pkg.manifest.rel.clone(), 0))
}
