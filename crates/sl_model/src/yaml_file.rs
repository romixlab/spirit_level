//! A YAML file (`mkdocs.yml`) read into the generic [`ConfigNode`] tree, so a mapping key or a sequence entry
//! keeps the line it is written on. Built on yaml-rust2's event parser: there is no ready-made marked tree, so
//! [`Builder`] assembles one from [`Event`]s, resolving anchors and aliases as it goes. A tag this crate does not
//! know (`!ENV`, `!!python/name:...`) changes nothing: YAML tags are metadata on top of the plain, quoted or block
//! scalar actually written, which the parser still reports.

use crate::{ConfigNode, ConfigValue};
use yaml_rust2::Event;
use yaml_rust2::parser::{MarkedEventReceiver, Parser, Tag};
use yaml_rust2::scanner::{Marker, TScalarStyle};

/// A YAML file of the project
#[derive(Debug)]
pub struct YamlFile {
    /// Path relative to the project root
    pub rel: String,
    /// The text as read
    pub text: String,
    /// The document's top-level node
    pub root: ConfigNode,
}

impl YamlFile {
    /// Parses `text`; the error carries the line
    pub fn parse(rel: impl Into<String>, text: String) -> Result<Self, String> {
        let rel = rel.into();
        let mut builder = Builder::default();
        if let Err(e) = Parser::new_from_str(&text).load(&mut builder, false) {
            let line = e.marker().line();
            return Err(format!("{rel}:{line}: {e}"));
        }
        let root = builder.root.unwrap_or(ConfigNode {
            line: 1,
            value: ConfigValue::Null,
        });
        Ok(Self { rel, text, root })
    }

    /// The node at a dotted path of mapping keys
    pub fn get(&self, path: &[&str]) -> Option<&ConfigNode> {
        self.root.get(path)
    }
}

/// A node still being filled in, one level of nesting
enum Frame {
    Sequence(u32, Vec<ConfigNode>),
    /// Entries closed so far, and the key (text, line) waiting for its value
    Mapping(u32, Vec<(String, u32, ConfigNode)>, Option<(String, u32)>),
}

/// Builds a [`ConfigNode`] tree from the parser's events, one open [`Frame`] per nesting level
#[derive(Default)]
struct Builder {
    stack: Vec<(Frame, usize)>,
    anchors: std::collections::BTreeMap<usize, ConfigNode>,
    root: Option<ConfigNode>,
}

impl MarkedEventReceiver for Builder {
    fn on_event(&mut self, ev: Event, mark: Marker) {
        let line = mark.line() as u32;
        match ev {
            Event::Nothing
            | Event::StreamStart
            | Event::StreamEnd
            | Event::DocumentStart
            | Event::DocumentEnd => {}
            Event::SequenceStart(aid, _) => {
                self.stack.push((Frame::Sequence(line, Vec::new()), aid))
            }
            Event::MappingStart(aid, _) => self
                .stack
                .push((Frame::Mapping(line, Vec::new(), None), aid)),
            Event::SequenceEnd => {
                if let Some((Frame::Sequence(line, items), aid)) = self.stack.pop() {
                    self.close(
                        ConfigNode {
                            line,
                            value: ConfigValue::Sequence(items),
                        },
                        aid,
                    );
                }
            }
            Event::MappingEnd => {
                if let Some((Frame::Mapping(line, entries, _), aid)) = self.stack.pop() {
                    self.close(
                        ConfigNode {
                            line,
                            value: ConfigValue::Mapping(entries),
                        },
                        aid,
                    );
                }
            }
            Event::Scalar(text, style, aid, tag) => {
                let node = ConfigNode {
                    line,
                    value: scalar_value(text, style, tag.as_ref()),
                };
                self.close(node, aid);
            }
            Event::Alias(id) => {
                let node = self.anchors.get(&id).cloned().unwrap_or(ConfigNode {
                    line,
                    value: ConfigValue::Null,
                });
                self.attach(node);
            }
        }
    }
}

impl Builder {
    /// A node (scalar, or a sequence/mapping just closed) is complete: remember it under its anchor, then attach
    /// it to whatever is open above it
    fn close(&mut self, node: ConfigNode, anchor_id: usize) {
        if anchor_id > 0 {
            self.anchors.insert(anchor_id, node.clone());
        }
        self.attach(node);
    }

    /// Attaches a completed node to the open sequence or mapping above it, or sets it as the document's root when
    /// nothing is open
    fn attach(&mut self, node: ConfigNode) {
        match self.stack.last_mut() {
            None => self.root = Some(node),
            Some((Frame::Sequence(_, items), _)) => items.push(node),
            Some((Frame::Mapping(_, entries, pending), _)) => match pending.take() {
                Some((key, line)) => entries.push((key, line, node)),
                None => {
                    // a key must be a scalar: a mapping or sequence used as a key has no dotted name, so it is
                    // dropped rather than guessed at
                    if let ConfigValue::Scalar(key) = node.value {
                        *pending = Some((key, node.line));
                    }
                }
            },
        }
    }
}

/// A plain, untagged scalar spelling YAML's null (`~`, `null`, or nothing after the key) reads as no value, same
/// as a TOML key that was never written; everything else keeps its written text
fn scalar_value(text: String, style: TScalarStyle, tag: Option<&Tag>) -> ConfigValue {
    let is_plain_null = style == TScalarStyle::Plain
        && tag.is_none()
        && matches!(text.as_str(), "" | "~" | "null" | "Null" | "NULL");
    if is_plain_null {
        ConfigValue::Null
    } else {
        ConfigValue::Scalar(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapping_sequence_and_lines() {
        let f = YamlFile::parse(
            "mkdocs.yml",
            "site_name: x\nnav:\n  - Home: index.md\n  - Guide:\n      - guide/a.md\n".into(),
        )
        .unwrap();
        assert_eq!(
            f.get(&["site_name"]).and_then(ConfigNode::as_str),
            Some("x")
        );
        let nav = f.get(&["nav"]).and_then(ConfigNode::as_sequence).unwrap();
        assert_eq!(nav.len(), 2);
        let home = nav[0].as_mapping().unwrap();
        assert_eq!(home[0].0, "Home");
        assert_eq!(home[0].1, 3);
        assert_eq!(home[0].2.as_str(), Some("index.md"));
        let guide = nav[1].as_mapping().unwrap();
        assert_eq!(guide[0].1, 4);
        assert_eq!(
            guide[0].2.as_sequence().unwrap()[0].as_str(),
            Some("guide/a.md")
        );
    }

    #[test]
    fn null_and_anchors() {
        let f = YamlFile::parse("mkdocs.yml", "a: &x [1, 2]\nb: *x\nc:\nd: ~\n".into()).unwrap();
        assert_eq!(
            f.get(&["b"])
                .and_then(ConfigNode::as_sequence)
                .map(<[_]>::len),
            Some(2)
        );
        assert_eq!(f.get(&["c"]).map(|n| &n.value), Some(&ConfigValue::Null));
        assert_eq!(f.get(&["d"]).map(|n| &n.value), Some(&ConfigValue::Null));
    }

    #[test]
    fn unknown_tag_still_parses() {
        let f = YamlFile::parse("mkdocs.yml", "extra:\n  version: !ENV VERSION\n".into()).unwrap();
        assert_eq!(
            f.get(&["extra", "version"]).and_then(ConfigNode::as_str),
            Some("VERSION")
        );
    }

    #[test]
    fn parse_error_has_line() {
        let e = YamlFile::parse("mkdocs.yml", "a: [1, 2\n".into()).unwrap_err();
        assert!(e.starts_with("mkdocs.yml:"), "{e}");
    }
}
