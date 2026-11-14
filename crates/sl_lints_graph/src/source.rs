//! The source-reading graph lints: `unsafe` against the map, I/O in `_core` crates.

use sl_analysis::Context;
use sl_analysis::syntax::{line, without_tests};
use sl_config::Map;
use sl_core::{Diagnostic, Span};
use sl_model::{FileKind, Project};
use syn::spanned::Spanned;
use syn::visit::Visit;

/// An attribute rustc's `unsafe_code` lint fires on: `no_mangle`, `export_name`, `link_section` (bare or in
/// `unsafe(...)`), and a linkme `distributed_slice(NAME)` element, which expands to a `link_section`
fn unsafe_attribute(a: &syn::Attribute) -> Option<&'static str> {
    let path = a.path();
    if path.is_ident("unsafe") {
        return Some("an unsafe attribute");
    }
    if ["no_mangle", "export_name", "link_section"]
        .iter()
        .any(|n| path.is_ident(n))
    {
        return Some("a no_mangle, export_name or link_section attribute");
    }
    let element = path
        .segments
        .last()
        .is_some_and(|s| s.ident == "distributed_slice")
        && matches!(a.meta, syn::Meta::List(_));
    element.then_some("a linkme distributed_slice element")
}

/// The first `unsafe` of a file: (line, what)
fn first_unsafe(file: &syn::File) -> Option<(u32, &'static str)> {
    struct V(Option<(u32, &'static str)>);
    impl<'ast> Visit<'ast> for V {
        fn visit_expr_unsafe(&mut self, e: &'ast syn::ExprUnsafe) {
            self.0.get_or_insert((line(e), "an unsafe block"));
        }
        fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
            if matches!(f.sig.safety, syn::Safety::Unsafe(_)) {
                self.0.get_or_insert((line(f), "an unsafe fn"));
            }
            syn::visit::visit_item_fn(self, f);
        }
        fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
            if matches!(f.sig.safety, syn::Safety::Unsafe(_)) {
                self.0.get_or_insert((line(f), "an unsafe fn"));
            }
            syn::visit::visit_impl_item_fn(self, f);
        }
        fn visit_item_impl(&mut self, i: &'ast syn::ItemImpl) {
            if i.unsafety.is_some() {
                self.0.get_or_insert((line(i), "an unsafe impl"));
            }
            syn::visit::visit_item_impl(self, i);
        }
        fn visit_item_trait(&mut self, t: &'ast syn::ItemTrait) {
            if t.unsafety.is_some() {
                self.0.get_or_insert((line(t), "an unsafe trait"));
            }
            syn::visit::visit_item_trait(self, t);
        }
        fn visit_attribute(&mut self, a: &'ast syn::Attribute) {
            if let Some(what) = unsafe_attribute(a) {
                self.0.get_or_insert((a.span().start().line as u32, what));
            }
            if (a.path().is_ident("allow") || a.path().is_ident("expect"))
                && a.meta
                    .require_list()
                    .is_ok_and(|l| l.tokens.to_string().contains("unsafe_code"))
            {
                self.0
                    .get_or_insert((a.span().start().line as u32, "an allow(unsafe_code)"));
            }
        }
    }
    let mut v = V(None);
    v.visit_file(file);
    v.0
}

pub(crate) fn unsafe_allow_list(cx: &Context<'_>, p: &Project, map: &Map) {
    let root = &p.root_manifest;
    let forbid_at = if p.is_workspace() {
        ["workspace", "lints", "rust", "unsafe_code"]
    } else {
        ["lints", "rust", "unsafe_code", ""]
    };
    let path: Vec<&str> = forbid_at
        .iter()
        .copied()
        .filter(|s| !s.is_empty())
        .collect();
    let item = root.get(&path);
    let level = item.and_then(|i| {
        i.as_str()
            .map(str::to_owned)
            .or_else(|| i.get("level").and_then(|l| l.as_str()).map(str::to_owned))
    });
    if level.as_deref() != Some("forbid") {
        cx.emit(
            Diagnostic::new("unsafe-allow-list", "unsafe_code", format!("{} does not set unsafe_code = \"forbid\"", path[..path.len() - 1].join(".")))
                .span(root.span(&path))
                .help(format!("set `unsafe_code = \"forbid\"` under [{}]; the crates the map marks `unsafe = true` override it in their own [lints]", path[..path.len() - 1].join("."))),
        );
    }
    for pkg in &p.packages {
        let allowed = map.crates.get(&pkg.name).is_some_and(|c| c.r#unsafe);
        let mut found = None;
        for f in p
            .sources(pkg)
            .iter()
            .filter(|f| f.kind == FileKind::Src && !p.is_test_file(pkg, f))
        {
            let Ok(ast) = f.ast() else { continue };
            if let Some((l, what)) = first_unsafe(ast) {
                found = Some((f.rel.clone(), l, what));
                break;
            }
        }
        match (found, allowed) {
            (Some((file, l, what)), false) => cx.emit(
                Diagnostic::new("unsafe-allow-list", format!("crate:{}", pkg.name), format!("{} has {what}, but the map does not mark it unsafe", pkg.name))
                    .span(Span::new(file, l))
                    .krate(&pkg.name)
                    .help("remove the unsafe code, or move it into a crate the map marks `unsafe = true` (a small one, with a safe API)"),
            ),
            (None, true) => cx.emit(
                Diagnostic::new("unsafe-allow-list", format!("map:{}", pkg.name), format!("the map marks {} unsafe, but it has no unsafe code", pkg.name))
                    .span(Span::new(map.file.clone(), map.crates.get(&pkg.name).map_or(0, |c| c.line)))
                    .krate(&pkg.name)
                    .help("remove `unsafe = true` from its map entry, so the list stays the list of crates that need it"),
            ),
            _ => {}
        }
    }
}

pub(crate) fn core_types(cx: &Context<'_>, p: &Project) {
    let params = cx.params("core-crate-types");
    let suffix = params.str("suffix");
    let io: Vec<Vec<String>> = params
        .list("io")
        .iter()
        .map(|s| s.split("::").map(str::to_owned).collect())
        .collect();
    struct V<'a> {
        io: &'a [Vec<String>],
        hit: Option<(u32, String)>,
    }
    impl V<'_> {
        fn check(&mut self, segs: &[String], at: u32) {
            if self.hit.is_some() {
                return;
            }
            if let Some(p) = self
                .io
                .iter()
                .find(|p| segs.len() >= p.len() && segs[..p.len()] == p[..])
            {
                self.hit = Some((at, p.join("::")));
            }
        }
    }
    impl<'ast> Visit<'ast> for V<'_> {
        fn visit_path(&mut self, path: &'ast syn::Path) {
            let segs: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();
            self.check(&segs, path.span().start().line as u32);
            syn::visit::visit_path(self, path);
        }
        fn visit_use_tree(&mut self, t: &'ast syn::UseTree) {
            fn flat(t: &syn::UseTree, pre: &mut Vec<String>, out: &mut Vec<Vec<String>>) {
                match t {
                    syn::UseTree::Path(p) => {
                        pre.push(p.ident.to_string());
                        flat(&p.tree, pre, out);
                        pre.pop();
                    }
                    syn::UseTree::Name(n) => {
                        let mut v = pre.clone();
                        v.push(n.ident.to_string());
                        out.push(v);
                    }
                    syn::UseTree::Rename(r) => {
                        let mut v = pre.clone();
                        v.push(r.ident.to_string());
                        out.push(v);
                    }
                    syn::UseTree::Glob(_) => out.push(pre.clone()),
                    syn::UseTree::Group(g) => g.items.iter().for_each(|i| flat(i, pre, out)),
                }
            }
            let mut out = Vec::new();
            flat(t, &mut Vec::new(), &mut out);
            for segs in out {
                self.check(&segs, t.span().start().line as u32);
            }
        }
    }
    for pkg in p.packages.iter().filter(|k| k.name.ends_with(suffix)) {
        for f in p
            .sources(pkg)
            .iter()
            .filter(|f| f.kind == FileKind::Src && !p.is_test_file(pkg, f))
        {
            let Ok(ast) = f.ast() else { continue };
            let ast = without_tests(ast);
            let mut v = V { io: &io, hit: None };
            v.visit_file(&ast);
            if let Some((l, what)) = v.hit {
                cx.emit(
                    Diagnostic::new("core-crate-types", format!("{}:{}", pkg.name, f.rel), format!("{} is a {suffix} crate but {} uses {what}", pkg.name, f.rel))
                        .span(Span::new(f.rel.clone(), l))
                        .krate(&pkg.name)
                        .help(format!("a {suffix} crate holds types only, so a CLI, a server, a test or a wasm build can use them without I/O; move this into the crate that does the work")),
                );
            }
        }
    }
}
