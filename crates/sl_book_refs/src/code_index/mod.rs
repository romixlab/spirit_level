//! The index of public items a book's code links name.
//!
//! Every public item of the book's library crates, found by walking each library's module tree from its root as
//! the compiler does (`mod x;` to `x.rs` or `x/mod.rs`, `#[path]` honoured, `#[cfg(test)]` modules left out). Each
//! entry has its path through the modules it is defined in (`ww_tape::writer::resume::TapeWriter::resume_on`), its
//! kind, file and lines, its signature (its whole source when short), its first doc paragraph, and the address of
//! its lines on the forge at one commit.
//!
//! The index is written at every book build and never kept in git, so it always matches the code: a link to a
//! renamed or removed item fails the build in the change that renames it.

use serde::{Deserialize, Serialize};
use sl_model::{Package, Project, SourceFile, TargetKind};
use syn::spanned::Spanned;
use syn::{Attribute, Fields, ImplItem, Item, Visibility};

/// The version of the index's layout; the Markdown extension refuses an index of another
pub(crate) const FORMAT: u32 = 1;

/// Lines of an item's source shown whole at most; a longer item shows its signature alone
const WHOLE_LINES_MAX: usize = 14;

/// What an item is
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// A function or method
    Fn,
    /// A struct
    Struct,
    /// A named field of a struct
    Field,
    /// An enum
    Enum,
    /// A variant of an enum
    Variant,
    /// A trait
    Trait,
    /// A type alias
    Type,
    /// A constant, associated ones included
    Const,
    /// A static
    Static,
    /// An exported `macro_rules!` macro
    Macro,
}

/// One public item
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    /// Its path from its crate's name through the modules it is defined in
    pub path: String,
    /// What it is
    pub kind: Kind,
    /// The file it is defined in, relative to the project root
    pub file: String,
    /// Its first and last line in the file, from 1: from its visibility to its end, docs and attributes left out
    pub lines: [usize; 2],
    /// Its signature, or its whole source when short, dedented
    pub source: String,
    /// The first paragraph of its docs, as written
    pub doc: String,
    /// Its lines on the forge at the index's commit
    pub url: String,
}

/// The index file
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Index {
    /// The version of its layout, 1; the Markdown extension refuses an index of another
    pub format: u32,
    /// The commit the addresses point at
    pub commit: String,
    /// The entries, sorted by path
    pub items: Vec<Entry>,
}

/// Where the entries' addresses point
#[derive(Clone, Copy, Debug)]
pub struct Forge<'a> {
    /// The repository's address (`https://github.com/vhrdtech/ww_tape`)
    pub repository: &'a str,
    /// The full commit hash the book is built from
    pub commit: &'a str,
}

impl Forge<'_> {
    /// A file's lines, as GitHub, Gitea and Forgejo address them
    fn url(&self, file: &str, [from, to]: [usize; 2]) -> String {
        format!(
            "{}/blob/{}/{file}#L{from}-L{to}",
            self.repository.trim_end_matches('/'),
            self.commit
        )
    }
}

/// Why an index cannot be built
#[derive(Debug, PartialEq, Eq)]
pub enum IndexError {
    /// A crate the settings name is no library of the workspace
    UnknownCrate {
        /// The name as written
        name: String,
        /// The libraries there are
        known: Vec<String>,
    },
    /// A source file does not parse (`file:line:column: message`)
    Parse(String),
    /// A `mod x;` whose file is not there
    MissingModule {
        /// The declaring file
        file: String,
        /// The module's name
        name: String,
    },
}

impl std::fmt::Display for IndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownCrate { name, known } => write!(
                f,
                "`{name}` is no library of the workspace; the libraries are {}",
                known.join(", ")
            ),
            Self::Parse(e) => write!(f, "{e}"),
            Self::MissingModule { file, name } => {
                write!(f, "{file}: `mod {name};` has no file")
            }
        }
    }
}

impl std::error::Error for IndexError {}

/// The index of `crates` (library target names; empty: every library of the workspace)
///
/// # Errors
///
/// A crate is no library of the workspace, a file does not parse, or a declared module has no file.
pub fn index(project: &Project, crates: &[String], forge: Forge<'_>) -> Result<Index, IndexError> {
    let libs: Vec<(&Package, &str, &str)> = project
        .packages
        .iter()
        .flat_map(|p| {
            p.targets
                .iter()
                .zip(&p.target_roots)
                .filter(|((k, _), _)| matches!(k, TargetKind::Lib | TargetKind::ProcMacro))
                .map(move |((_, name), root)| (p, name.as_str(), root.as_str()))
        })
        .collect();
    for c in crates {
        if !libs.iter().any(|(_, name, _)| name == c) {
            let mut known: Vec<String> = libs.iter().map(|(_, n, _)| (*n).to_owned()).collect();
            known.sort();
            return Err(IndexError::UnknownCrate {
                name: c.clone(),
                known,
            });
        }
    }
    let mut items = Vec::new();
    for (pkg, name, root) in libs {
        if !crates.is_empty() && !crates.iter().any(|c| c == name) {
            continue;
        }
        let files = project.sources(pkg);
        let mut walk = Walk {
            files,
            forge,
            out: &mut items,
        };
        walk.file(root, &[name.to_owned()])?;
    }
    items.sort_by(|a, b| a.path.cmp(&b.path).then(a.lines.cmp(&b.lines)));
    Ok(Index {
        format: FORMAT,
        commit: forge.commit.to_owned(),
        items,
    })
}

/// The first paragraph of the docs in `attrs`
fn doc(attrs: &[Attribute]) -> String {
    let mut lines = Vec::new();
    for a in attrs.iter().filter(|a| a.path().is_ident("doc")) {
        let syn::Meta::NameValue(nv) = &a.meta else {
            continue;
        };
        let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) = &nv.value
        else {
            continue;
        };
        let value = s.value();
        // an empty `///` line is an empty value, which has no lines: a paragraph break all the same
        let parts: Vec<&str> = if value.trim().is_empty() {
            vec![""]
        } else {
            value.lines().collect()
        };
        for l in parts {
            let line = l.strip_prefix(' ').unwrap_or(l);
            if line.trim().is_empty() {
                if lines.is_empty() {
                    continue;
                }
                return lines.join(" ");
            }
            lines.push(line.trim_end().to_owned());
        }
    }
    lines.join(" ")
}

fn is_pub(v: &Visibility) -> bool {
    matches!(v, Visibility::Public(_))
}

/// Whether attributes hold `#[cfg(test)]`
fn is_cfg_test(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("cfg") && a.parse_args::<syn::Ident>().is_ok_and(|i| i == "test")
    })
}

/// `#[path = "..."]` on a module
fn path_attr(attrs: &[Attribute]) -> Option<String> {
    attrs.iter().find_map(|a| {
        if !a.path().is_ident("path") {
            return None;
        }
        match &a.meta {
            syn::Meta::NameValue(syn::MetaNameValue {
                value:
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }),
                ..
            }) => Some(s.value()),
            _ => None,
        }
    })
}

/// The directory of a path inside a package, with a trailing slash unless empty
fn dir_of(path: &str) -> &str {
    path.rfind('/').map_or("", |i| &path[..=i])
}

struct Walk<'a> {
    files: &'a [SourceFile],
    forge: Forge<'a>,
    out: &'a mut Vec<Entry>,
}

/// One file being walked: its lines and where its child modules' files are
struct At<'a> {
    file: &'a SourceFile,
    lines: Vec<&'a str>,
    /// The directory a `mod x;` in this file looks in, inside the package, with a trailing slash unless empty
    children: String,
}

impl Walk<'_> {
    /// Walks the module in `in_package` (a path inside the package) whose path is `module`
    fn file(&mut self, in_package: &str, module: &[String]) -> Result<(), IndexError> {
        let Some(file) = self.files.iter().find(|f| f.in_package == in_package) else {
            return Ok(());
        };
        let ast = file.ast().map_err(|e| IndexError::Parse(e.to_owned()))?;
        let stem = in_package.rsplit('/').next().unwrap_or(in_package);
        let children = if matches!(stem, "lib.rs" | "main.rs" | "mod.rs") {
            dir_of(in_package).to_owned()
        } else {
            format!("{}/", in_package.trim_end_matches(".rs"))
        };
        let at = At {
            file,
            lines: file.text.lines().collect(),
            children,
        };
        self.items(&at, module, &at.children.clone(), &ast.items)
    }

    /// Walks `items` of `module`, whose out-of-line children are in `children`
    fn items(
        &mut self,
        at: &At<'_>,
        module: &[String],
        children: &str,
        items: &[Item],
    ) -> Result<(), IndexError> {
        let join = |name: &syn::Ident| format!("{}::{name}", module.join("::"));
        for item in items {
            match item {
                Item::Fn(f) if is_pub(&f.vis) => {
                    let open = f.block.brace_token.span.open().start().line;
                    self.push(at, join(&f.sig.ident), Kind::Fn, &f.attrs, item, Some(open));
                }
                Item::Struct(s) if is_pub(&s.vis) => self.structure(at, join(&s.ident), s),
                Item::Enum(e) if is_pub(&e.vis) => self.enumeration(at, join(&e.ident), e),
                Item::Trait(t) if is_pub(&t.vis) => {
                    let open = t.brace_token.span.open().start().line;
                    self.push(at, join(&t.ident), Kind::Trait, &t.attrs, item, Some(open));
                }
                Item::Type(t) if is_pub(&t.vis) => {
                    self.push(at, join(&t.ident), Kind::Type, &t.attrs, item, None);
                }
                Item::Const(c) if is_pub(&c.vis) => {
                    self.push(at, join(&c.ident), Kind::Const, &c.attrs, item, None);
                }
                Item::Static(s) if is_pub(&s.vis) => {
                    self.push(at, join(&s.ident), Kind::Static, &s.attrs, item, None);
                }
                Item::Macro(m) => self.exported_macro(at, module, m),
                Item::Mod(m) if !is_cfg_test(&m.attrs) => self.module(at, module, children, m)?,
                Item::Impl(i) if i.trait_.is_none() => {
                    if let syn::Type::Path(p) = &*i.self_ty
                        && let Some(ty) = p.path.segments.last()
                    {
                        self.inherent(at, &join(&ty.ident), &i.items);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// A `macro_rules!` macro, when exported: it lives at the crate's root, wherever it is written
    fn exported_macro(&mut self, at: &At<'_>, module: &[String], m: &syn::ItemMacro) {
        if !m.attrs.iter().any(|a| a.path().is_ident("macro_export")) {
            return;
        }
        if let Some(ident) = &m.ident {
            let krate = module.first().map_or("", String::as_str);
            let path = format!("{krate}::{ident}");
            self.push(at, path, Kind::Macro, &m.attrs, m, None);
        }
    }

    /// A public struct and its public named fields
    fn structure(&mut self, at: &At<'_>, name: String, s: &syn::ItemStruct) {
        let open = match &s.fields {
            Fields::Named(n) => Some(n.brace_token.span.open().start().line),
            _ => None,
        };
        if let Fields::Named(n) = &s.fields {
            for f in n.named.iter().filter(|f| is_pub(&f.vis)) {
                if let Some(ident) = &f.ident {
                    self.push(
                        at,
                        format!("{name}::{ident}"),
                        Kind::Field,
                        &f.attrs,
                        f,
                        None,
                    );
                }
            }
        }
        self.push(at, name, Kind::Struct, &s.attrs, s, open);
    }

    /// A public enum and its variants
    fn enumeration(&mut self, at: &At<'_>, name: String, e: &syn::ItemEnum) {
        for v in &e.variants {
            let path = format!("{name}::{}", v.ident);
            self.push(at, path, Kind::Variant, &v.attrs, v, None);
        }
        let open = e.brace_token.span.open().start().line;
        self.push(at, name, Kind::Enum, &e.attrs, e, Some(open));
    }

    /// The public functions and constants of an inherent impl of `ty`
    fn inherent(&mut self, at: &At<'_>, ty: &str, items: &[ImplItem]) {
        for it in items {
            match it {
                ImplItem::Fn(f) if is_pub(&f.vis) => {
                    let open = f.block.brace_token.span.open().start().line;
                    let path = format!("{ty}::{}", f.sig.ident);
                    self.push(at, path, Kind::Fn, &f.attrs, it, Some(open));
                }
                ImplItem::Const(c) if is_pub(&c.vis) => {
                    let path = format!("{ty}::{}", c.ident);
                    self.push(at, path, Kind::Const, &c.attrs, it, None);
                }
                _ => {}
            }
        }
    }

    /// A module, inline or in its own file
    fn module(
        &mut self,
        at: &At<'_>,
        module: &[String],
        children: &str,
        m: &syn::ItemMod,
    ) -> Result<(), IndexError> {
        let mut inner = module.to_vec();
        inner.push(m.ident.to_string());
        if let Some((_, nested)) = &m.content {
            let dir = format!("{children}{}/", m.ident);
            self.items(at, &inner, &dir, nested)
        } else {
            let file = self.module_file(at, children, &m.ident.to_string(), &m.attrs)?;
            self.file(&file, &inner)
        }
    }

    /// The file of `mod name;`: its `#[path]` beside the declaring file, else `name.rs` or `name/mod.rs`
    fn module_file(
        &self,
        at: &At<'_>,
        children: &str,
        name: &str,
        attrs: &[Attribute],
    ) -> Result<String, IndexError> {
        let candidates = match path_attr(attrs) {
            Some(p) => vec![format!("{}{p}", dir_of(&at.file.in_package))],
            None => vec![
                format!("{children}{name}.rs"),
                format!("{children}{name}/mod.rs"),
            ],
        };
        candidates
            .into_iter()
            .find(|c| self.files.iter().any(|f| &f.in_package == c))
            .ok_or_else(|| IndexError::MissingModule {
                file: at.file.rel.clone(),
                name: name.to_owned(),
            })
    }

    /// Adds an item spanning `node`, whose body opens on line `open` (none: no body to cut off)
    fn push(
        &mut self,
        at: &At<'_>,
        path: String,
        kind: Kind,
        attrs: &[Attribute],
        node: &dyn Spanned,
        open: Option<usize>,
    ) {
        let span = node.span();
        // the item's span starts at its first attribute; the reader wants it from the line after them
        let start = attrs
            .iter()
            .map(|a| a.span().end().line + 1)
            .max()
            .unwrap_or(span.start().line)
            .max(span.start().line);
        let end = span.end().line.max(start);
        let lines = [start, end];
        self.out.push(Entry {
            path,
            kind,
            file: at.file.rel.clone(),
            lines,
            source: source(&at.lines, start, end, open),
            doc: doc(attrs),
            url: self.forge.url(&at.file.rel, lines),
        });
    }
}

/// The lines `from..=to` (from 1), dedented
fn text(lines: &[&str], from: usize, to: usize) -> String {
    let picked = lines
        .get(from.saturating_sub(1)..to.min(lines.len()))
        .unwrap_or_default();
    let indent = picked
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    picked
        .iter()
        .map(|l| l.get(indent..).unwrap_or("").trim_end())
        .collect::<Vec<_>>()
        .join("\n")
}

/// An item's whole source when short or without a body, else the lines before its body, the brace cut off
fn source(lines: &[&str], start: usize, end: usize, open: Option<usize>) -> String {
    match open {
        Some(open) if end + 1 - start > WHOLE_LINES_MAX => text(lines, start, open)
            .trim_end()
            .trim_end_matches('{')
            .trim_end()
            .to_owned(),
        _ => text(lines, start, end),
    }
}

#[cfg(test)]
mod tests;
