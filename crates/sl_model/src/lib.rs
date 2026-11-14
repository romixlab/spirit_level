//! The project model of a `spirit_level` run.
//!
//! ```no_run
//! let p = sl_model::Project::load(std::path::Path::new(".")).unwrap();
//! for pkg in &p.packages {
//!     println!("{} in {}", pkg.name, pkg.dir);
//! }
//! ```

pub mod lines;
pub mod project;
pub mod source;
pub mod toml_file;

pub use lines::{LineCount, count_lines};
pub use project::{
    Dep, DepKind, LoadError, Package, Project, Resolve, ResolvedPackage, TargetKind, command,
};
pub use source::{FileKind, SourceFile};
pub use toml_edit;
pub use toml_file::TomlFile;
