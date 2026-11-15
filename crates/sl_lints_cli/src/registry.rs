//! How a CLI's commands come together, read from its sources: elements that register a command themselves
//! (inventory, linkme), and the `#[derive(Subcommand)]` enums that list commands in one place.

use sl_analysis::Context;
use sl_analysis::syntax::{line, without_tests};
use sl_model::{DepKind, FileKind, Package};
use std::collections::BTreeSet;
use syn::visit::Visit;

/// What the sources of a CLI and of the workspace crates it uses show
#[derive(Debug, Default)]
pub(crate) struct Registry {
    /// The first registration element: (file, line, what)
    pub element: Option<(String, u32, &'static str)>,
    /// The `#[derive(Subcommand)]` enum with the most variants: (file, line, name, variants)
    pub central: Option<(String, u32, String, usize)>,
}

/// Reads the sources of `pkg` and of every workspace member it reaches through normal dependencies
pub(crate) fn scan(cx: &Context<'_>, pkg: &Package) -> Registry {
    let mut out = Registry::default();
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut stack: Vec<&Package> = vec![pkg];
    while let Some(p) = stack.pop() {
        if !seen.insert(p.name.as_str()) {
            continue;
        }
        let crates = registry_crates(p);
        for f in cx
            .project
            .sources(p)
            .iter()
            .filter(|f| f.kind == FileKind::Src)
        {
            let Ok(ast) = f.ast() else { continue };
            let mut v = Visitor {
                crates: &crates,
                element: None,
                central: None,
            };
            v.visit_file(&without_tests(ast));
            if out.element.is_none()
                && let Some((l, what)) = v.element
            {
                out.element = Some((f.rel.clone(), l, what));
            }
            if let Some((l, name, n)) = v.central
                && out.central.as_ref().is_none_or(|c| n > c.3)
            {
                out.central = Some((f.rel.clone(), l, name, n));
            }
        }
        stack.extend(
            p.deps
                .iter()
                .filter(|d| d.member && d.kind == DepKind::Normal)
                .filter_map(|d| cx.project.package(&d.name)),
        );
    }
    out
}

/// The registry crates a package depends on: (the name its paths use, a rename included; the crate's name)
fn registry_crates(p: &Package) -> Vec<(String, String)> {
    p.deps
        .iter()
        .filter(|d| d.kind == DepKind::Normal && REGISTRIES.iter().any(|(c, _, _)| *c == d.name))
        .map(|d| {
            (
                d.rename.as_deref().unwrap_or(&d.name).replace('-', "_"),
                d.name.clone(),
            )
        })
        .collect()
}

/// The registry crates: the crate, the item whose use registers one entry, what that is called in a message
const REGISTRIES: &[(&str, &str, &str)] = &[
    ("inventory", "submit", "an inventory::submit! element"),
    (
        "linkme",
        "distributed_slice",
        "a linkme distributed_slice element",
    ),
];

struct Visitor<'c> {
    /// From [`registry_crates`]
    crates: &'c [(String, String)],
    /// The first element: (line, what)
    element: Option<(u32, &'static str)>,
    /// The derive(Subcommand) enum with the most variants: (line, name, variants)
    central: Option<(u32, String, usize)>,
}

impl Visitor<'_> {
    /// Whether a path names `item` of the registry crate `krate`: `krate::item` (or through a rename), or a bare
    /// `item` in a package that depends on `krate`
    fn names(&self, path: &syn::Path, krate: &str, item: &str) -> bool {
        if path.segments.last().is_none_or(|s| s.ident != item) {
            return false;
        }
        let depends = |used: Option<&str>| {
            self.crates
                .iter()
                .any(|(n, c)| c == krate && used.is_none_or(|u| u == n))
        };
        match path.segments.first().map(|s| s.ident.to_string()) {
            Some(first) if path.segments.len() > 1 => first == krate || depends(Some(&first)),
            _ => depends(None),
        }
    }

    /// The registration a path names, as [`REGISTRIES`] calls it
    fn registration(&self, path: &syn::Path, items: &[&str]) -> Option<&'static str> {
        REGISTRIES
            .iter()
            .find(|(krate, item, _)| items.contains(item) && self.names(path, krate, item))
            .map(|(_, _, what)| *what)
    }
}

impl<'ast> Visit<'ast> for Visitor<'_> {
    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        if let Some(what) = self.registration(&m.path, &["submit"]) {
            self.element.get_or_insert((line(&m.path), what));
        }
        syn::visit::visit_macro(self, m);
    }

    fn visit_attribute(&mut self, a: &'ast syn::Attribute) {
        // `#[distributed_slice]` declares the slice; `#[distributed_slice(NAME)]` adds an element to it
        if matches!(a.meta, syn::Meta::List(_))
            && let Some(what) = self.registration(a.path(), &["distributed_slice"])
        {
            self.element.get_or_insert((line(a.path()), what));
        }
    }

    fn visit_item_enum(&mut self, e: &'ast syn::ItemEnum) {
        let subcommand = e.attrs.iter().any(|a| {
            a.path().is_ident("derive")
                && a.meta.require_list().is_ok_and(|l| {
                    l.tokens
                        .to_string()
                        .split([',', ' ', ':'])
                        .any(|t| t == "Subcommand")
                })
        });
        let n = e.variants.len();
        if subcommand && self.central.as_ref().is_none_or(|c| n > c.2) {
            self.central = Some((line(&e.ident), e.ident.to_string(), n));
        }
        syn::visit::visit_item_enum(self, e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The element a file shows to a package that depends on `crates` (used name, crate name)
    fn element(crates: &[(&str, &str)], src: &str) -> Option<&'static str> {
        let crates: Vec<(String, String)> = crates
            .iter()
            .map(|(n, c)| ((*n).to_owned(), (*c).to_owned()))
            .collect();
        let mut v = Visitor {
            crates: &crates,
            element: None,
            central: None,
        };
        v.visit_file(&without_tests(&syn::parse_file(src).unwrap()));
        v.element.map(|e| e.1)
    }

    #[test]
    fn finds_registrations() {
        let inv = [("inventory", "inventory")];
        assert!(element(&[], "inventory::submit! { C }").is_some());
        assert!(element(&inv, "use inventory::submit; submit! { C }").is_some());
        assert!(element(&[("inv", "inventory")], "inv::submit! { C }").is_some());
        assert!(
            element(
                &[],
                "#[linkme::distributed_slice(COMMANDS)] static C: Cmd = Cmd;"
            )
            .is_some()
        );
        assert!(element(&inv, "fn main() { inventory::submit! { C } }").is_some());
    }

    #[test]
    fn ignores_what_only_looks_like_one() {
        assert_eq!(element(&[], "submit! { C }"), None);
        assert_eq!(element(&[], "form::submit! { C }"), None);
        assert_eq!(
            element(&[], "#[linkme::distributed_slice] static C: [Cmd];"),
            None
        );
        assert_eq!(
            element(
                &[],
                "// inventory::submit! { C }\nconst S: &str = \"inventory::submit!\";"
            ),
            None
        );
        assert_eq!(
            element(&[], "#[cfg(test)] mod tests { inventory::submit! { C } }"),
            None
        );
    }

    #[test]
    fn central_is_the_largest_subcommand_enum() {
        let src = "#[derive(Subcommand)] enum Group { A, B }\n#[derive(Debug)] enum Colour { A, B, C, D }\n#[derive(clap::Subcommand)] enum Cmd { A, B, C }";
        let mut v = Visitor {
            crates: &[],
            element: None,
            central: None,
        };
        v.visit_file(&syn::parse_file(src).unwrap());
        assert_eq!(v.central, Some((3, "Cmd".to_owned(), 3)));
    }
}
