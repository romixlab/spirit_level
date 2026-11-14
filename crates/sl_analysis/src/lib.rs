//! The [`Analysis`] trait and its [`Context`].
//!
//! ```
//! use sl_analysis::{Analysis, Context};
//! use sl_core::{Diagnostic, FixKind, LintMeta, Source};
//!
//! static NO_EMPTY: LintMeta = LintMeta {
//!     code: "M0099",
//!     id: "no-empty-workspace",
//!     category: "manifest",
//!     summary: "a workspace has at least one member",
//!     docs: "",
//!     params: &[],
//!     needs: &[],
//!     fix: FixKind::None,
//!     source: Source::Native,
//! };
//!
//! static LINTS: &[&LintMeta] = &[&NO_EMPTY];
//!
//! struct Members;
//! impl Analysis for Members {
//!     fn name(&self) -> &'static str { "members" }
//!     fn lints(&self) -> &'static [&'static LintMeta] { LINTS }
//!     fn run(&self, cx: &Context<'_>) {
//!         if cx.project.packages.is_empty() {
//!             cx.emit(Diagnostic::new("no-empty-workspace", "workspace", "the workspace has no members"));
//!         }
//!     }
//! }
//! ```

pub mod api;
pub mod comments;
pub mod crate_tree;
pub mod syntax;

use sl_config::{Config, Map};
use sl_core::{Date, Diagnostic, LintMeta, Params};
use sl_model::{Package, Project};
use std::cell::RefCell;
use std::path::PathBuf;

/// One analysis: a family of lints computed together (they share a parse, a graph, a tool run)
pub trait Analysis: Sync {
    /// A short name for reports and timings (`manifest`, `clippy`)
    fn name(&self) -> &'static str;
    /// The lints it reports
    fn lints(&self) -> &'static [&'static LintMeta];
    /// Emits its findings through `cx`. Called only when at least one of its lints is enabled and that lint's
    /// needs are met; [`Context::enabled`] says which
    fn run(&self, cx: &Context<'_>);
}

/// What an analysis sees during a run
pub struct Context<'a> {
    /// The project
    pub project: &'a Project,
    /// The resolved configuration
    pub config: &'a Config,
    /// Today, for anything dated (expiries are checked by the configuration already)
    pub today: Date,
    runnable: Vec<&'static str>,
    elf: &'a [PathBuf],
    out: RefCell<Vec<Diagnostic>>,
    skipped: RefCell<Vec<Skipped>>,
}

/// A lint an analysis could not run after all, found out while running (a tool that failed, a toolchain or
/// target that is not installed); the engine reports it as not run, like a missing need
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Skipped {
    /// The lint
    pub lint: String,
    /// What was missing, in the form of [`sl_core::Need::name`] (`tool:cargo-deny`, `toolchain:1.88`)
    pub need: String,
    /// Why, and what would provide it
    pub reason: String,
}

/// What an analysis produced: its findings and the lints it could not run
#[derive(Debug, Default)]
pub struct Outcome {
    /// The findings
    pub diagnostics: Vec<Diagnostic>,
    /// The lints it could not run
    pub skipped: Vec<Skipped>,
}

impl<'a> Context<'a> {
    /// A context in which the lints `runnable` may report
    pub fn new(
        project: &'a Project,
        config: &'a Config,
        today: Date,
        runnable: Vec<&'static str>,
    ) -> Self {
        Context {
            project,
            config,
            today,
            runnable,
            elf: &[],
            out: RefCell::new(Vec::new()),
            skipped: RefCell::new(Vec::new()),
        }
    }

    /// The same context with the linked ELFs given for the run (`--elf`)
    pub fn with_elf(mut self, elf: &'a [PathBuf]) -> Self {
        self.elf = elf;
        self
    }

    /// The linked ELFs given for the run; lints that read them declare [`sl_core::Need::Elf`]
    pub fn elf(&self) -> &'a [PathBuf] {
        self.elf
    }

    /// Whether a lint runs in this run (enabled, its needs met, selected)
    pub fn enabled(&self, id: &str) -> bool {
        self.runnable.contains(&id)
    }

    /// A lint's parameters
    pub fn params(&self, id: &str) -> &Params {
        &self.config.lint(id).params
    }

    /// A lint's parameters for one crate (with the exceptions naming it)
    pub fn params_for(&self, id: &str, pkg: &Package) -> Params {
        self.config.params_for(id, &pkg.name)
    }

    /// The crate map; lints that need it declare [`sl_core::Need::Map`], so it is there when they run
    pub fn map(&self) -> Option<&'a Map> {
        self.config.map.as_ref()
    }

    /// Reports a finding; one of a lint that is not running is dropped, so an analysis may compute everything
    /// and emit freely
    pub fn emit(&self, d: Diagnostic) {
        if self.enabled(&d.lint) {
            self.out.borrow_mut().push(d);
        }
    }

    /// Reports that a running lint could not run after all; its findings are dropped, so a half-run lint never
    /// reports a partial result as the whole
    pub fn not_run(&self, lint: &str, need: impl Into<String>, reason: impl Into<String>) {
        if self.enabled(lint) && !self.skipped.borrow().iter().any(|s| s.lint == lint) {
            self.skipped.borrow_mut().push(Skipped {
                lint: lint.to_owned(),
                need: need.into(),
                reason: reason.into(),
            });
        }
    }

    /// Reports that every running lint of this analysis could not run after all (its tool failed)
    pub fn not_run_all(&self, need: &str, reason: &str) {
        for id in self.runnable.clone() {
            self.not_run(id, need, reason);
        }
    }

    /// The findings and the lints that could not run
    pub fn take(self) -> Outcome {
        let skipped = self.skipped.into_inner();
        let mut diagnostics = self.out.into_inner();
        diagnostics.retain(|d| !skipped.iter().any(|s| s.lint == d.lint));
        Outcome {
            diagnostics,
            skipped,
        }
    }
}

/// Declares a native lint's `static LintMeta`, its documentation read from `docs/<id>.md` of the calling crate.
/// The code comes first; it is the lint's row in `crates/spirit_level/codes.toml`.
///
/// ```ignore
/// sl_analysis::lint!(CRATE_LINES, "S0001", "crate-lines", "size", "a crate's src/ stays under its line limit",
///     params: [("max-lines", Int, "20000", "code lines above which a crate is reported")],
///     needs: [], fix: None);
/// ```
///
/// An adapter of an external tool names the tool and how it is used after the summary:
///
/// ```ignore
/// sl_analysis::lint!(DENY_BANS, "X0020", "deny-bans", "tools", "no banned crate, no duplicate version deny.toml refuses",
///     tool: ("cargo-deny", Executable), params: [], needs: [Need::Tool("cargo-deny")], fix: None);
/// ```
#[macro_export]
macro_rules! lint {
    ($name:ident, $code:literal, $id:literal, $cat:literal, $summary:literal, tool: ($tool:literal, $use:ident),
     params: [$(($p:literal, $k:ident, $d:literal, $doc:literal)),* $(,)?],
     needs: [$($need:expr),* $(,)?], fix: $fix:ident) => {
        #[doc = concat!("`", $code, "` `", $id, "`: ", $summary)]
        pub static $name: $crate::reexport::LintMeta = $crate::reexport::LintMeta {
            code: $code,
            id: $id,
            category: $cat,
            summary: $summary,
            docs: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/", $id, ".md")),
            params: &[$($crate::reexport::ParamSpec {
                name: $p,
                kind: $crate::reexport::ParamKind::$k,
                default: $d,
                doc: $doc,
            }),*],
            needs: &[$($need),*],
            fix: $crate::reexport::FixKind::$fix,
            source: $crate::reexport::Source::Tool {
                name: $tool,
                r#use: $crate::reexport::ToolUse::$use,
            },
        };
    };
    ($name:ident, $code:literal, $id:literal, $cat:literal, $summary:literal,
     params: [$(($p:literal, $k:ident, $d:literal, $doc:literal)),* $(,)?],
     needs: [$($need:expr),* $(,)?], fix: $fix:ident) => {
        #[doc = concat!("`", $code, "` `", $id, "`: ", $summary)]
        pub static $name: $crate::reexport::LintMeta = $crate::reexport::LintMeta {
            code: $code,
            id: $id,
            category: $cat,
            summary: $summary,
            docs: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/", $id, ".md")),
            params: &[$($crate::reexport::ParamSpec {
                name: $p,
                kind: $crate::reexport::ParamKind::$k,
                default: $d,
                doc: $doc,
            }),*],
            needs: &[$($need),*],
            fix: $crate::reexport::FixKind::$fix,
            source: $crate::reexport::Source::Native,
        };
    };
}

/// Paths the [`lint!`] macro uses
#[doc(hidden)]
pub mod reexport {
    pub use sl_core::{FixKind, LintMeta, ParamKind, ParamSpec, Source, ToolUse};
}
