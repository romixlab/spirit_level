//! The weighted module graph of one crate: nodes are modules cut to a depth, an edge counts the references from
//! one node's code into another's.

use sl_analysis::api::{Names, Target, use_leaves};
use sl_analysis::crate_tree::CrateTree;
use sl_analysis::syntax::{Tok, end_line, is_test, line, test_ranges, tokens};
use sl_model::SourceFile;
use std::collections::{BTreeMap, HashMap};
use syn::Item;

/// One node: the modules under one path of the chosen depth
pub(crate) struct Node {
    /// `a::b`, or `(root)` for the crate root's own code
    pub name: String,
    /// Code lines of its modules
    pub code: u32,
}

/// The references from one node into another
pub(crate) struct Edge {
    /// How many
    pub refs: u32,
    /// The first one: file and line
    pub first: (String, u32),
}

/// The graph
pub(crate) struct Graph {
    pub nodes: Vec<Node>,
    pub edges: BTreeMap<(usize, usize), Edge>,
}

/// The graph of `tree` with nodes at `depth` (1: top-level modules)
pub(crate) fn build(tree: &CrateTree<'_>, depth: usize) -> Graph {
    let names = Names::new(tree);
    let mut b = Builder::new(tree, depth);
    let aliases = b.use_refs(&names);
    let mut files: Vec<_> = tree.modules.iter().map(|m| m.file).collect();
    files.sort_by_key(|f| f.rel.clone());
    files.dedup_by_key(|f| f.rel.clone());
    for file in files {
        b.scan_file(&names, &aliases, file);
    }
    b.g
}

struct Builder<'t, 'a> {
    tree: &'t CrateTree<'a>,
    node_of: Vec<usize>,
    g: Graph,
}

impl<'t, 'a> Builder<'t, 'a> {
    /// Maps every module to its node and sums the nodes' code lines
    fn new(tree: &'t CrateTree<'a>, depth: usize) -> Self {
        let mut ids: HashMap<String, usize> = HashMap::new();
        let mut nodes: Vec<Node> = Vec::new();
        let mut node_of = Vec::with_capacity(tree.modules.len());
        for m in &tree.modules {
            let segs = m.segments();
            let name = if segs.is_empty() {
                "(root)".to_owned()
            } else {
                segs[..segs.len().min(depth.max(1))].join("::")
            };
            let id = *ids.entry(name.clone()).or_insert_with(|| {
                nodes.push(Node { name, code: 0 });
                nodes.len() - 1
            });
            nodes[id].code += m.code;
            node_of.push(id);
        }
        Builder {
            tree,
            node_of,
            g: Graph {
                nodes,
                edges: BTreeMap::new(),
            },
        }
    }

    fn add(&mut self, from: usize, to: usize, file: &str, line: u32) {
        let (a, b) = (self.node_of[from], self.node_of[to]);
        if a != b {
            let e = self.g.edges.entry((a, b)).or_insert_with(|| Edge {
                refs: 0,
                first: (file.to_owned(), line),
            });
            e.refs += 1;
        }
    }

    /// One reference per `use` leaf into the crate; returns the names each module's `use` items bring in
    fn use_refs(&mut self, names: &Names<'_, '_>) -> Vec<HashMap<String, Target>> {
        let tree = self.tree;
        let mut aliases: Vec<HashMap<String, Target>> = vec![HashMap::new(); tree.modules.len()];
        for (i, m) in tree.modules.iter().enumerate() {
            for item in m.items {
                let Item::Use(u) = item else { continue };
                if is_test(&u.attrs) {
                    continue;
                }
                for l in use_leaves(&u.tree) {
                    if let Some(t) = names.resolve(i, &l.segs) {
                        self.add(i, t.module(), &m.file.rel, line(u));
                        if !l.glob && l.name != "_" {
                            aliases[i].insert(l.name, t);
                        }
                    }
                }
            }
        }
        aliases
    }

    /// Paths into the crate and uses of imported names in one file's code, macro bodies included; test code and
    /// module-level `use` items (counted by [`Builder::use_refs`]) skipped
    fn scan_file(
        &mut self,
        names: &Names<'_, '_>,
        aliases: &[HashMap<String, Target>],
        file: &SourceFile,
    ) {
        let (Ok(ast), Some(toks)) = (file.ast(), tokens(&file.text)) else {
            return;
        };
        let mods = &self.tree.modules;
        let here: Vec<(usize, (u32, u32))> = mods
            .iter()
            .enumerate()
            .filter(|(_, m)| std::ptr::eq(m.file, file))
            .map(|(i, m)| (i, m.lines))
            .collect();
        let mut skip = test_ranges(ast);
        for (i, _) in &here {
            for item in mods[*i].items {
                if let Item::Use(u) = item {
                    skip.push((line(u), end_line(u)));
                }
            }
        }
        let module_at = |l: u32| {
            here.iter()
                .filter(|(_, (a, b))| *a <= l && l <= *b)
                .min_by_key(|(_, (a, b))| b - a)
                .map(|(i, _)| *i)
        };
        let mut i = 0;
        while i < toks.len() {
            let (Tok::Ident(s), at) = &toks[i] else {
                i += 1;
                continue;
            };
            let counted = (i == 0 || toks[i - 1].0 != Tok::PathSep)
                && !skip.iter().any(|(a, b)| a <= at && at <= b);
            let Some(m) = module_at(*at).filter(|_| counted) else {
                i += 1;
                continue;
            };
            let (segs, end, grouped) = path_at(&toks, i);
            let starts_path = segs.len() > 1
                && (matches!(s.as_str(), "crate" | "self" | "super") || mods[m].child(s).is_some());
            let target = if starts_path {
                // `crate::{a, b}` in a function body: the group is not followed here
                names.resolve(m, &segs).filter(|_| !grouped)
            } else {
                aliases[m].get(s).cloned()
            };
            if let Some(t) = &target {
                self.add(m, t.module(), &file.rel, *at);
            }
            i = if starts_path || target.is_some() {
                end
            } else {
                i + 1
            };
        }
    }
}

/// The path starting at token `i`: its segments, the index after it, and whether `::{` follows
fn path_at(toks: &[(Tok, u32)], i: usize) -> (Vec<String>, usize, bool) {
    let mut segs = Vec::new();
    if let Tok::Ident(s) = &toks[i].0 {
        segs.push(s.clone());
    }
    let mut j = i + 1;
    while j + 1 < toks.len() && toks[j].0 == Tok::PathSep {
        let Tok::Ident(n) = &toks[j + 1].0 else { break };
        segs.push(n.clone());
        j += 2;
    }
    let grouped = toks.get(j).is_some_and(|t| t.0 == Tok::PathSep);
    (segs, j, grouped)
}
