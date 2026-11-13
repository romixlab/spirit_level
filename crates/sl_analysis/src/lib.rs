//! The [`Analysis`] trait and its [`Context`].
//!
//! ```
//! use sl_analysis::{Analysis, Context};
//! use sl_core::{Diagnostic, FixKind, LintMeta, Source};
//!
//! static NO_EMPTY: LintMeta = LintMeta {
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

pub mod comments;
pub mod syntax;

use sl_config::{Config, Map};
use sl_core::{Date, Diagnostic, LintMeta, Params};
use sl_model::{Package, Project};
use std::cell::RefCell;

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
    out: RefCell<Vec<Diagnostic>>,
}

impl<'a> Context<'a> {
    /// A context in which the lints `runnable` may report
    pub fn new(
        project: &'a Project,
        config: &'a Config,
        today: Date,
        runnable: Vec<&'static str>,
    ) -> Context<'a> {
        Context {
            project,
            config,
            today,
            runnable,
            out: RefCell::new(Vec::new()),
        }
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

    /// The findings emitted so far
    pub fn take(self) -> Vec<Diagnostic> {
        self.out.into_inner()
    }
}

/// Declares a native lint's `static LintMeta`, its documentation read from `docs/<id>.md` of the calling crate
///
/// ```ignore
/// sl_analysis::lint!(CRATE_LINES, "crate-lines", "size", "a crate's src/ stays under its line limit",
///     params: [("max-lines", Int, "20000", "code lines above which a crate is reported")],
///     needs: [], fix: None);
/// ```
#[macro_export]
macro_rules! lint {
    ($name:ident, $id:literal, $cat:literal, $summary:literal,
     params: [$(($p:literal, $k:ident, $d:literal, $doc:literal)),* $(,)?],
     needs: [$($need:expr),* $(,)?], fix: $fix:ident) => {
        #[doc = concat!("`", $id, "`: ", $summary)]
        pub static $name: $crate::reexport::LintMeta = $crate::reexport::LintMeta {
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
    pub use sl_core::{FixKind, LintMeta, ParamKind, ParamSpec, Source};
}
