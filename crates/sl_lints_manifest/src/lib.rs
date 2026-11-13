//! Manifest and repo-file lints. One analysis, [`MANIFEST`], reports them all: they share nothing heavier than
//! the manifests cargo metadata already read.

mod deps;
mod package;
mod repo;
mod workspace;

use sl_analysis::{Analysis, Context, lint};
use sl_core::{LintMeta, Need, Span};
use sl_model::Package;

lint!(PACKAGE_DESCRIPTION, "package-description", "manifest", "every crate has a description in its Cargo.toml",
    params: [], needs: [], fix: None);
lint!(PACKAGE_LICENSE, "package-license", "manifest", "every crate states its licence",
    params: [], needs: [], fix: None);
lint!(EDITION_LATEST, "edition-latest", "manifest", "every crate is on the current edition",
    params: [("edition", Str, "\"2024\"", "the edition every crate is on")], needs: [], fix: None);
lint!(RUST_VERSION_SET, "rust-version-set", "manifest", "every crate declares its minimum Rust version",
    params: [], needs: [], fix: None);
lint!(PUBLISH_FALSE, "publish-false", "manifest", "workspace crates are publish = false unless listed as published",
    params: [("published", StrList, "[]", "crates that are published (stable API)")], needs: [], fix: None);
lint!(README_PER_CRATE, "readme-per-crate", "manifest", "every crate has a README",
    params: [("file", Str, "\"README.md\"", "the README's file name in the crate directory")], needs: [], fix: None);
lint!(CRATE_DOCS, "crate-docs", "manifest", "every crate root opens with crate-level documentation",
    params: [], needs: [], fix: None);
lint!(WORKSPACE_INHERITANCE, "workspace-inheritance", "workspace", "members inherit the shared package keys from [workspace.package]",
    params: [("keys", StrList, "[\"edition\", \"rust-version\", \"license\", \"repository\"]", "the keys every member inherits")],
    needs: [], fix: Exact);
lint!(WORKSPACE_LINTS, "workspace-lints", "workspace", "the root declares [workspace.lints] and every member inherits them",
    params: [], needs: [], fix: Exact);
lint!(WORKSPACE_DEPENDENCIES, "workspace-dependencies", "workspace", "members take every dependency from [workspace.dependencies]",
    params: [("members", Bool, "true", "also workspace members (path dependencies)")], needs: [], fix: None);
lint!(DEFAULT_FEATURES_OFF, "default-features-off", "dependencies", "third-party dependencies are declared with default-features = false",
    params: [("allow", StrList, "[]", "crates whose default features are wanted")], needs: [], fix: None);
lint!(NO_SYS_CRATES, "no-sys-crates", "dependencies", "the dependency graph has no -sys crates and no C build steps",
    params: [("allow", StrList, "[]", "crates allowed anyway, each named with its reason in AGENTS.md")],
    needs: [Need::Resolve], fix: None);
lint!(DUPLICATE_VERSIONS, "duplicate-versions", "dependencies", "no crate is in the dependency graph in two versions",
    params: [("allow", StrList, "[]", "crates allowed in several versions")], needs: [Need::Resolve], fix: None);
lint!(GIT_DEPENDENCY_PINS, "git-dependency-pins", "dependencies", "git dependencies follow a branch, with no rev or tag pin",
    params: [("forbid", StrList, "[\"rev\", \"tag\"]", "the pins that are refused")], needs: [], fix: None);
lint!(PATH_OUTSIDE_WORKSPACE, "path-outside-workspace", "dependencies", "path dependencies stay inside the workspace",
    params: [], needs: [], fix: None);
lint!(PROFILE_DEFAULTS, "profile-defaults", "repo", "the dev profile splits debug info and a deploy profile builds incrementally",
    params: [("deploy-profile", Str, "\"deploy\"", "the profile `just install` builds with")], needs: [], fix: None);
lint!(TOOLCHAIN_PINNED, "toolchain-pinned", "repo", "the repo pins its toolchain in rust-toolchain.toml",
    params: [], needs: [], fix: None);
lint!(REPO_FILES, "repo-files", "repo", "the repo carries its rule files",
    params: [("files", StrList, "[\"README.md\", \"AGENTS.md\", \"CLAUDE.md\", \"FEATURES.md\", \"CHANGELOG.md\"]", "files that must exist at the root")],
    needs: [], fix: None);
lint!(CHANGELOG_UNRELEASED, "changelog-unreleased", "repo", "CHANGELOG.md has an Unreleased section",
    params: [], needs: [], fix: None);
lint!(JUSTFILE_RECIPES, "justfile-recipes", "repo", "the justfile has the standard recipes",
    params: [("recipes", StrList, "[\"default\", \"test\", \"lint\", \"install\", \"deploy\"]", "recipes that must exist")],
    needs: [], fix: None);
lint!(JUSTFILE_SIMPLE, "justfile-simple", "repo", "justfile recipes only call programs: a few lines, no inline scripts",
    params: [("max-lines", Int, "8", "command lines a recipe may have")], needs: [], fix: None);
lint!(FEATURE_DOCS, "feature-docs", "manifest", "every cargo feature has a comment saying what it turns on",
    params: [("skip", StrList, "[\"default\"]", "features that need no comment")], needs: [], fix: None);
lint!(CRATE_DIR_NAME, "crate-dir-name", "workspace", "a member's directory is named after its crate",
    params: [], needs: [], fix: None);
lint!(MACRO_CRATE_SUFFIX, "macro-crate-suffix", "workspace", "proc-macro crates and only they end in _macros or _derive",
    params: [], needs: [], fix: None);

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
    &TOOLCHAIN_PINNED,
    &REPO_FILES,
    &CHANGELOG_UNRELEASED,
    &JUSTFILE_RECIPES,
    &JUSTFILE_SIMPLE,
    &FEATURE_DOCS,
    &CRATE_DIR_NAME,
    &MACRO_CRATE_SUFFIX,
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
        workspace::run(cx);
        deps::run(cx);
        repo::run(cx);
    }
}

/// The line of a key of a member's manifest; when the key is missing, the line of the nearest table above it
fn at(pkg: &Package, path: &[&str]) -> Span {
    let line = (1..=path.len())
        .rev()
        .map(|n| pkg.manifest.line(&path[..n]))
        .find(|l| *l > 0)
        .unwrap_or(0);
    Span::new(pkg.manifest.rel.clone(), line)
}
