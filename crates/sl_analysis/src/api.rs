//! Names inside one crate: what a `use` brings into a module, how a path resolves to a module of the crate, and
//! the public items a library offers (reachable from its root through `pub mod` and `pub use`). Read from the
//! syntax alone: no macro expansion, no type information.

use crate::crate_tree::CrateTree;
use crate::syntax::{is_test, item_attrs, line};
use std::collections::{BTreeSet, HashMap};
use syn::{Item, UseTree, Visibility};

/// One leaf of a `use` tree
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UseLeaf {
    /// The full path (`crate`, `a`, `B`); for a glob the path of what is globbed
    pub segs: Vec<String>,
    /// The name it binds in the module (the last segment or the `as` name); empty for a glob
    pub name: String,
    /// `use x::*`
    pub glob: bool,
}

/// Every leaf of a `use` item (`use a::{b, c::d as e, f::*}` gives three)
pub fn use_leaves(tree: &UseTree) -> Vec<UseLeaf> {
    let mut out = Vec::new();
    leaves(tree, &mut Vec::new(), &mut out);
    out
}

fn leaves(tree: &UseTree, prefix: &mut Vec<String>, out: &mut Vec<UseLeaf>) {
    match tree {
        UseTree::Path(p) => {
            prefix.push(p.ident.to_string());
            leaves(&p.tree, prefix, out);
            prefix.pop();
        }
        UseTree::Name(n) => {
            let name = n.ident.to_string();
            if name == "self" {
                if let Some(last) = prefix.last() {
                    out.push(UseLeaf {
                        segs: prefix.clone(),
                        name: last.clone(),
                        glob: false,
                    });
                }
            } else {
                let mut segs = prefix.clone();
                segs.push(name.clone());
                out.push(UseLeaf {
                    segs,
                    name,
                    glob: false,
                });
            }
        }
        UseTree::Rename(r) => {
            let mut segs = prefix.clone();
            if r.ident != "self" {
                segs.push(r.ident.to_string());
            }
            out.push(UseLeaf {
                segs,
                name: r.rename.to_string(),
                glob: false,
            });
        }
        UseTree::Glob(_) => out.push(UseLeaf {
            segs: prefix.clone(),
            name: String::new(),
            glob: true,
        }),
        UseTree::Group(g) => {
            for t in &g.items {
                leaves(t, prefix, out);
            }
        }
    }
}

/// What a path names inside the crate
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    /// A module
    Module(usize),
    /// An item (or something else not a module) called `name` in a module
    Item(usize, String),
}

impl Target {
    /// The module it is in (or is)
    pub fn module(&self) -> usize {
        match self {
            Target::Module(m) | Target::Item(m, _) => *m,
        }
    }
}

/// The names of one crate: each module's non-glob imports
pub struct Names<'t, 'a> {
    tree: &'t CrateTree<'a>,
    imports: Vec<HashMap<String, Vec<String>>>,
}

impl<'t, 'a> Names<'t, 'a> {
    /// Reads every module's `use` items (test items skipped)
    pub fn new(tree: &'t CrateTree<'a>) -> Names<'t, 'a> {
        let imports = tree
            .modules
            .iter()
            .map(|m| {
                let mut map = HashMap::new();
                for item in m.items {
                    if let Item::Use(u) = item
                        && !is_test(&u.attrs)
                    {
                        for l in use_leaves(&u.tree) {
                            if !l.glob && l.name != "_" {
                                map.insert(l.name, l.segs);
                            }
                        }
                    }
                }
                map
            })
            .collect();
        Names { tree, imports }
    }

    /// The tree
    pub fn tree(&self) -> &'t CrateTree<'a> {
        self.tree
    }

    /// Whether `name` is imported into module `m` by a `use`
    pub fn imported(&self, m: usize, name: &str) -> bool {
        self.imports[m].contains_key(name)
    }

    /// What a path written in module `from` names inside the crate; `None` for another crate, the prelude or a
    /// name the syntax cannot place. `use` re-exports are followed to where the item is defined
    pub fn resolve(&self, from: usize, segs: &[String]) -> Option<Target> {
        self.resolve_depth(from, segs, 0)
    }

    fn resolve_depth(&self, from: usize, segs: &[String], depth: u32) -> Option<Target> {
        if depth > 8 || segs.is_empty() {
            return None;
        }
        let mods = &self.tree.modules;
        let (mut cur, mut i) = match segs[0].as_str() {
            "crate" => (0, 1),
            "self" => (from, 1),
            "super" => {
                let mut cur = from;
                let mut i = 0;
                while segs.get(i).is_some_and(|s| s == "super") {
                    cur = mods[cur].parent?;
                    i += 1;
                }
                (cur, i)
            }
            first => {
                if let Some(c) = mods[from].child(first) {
                    (c, 1)
                } else {
                    let p = self.imports[from].get(first)?;
                    if p.len() == 1 && p[0] == first {
                        return None;
                    }
                    match self.resolve_depth(from, p, depth + 1)? {
                        Target::Module(t) => (t, 1),
                        t @ Target::Item(..) => return Some(t),
                    }
                }
            }
        };
        while i < segs.len() {
            let s = &segs[i];
            if let Some(c) = mods[cur].child(s) {
                cur = c;
            } else if let Some(p) = self.imports[cur].get(s) {
                match self.resolve_depth(cur, p, depth + 1) {
                    Some(Target::Module(t)) => cur = t,
                    Some(t @ Target::Item(..)) => return Some(t),
                    None => return Some(Target::Item(cur, s.clone())),
                }
            } else {
                return Some(Target::Item(cur, s.clone()));
            }
            i += 1;
        }
        Some(Target::Module(cur))
    }
}

/// A public item of a library: declared `pub` and reachable from the crate root
pub struct PubItem<'a> {
    /// The module it is declared in
    pub module: usize,
    /// Its name
    pub name: String,
    /// `fn`, `struct`, `enum`, `trait`, `type`, `const`, `static`, `union`, `macro`
    pub kind: &'static str,
    /// The item
    pub item: &'a Item,
    /// Its first line
    pub line: u32,
}

/// The name and kind of an item that can be public (`None` for impls, uses, modules and the like)
fn item_name(item: &Item) -> Option<(String, &'static str)> {
    let (ident, kind) = match item {
        Item::Fn(x) => (&x.sig.ident, "fn"),
        Item::Struct(x) => (&x.ident, "struct"),
        Item::Enum(x) => (&x.ident, "enum"),
        Item::Trait(x) => (&x.ident, "trait"),
        Item::Type(x) => (&x.ident, "type"),
        Item::Const(x) => (&x.ident, "const"),
        Item::Static(x) => (&x.ident, "static"),
        Item::Union(x) => (&x.ident, "union"),
        Item::Macro(x) => (x.ident.as_ref()?, "macro"),
        _ => return None,
    };
    Some((ident.to_string(), kind))
}

/// Whether an item is declared plain `pub` (an exported `macro_rules!` counts)
fn is_pub(item: &Item) -> bool {
    let vis = match item {
        Item::Fn(x) => &x.vis,
        Item::Struct(x) => &x.vis,
        Item::Enum(x) => &x.vis,
        Item::Trait(x) => &x.vis,
        Item::Type(x) => &x.vis,
        Item::Const(x) => &x.vis,
        Item::Static(x) => &x.vis,
        Item::Union(x) => &x.vis,
        Item::Macro(x) => {
            return x.attrs.iter().any(|a| a.path().is_ident("macro_export"));
        }
        _ => return false,
    };
    matches!(vis, Visibility::Public(_))
}

/// The public items of a crate: `pub` items of modules reachable from the root through `pub mod`, `pub use` of a
/// module and `pub use m::*`, plus `pub` items re-exported by name (`pub use inner::Thing`) and exported macros
pub fn public_items<'a>(names: &Names<'_, 'a>) -> Vec<PubItem<'a>> {
    let tree = names.tree();
    let mods = &tree.modules;
    let mut reach: BTreeSet<usize> = BTreeSet::from([0]);
    let mut named: BTreeSet<(usize, String)> = BTreeSet::new();
    loop {
        let before = (reach.len(), named.len());
        for m in reach.clone() {
            for (_, c) in &mods[m].children {
                if mods[*c].public {
                    reach.insert(*c);
                }
            }
            for item in mods[m].items {
                let Item::Use(u) = item else { continue };
                if is_test(&u.attrs) || !matches!(u.vis, Visibility::Public(_)) {
                    continue;
                }
                for l in use_leaves(&u.tree) {
                    match names.resolve(m, &l.segs) {
                        Some(Target::Module(t)) if l.glob => {
                            // the glob's pub items count through `named`, its pub modules through reach
                            for it in mods[t].items {
                                if is_pub(it)
                                    && let Some((n, _)) = item_name(it)
                                {
                                    named.insert((t, n));
                                }
                            }
                            for (_, c) in &mods[t].children {
                                if mods[*c].public {
                                    reach.insert(*c);
                                }
                            }
                        }
                        Some(Target::Module(t)) => {
                            reach.insert(t);
                        }
                        Some(Target::Item(t, n)) => {
                            named.insert((t, n));
                        }
                        None => {}
                    }
                }
            }
        }
        if (reach.len(), named.len()) == before {
            break;
        }
    }
    let mut out = Vec::new();
    for (i, m) in mods.iter().enumerate() {
        for item in m.items {
            if is_test(item_attrs(item)) || !is_pub(item) {
                continue;
            }
            let Some((name, kind)) = item_name(item) else {
                continue;
            };
            let exported_macro = kind == "macro";
            if exported_macro || reach.contains(&i) || named.contains(&(i, name.clone())) {
                out.push(PubItem {
                    module: i,
                    name,
                    kind,
                    item,
                    line: line(item),
                });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leaves_of(src: &str) -> Vec<(String, String, bool)> {
        let u: syn::ItemUse = syn::parse_str(src).unwrap();
        use_leaves(&u.tree)
            .into_iter()
            .map(|l| (l.segs.join("::"), l.name, l.glob))
            .collect()
    }

    #[test]
    fn use_trees() {
        assert_eq!(
            leaves_of("use crate::{a::{self, B}, c as d, e::*};"),
            vec![
                ("crate::a".into(), "a".into(), false),
                ("crate::a::B".into(), "B".into(), false),
                ("crate::c".into(), "d".into(), false),
                ("crate::e".into(), String::new(), true),
            ]
        );
    }
}
