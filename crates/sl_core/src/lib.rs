//! The types every `spirit_level` crate shares.
//!
//! - [`LintMeta`]: what a lint is (code, id, category, summary, documentation, parameters, needs, fix kind, source);
//!   every lint declares one as a `static`, and the book, `explain` and the configuration checks read it.
//! - [`Level`], [`ParamSpec`], [`ParamValue`], [`Params`]: how a lint is configured.
//! - [`Diagnostic`], [`Span`], [`Metric`], [`Fix`]: what a lint reports.
//! - [`Date`], [`Expiry`]: expiries of overrides and waivers.
//! - [`glob`]: the path patterns of scoped exceptions.
//! - [`TaskClass`], [`Classes`]: the task classes of firmware modules.

pub mod class;
pub mod date;
pub mod diagnostic;
pub mod glob;
pub mod meta;
pub mod params;

pub use class::{Classes, TaskClass};
pub use date::{Date, Expiry, Version};
pub use diagnostic::{Confidence, Diagnostic, Edit, Fix, Metric, Span};
pub use meta::{
    CODE_GROUPS, FixKind, Level, LintMeta, Need, PINNED_NIGHTLY, PINNED_TOOLCHAIN, Source, ToolUse,
    valid_code,
};
pub use params::{ParamKind, ParamSpec, ParamValue, Params};
