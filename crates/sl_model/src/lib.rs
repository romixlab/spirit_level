//! The project model of a `spirit_level` run.
//!
//! ```no_run
//! let p = sl_model::Project::load(std::path::Path::new(".")).unwrap();
//! for pkg in &p.packages {
//!     println!("{} in {}", pkg.name, pkg.dir);
//! }
//! ```

pub mod config_tree;
pub mod deny;
pub mod git;
pub mod graph;
pub mod justfile;
pub mod lines;
pub mod project;
pub mod source;
pub mod toml_file;
pub mod workflow;
pub mod yaml_file;

pub use config_tree::{ConfigNode, ConfigValue};
pub use deny::{DenyConfig, PackageSpec};
pub use graph::{Resolve, ResolvedPackage};
pub use justfile::Justfile;
pub use lines::{LineCount, count_lines};
pub use project::{
    Dep, DepKind, HAKARI_CONFIG_PATHS, LoadError, Package, Project, TargetKind, command,
    git_url_key,
};
pub use source::{FileKind, SourceFile};
pub use toml_edit;
pub use toml_file::TomlFile;
pub use workflow::Workflow;
pub use yaml_file::YamlFile;
