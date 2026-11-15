//! The project model of a `spirit_level` run.
//!
//! ```no_run
//! let p = sl_model::Project::load(std::path::Path::new(".")).unwrap();
//! for pkg in &p.packages {
//!     println!("{} in {}", pkg.name, pkg.dir);
//! }
//! ```

pub mod config_tree;
pub mod git;
pub mod lines;
pub mod project;
pub mod source;
pub mod toml_file;
pub mod yaml_file;

pub use config_tree::{ConfigNode, ConfigValue};
pub use lines::{LineCount, count_lines};
pub use project::{
    Dep, DepKind, HAKARI_CONFIG_PATHS, LoadError, Package, Project, Resolve, ResolvedPackage,
    TargetKind, command, git_url_key,
};
pub use source::{FileKind, SourceFile};
pub use toml_edit;
pub use toml_file::TomlFile;
pub use yaml_file::YamlFile;
