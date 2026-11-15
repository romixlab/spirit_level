//! A generic configuration tree, parsed from TOML or YAML so both formats are walked the same way.
//!
//! [`crate::TomlFile::tree`] builds one from a `zensical.toml`; [`crate::YamlFile`] builds one directly while
//! parsing an `mkdocs.yml`.

/// One node of the tree, with the line it starts on
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigNode {
    /// 1-based line the node starts on
    pub line: u32,
    /// The node's value
    pub value: ConfigValue,
}

/// A configuration node's value
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValue {
    /// An absent key, or an explicit null
    Null,
    /// A string, integer, float, boolean or datetime, read back as its written text
    Scalar(String),
    /// Entries in file order
    Sequence(Vec<ConfigNode>),
    /// Entries in file order: the key, the line it is written on, and the value
    Mapping(Vec<(String, u32, ConfigNode)>),
}

impl ConfigNode {
    /// The node as a scalar's text
    pub fn as_str(&self) -> Option<&str> {
        match &self.value {
            ConfigValue::Scalar(s) => Some(s),
            _ => None,
        }
    }

    /// The node as a sequence's entries
    pub fn as_sequence(&self) -> Option<&[Self]> {
        match &self.value {
            ConfigValue::Sequence(v) => Some(v),
            _ => None,
        }
    }

    /// The node as a mapping's entries
    pub fn as_mapping(&self) -> Option<&[(String, u32, Self)]> {
        match &self.value {
            ConfigValue::Mapping(v) => Some(v),
            _ => None,
        }
    }

    /// The value at a dotted path of mapping keys
    pub fn get(&self, path: &[&str]) -> Option<&Self> {
        let mut node = self;
        for k in path {
            node = &node.as_mapping()?.iter().find(|(key, _, _)| key == k)?.2;
        }
        Some(node)
    }
}
