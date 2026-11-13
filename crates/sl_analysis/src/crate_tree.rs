//! The module trees of a package's crates, resolved from each crate root by following `mod` declarations (with
//! `#[path]`) and inline modules, the way rustc finds them. Test-only modules are left out.

use crate::syntax::{end_line, is_test, item_attrs, line};
use sl_model::{FileKind, Package, Project, SourceFile, count_lines};
use syn::{Item, Visibility};

/// What kind of crate a tree is
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrateKind {
    /// `src/lib.rs`
    Lib,
    /// `src/main.rs` or `src/bin/*`
    Bin,
}

/// One crate of a package: its root file and every module reached from it
pub struct CrateTree<'a> {
    /// The crate's name: the library's name, or the binary's
    pub name: String,
    /// Library or binary
    pub kind: CrateKind,
    /// Its modules; `modules[0]` is the root
    pub modules: Vec<Module<'a>>,
}

/// One module: a file, or an inline `mod x { ... }` inside one
pub struct Module<'a> {
    /// Its path inside the crate (`a::b`, empty for the root)
    pub path: String,
    /// The file it is in
    pub file: &'a SourceFile,
    /// Its items, test items included (filter with [`is_test`])
    pub items: &'a [Item],
    /// Its parent module, `None` for the root
    pub parent: Option<usize>,
    /// Its child modules, by name
    pub children: Vec<(String, usize)>,
    /// Declared `pub mod`
    pub public: bool,
    /// The lines it covers in its file: the whole file, or the inline module's braces
    pub lines: (u32, u32),
    /// Code lines of its own items, without test items and without its inline child modules
    pub code: u32,
}

impl Module<'_> {
    /// The path's segments (none for the root)
    pub fn segments(&self) -> Vec<&str> {
        if self.path.is_empty() {
            Vec::new()
        } else {
            self.path.split("::").collect()
        }
    }

    /// Its child module called `name`
    pub fn child(&self, name: &str) -> Option<usize> {
        self.children
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, i)| *i)
    }
}

impl CrateTree<'_> {
    /// Code lines of every module
    pub fn code(&self) -> u32 {
        self.modules.iter().map(|m| m.code).sum()
    }

    /// The module with this path
    pub fn find(&self, path: &str) -> Option<usize> {
        self.modules.iter().position(|m| m.path == path)
    }

    /// The innermost module of `file` that covers `line`
    pub fn module_at(&self, file: &SourceFile, line: u32) -> Option<usize> {
        self.modules
            .iter()
            .enumerate()
            .filter(|(_, m)| std::ptr::eq(m.file, file) && m.lines.0 <= line && line <= m.lines.1)
            .min_by_key(|(_, m)| m.lines.1 - m.lines.0)
            .map(|(i, _)| i)
    }
}

/// The crates of a package (library first, then binaries), each with its module tree. Files that fail to parse
/// are left out with their modules
pub fn crates<'a>(project: &'a Project, pkg: &'a Package) -> Vec<CrateTree<'a>> {
    let sources = project.sources(pkg);
    let mut roots: Vec<&SourceFile> = sources
        .iter()
        .filter(|f| {
            f.kind == FileKind::Src
                && (f.is_crate_root()
                    || (f.in_package.starts_with("src/bin/")
                        && f.in_package.ends_with("/main.rs")
                        && f.in_package.matches('/').count() == 3))
        })
        .collect();
    roots.sort_by_key(|f| (f.in_package != "src/lib.rs", f.in_package.clone()));
    roots
        .into_iter()
        .filter_map(|root| {
            let (name, kind) = match root.in_package.as_str() {
                "src/lib.rs" => (lib_name(pkg), CrateKind::Lib),
                "src/main.rs" => (pkg.name.clone(), CrateKind::Bin),
                p => {
                    let stem = p
                        .trim_start_matches("src/bin/")
                        .trim_end_matches("/main.rs")
                        .trim_end_matches(".rs");
                    (stem.to_string(), CrateKind::Bin)
                }
            };
            let items = &root.ast().ok()?.items;
            let mut tree = CrateTree {
                name,
                kind,
                modules: Vec::new(),
            };
            let total = root.lines.total;
            tree.modules
                .push(module(String::new(), root, items, None, false, (1, total)));
            walk(&mut tree, 0, sources, &dir_of(&root.in_package, true));
            Some(tree)
        })
        .collect()
}

fn lib_name(pkg: &Package) -> String {
    pkg.targets
        .iter()
        .find(|(k, _)| {
            matches!(
                k,
                sl_model::TargetKind::Lib | sl_model::TargetKind::ProcMacro
            )
        })
        .map_or_else(|| pkg.name.replace('-', "_"), |(_, n)| n.replace('-', "_"))
}

fn module<'a>(
    path: String,
    file: &'a SourceFile,
    items: &'a [Item],
    parent: Option<usize>,
    public: bool,
    lines: (u32, u32),
) -> Module<'a> {
    Module {
        code: own_code(file, items),
        path,
        file,
        items,
        parent,
        children: Vec::new(),
        public,
        lines,
    }
}

/// Code lines of `items`, skipping test items and inline modules
fn own_code(file: &SourceFile, items: &[Item]) -> u32 {
    let lines: Vec<&str> = file.text.lines().collect();
    items
        .iter()
        .filter(|i| !is_test(item_attrs(i)))
        .filter(|i| !matches!(i, Item::Mod(m) if m.content.is_some()))
        .map(|i| {
            let (a, b) = (line(i) as usize, end_line(i) as usize);
            let a = a.max(1).min(lines.len().max(1));
            let b = b.clamp(a, lines.len().max(a));
            count_lines(&lines.get(a - 1..b).unwrap_or(&[]).join("\n")).code
        })
        .sum()
}

/// The directory (inside the package) where `mod x;` of a file looks for `x.rs` and `x/mod.rs`
fn dir_of(in_package: &str, owns_dir: bool) -> String {
    let (dir, file) = in_package.rsplit_once('/').unwrap_or(("", in_package));
    let stem = file.trim_end_matches(".rs");
    if owns_dir || matches!(stem, "mod" | "lib" | "main") {
        dir.to_string()
    } else {
        format!("{dir}/{stem}")
    }
}

fn walk<'a>(tree: &mut CrateTree<'a>, idx: usize, sources: &'a [SourceFile], dir: &str) {
    let m = &tree.modules[idx];
    let (file, items, prefix) = (m.file, m.items, m.path.clone());
    for item in items {
        let Item::Mod(im) = item else { continue };
        if is_test(&im.attrs) {
            continue;
        }
        let name = im.ident.to_string();
        let path = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}::{name}")
        };
        let public = matches!(im.vis, Visibility::Public(_));
        let attr_path = path_attr(&im.attrs);
        let child = if let Some((_, inner)) = &im.content {
            let lines = (line(im), end_line(im));
            let sub = match &attr_path {
                Some(p) => join_dir(dir, p),
                None => join_dir(dir, &name),
            };
            let c = tree.modules.len();
            tree.modules
                .push(module(path, file, inner, Some(idx), public, lines));
            walk(tree, c, sources, &sub);
            c
        } else {
            let candidates = match &attr_path {
                Some(p) => vec![normalize(&join_dir(dir, p))],
                None => vec![
                    normalize(&join_dir(dir, &format!("{name}.rs"))),
                    normalize(&join_dir(dir, &format!("{name}/mod.rs"))),
                ],
            };
            let Some(f) = candidates
                .iter()
                .find_map(|c| sources.iter().find(|s| s.in_package == *c))
            else {
                continue;
            };
            let Ok(ast) = f.ast() else { continue };
            let c = tree.modules.len();
            tree.modules.push(module(
                path,
                f,
                &ast.items,
                Some(idx),
                public,
                (1, f.lines.total),
            ));
            // a `#[path]` file owns no directory of its own name unless it is a mod.rs
            walk(tree, c, sources, &dir_of(&f.in_package, false));
            c
        };
        tree.modules[idx].children.push((name, child));
    }
}

fn path_attr(attrs: &[syn::Attribute]) -> Option<String> {
    attrs.iter().find_map(|a| {
        if !a.path().is_ident("path") {
            return None;
        }
        let syn::Meta::NameValue(nv) = &a.meta else {
            return None;
        };
        match &nv.value {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) => Some(s.value()),
            _ => None,
        }
    })
}

fn join_dir(dir: &str, rest: &str) -> String {
    if dir.is_empty() {
        rest.to_string()
    } else {
        format!("{dir}/{rest}")
    }
}

/// Resolves `.` and `..` segments
fn normalize(p: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    for s in p.split('/') {
        match s {
            "" | "." => {}
            ".." => {
                out.pop();
            }
            s => out.push(s),
        }
    }
    out.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dirs() {
        assert_eq!(dir_of("src/lib.rs", true), "src");
        assert_eq!(dir_of("src/a.rs", false), "src/a");
        assert_eq!(dir_of("src/a/mod.rs", false), "src/a");
        assert_eq!(dir_of("src/bin/x.rs", true), "src/bin");
        assert_eq!(normalize("src/a/../b.rs"), "src/b.rs");
    }
}
