//! Helpers for analyses over syn trees: test code, lines of spans, the functions of a file with their paths.

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Attribute, Block, ImplItem, Item, TraitItem};

/// Whether an attribute list marks test-only code: `#[cfg(test)]`, `#[test]`, `#[tokio::test]` and friends,
/// `#[cfg(any(test, ...))]` counts too
pub fn is_test(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|a| {
        let p = a.path();
        if p.segments.last().is_some_and(|s| s.ident == "test") {
            return true;
        }
        if p.is_ident("cfg") {
            let mut found = false;
            let _ = a.parse_nested_meta(|m| {
                if m.path.is_ident("test") {
                    found = true;
                } else if m.path.is_ident("any") || m.path.is_ident("all") {
                    let _ = m.parse_nested_meta(|n| {
                        if n.path.is_ident("test") {
                            found = true;
                        }
                        Ok(())
                    });
                }
                Ok(())
            });
            return found;
        }
        false
    })
}

/// The first line of a spanned node (1-based)
pub fn line<T: Spanned>(node: &T) -> u32 {
    node.span().start().line as u32
}

/// The last line of a spanned node
pub fn end_line<T: Spanned>(node: &T) -> u32 {
    node.span().end().line as u32
}

/// A copy of the file without test-only items (modules, functions, impls marked as tests)
pub fn without_tests(file: &syn::File) -> syn::File {
    let mut f = file.clone();
    strip(&mut f.items);
    f
}

fn strip(items: &mut Vec<Item>) {
    items.retain(|i| !is_test(item_attrs(i)));
    for i in items {
        match i {
            Item::Mod(m) => {
                if let Some((_, inner)) = &mut m.content {
                    strip(inner);
                }
            }
            Item::Impl(im) => im.items.retain(|ii| match ii {
                ImplItem::Fn(f) => !is_test(&f.attrs),
                _ => true,
            }),
            _ => {}
        }
    }
}

/// The attributes of an item
pub fn item_attrs(i: &Item) -> &[Attribute] {
    match i {
        Item::Const(x) => &x.attrs,
        Item::Enum(x) => &x.attrs,
        Item::ExternCrate(x) => &x.attrs,
        Item::Fn(x) => &x.attrs,
        Item::ForeignMod(x) => &x.attrs,
        Item::Impl(x) => &x.attrs,
        Item::Macro(x) => &x.attrs,
        Item::Mod(x) => &x.attrs,
        Item::Static(x) => &x.attrs,
        Item::Struct(x) => &x.attrs,
        Item::Trait(x) => &x.attrs,
        Item::TraitAlias(x) => &x.attrs,
        Item::Type(x) => &x.attrs,
        Item::Union(x) => &x.attrs,
        Item::Use(x) => &x.attrs,
        _ => &[],
    }
}

/// A function with a body
pub struct FnInfo<'a> {
    /// Its path inside the crate (`a::b::Type::method`)
    pub path: String,
    /// Its name
    pub name: String,
    /// Whether it is `async`
    pub is_async: bool,
    /// Its body
    pub block: &'a Block,
    /// First and last line of the whole function
    pub lines: (u32, u32),
}

/// Every function with a body in the file (free functions, inherent and trait impl methods, trait default
/// methods, nested modules), with paths starting at `module`
pub fn functions<'a>(file: &'a syn::File, module: &str) -> Vec<FnInfo<'a>> {
    let mut out = Vec::new();
    collect(&file.items, module, &mut out);
    out
}

fn join(a: &str, b: &str) -> String {
    if a.is_empty() {
        b.to_string()
    } else {
        format!("{a}::{b}")
    }
}

fn collect<'a>(items: &'a [Item], prefix: &str, out: &mut Vec<FnInfo<'a>>) {
    for item in items {
        match item {
            Item::Fn(f) => out.push(FnInfo {
                path: join(prefix, &f.sig.ident.to_string()),
                name: f.sig.ident.to_string(),
                is_async: f.sig.asyncness.is_some(),
                block: &f.block,
                lines: (line(f), end_line(f)),
            }),
            Item::Mod(m) => {
                if let Some((_, inner)) = &m.content {
                    collect(inner, &join(prefix, &m.ident.to_string()), out);
                }
            }
            Item::Impl(im) => {
                let ty = type_name(&im.self_ty);
                for ii in &im.items {
                    if let ImplItem::Fn(f) = ii {
                        out.push(FnInfo {
                            path: join(prefix, &format!("{ty}::{}", f.sig.ident)),
                            name: f.sig.ident.to_string(),
                            is_async: f.sig.asyncness.is_some(),
                            block: &f.block,
                            lines: (line(f), end_line(f)),
                        });
                    }
                }
            }
            Item::Trait(t) => {
                for ti in &t.items {
                    if let TraitItem::Fn(f) = ti
                        && let Some(b) = &f.default
                    {
                        out.push(FnInfo {
                            path: join(prefix, &format!("{}::{}", t.ident, f.sig.ident)),
                            name: f.sig.ident.to_string(),
                            is_async: f.sig.asyncness.is_some(),
                            block: b,
                            lines: (line(f), end_line(f)),
                        });
                    }
                }
            }
            _ => {}
        }
    }
}

/// The last path segment of a type (`Foo` for `crate::a::Foo<T>`), `_` when it has none
pub fn type_name(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(p) => p
            .path
            .segments
            .last()
            .map_or_else(|| "_".into(), |s| s.ident.to_string()),
        syn::Type::Reference(r) => type_name(&r.elem),
        _ => "_".into(),
    }
}

/// The names of every path segment in a type, for matching `Mutex`, `RefCell` anywhere inside
pub fn type_idents(ty: &syn::Type) -> Vec<String> {
    struct V(Vec<String>);
    impl<'ast> Visit<'ast> for V {
        fn visit_path_segment(&mut self, s: &'ast syn::PathSegment) {
            self.0.push(s.ident.to_string());
            syn::visit::visit_path_segment(self, s);
        }
    }
    let mut v = V(Vec::new());
    v.visit_type(ty);
    v.0
}

/// McCabe cyclomatic complexity of a block: 1 plus each branch point (`if`, each `match` arm after the first,
/// loops, `&&`, `||`, `?`)
pub fn complexity(block: &Block) -> u32 {
    struct C(u32);
    impl<'ast> Visit<'ast> for C {
        fn visit_expr_if(&mut self, e: &'ast syn::ExprIf) {
            self.0 += 1;
            syn::visit::visit_expr_if(self, e);
        }
        fn visit_expr_match(&mut self, e: &'ast syn::ExprMatch) {
            self.0 += e.arms.len().saturating_sub(1) as u32;
            syn::visit::visit_expr_match(self, e);
        }
        fn visit_expr_while(&mut self, e: &'ast syn::ExprWhile) {
            self.0 += 1;
            syn::visit::visit_expr_while(self, e);
        }
        fn visit_expr_for_loop(&mut self, e: &'ast syn::ExprForLoop) {
            self.0 += 1;
            syn::visit::visit_expr_for_loop(self, e);
        }
        fn visit_expr_loop(&mut self, e: &'ast syn::ExprLoop) {
            self.0 += 1;
            syn::visit::visit_expr_loop(self, e);
        }
        fn visit_expr_try(&mut self, e: &'ast syn::ExprTry) {
            self.0 += 1;
            syn::visit::visit_expr_try(self, e);
        }
        fn visit_expr_binary(&mut self, e: &'ast syn::ExprBinary) {
            if matches!(e.op, syn::BinOp::And(_) | syn::BinOp::Or(_)) {
                self.0 += 1;
            }
            syn::visit::visit_expr_binary(self, e);
        }
        fn visit_item_fn(&mut self, _: &'ast syn::ItemFn) {}
        fn visit_expr_closure(&mut self, e: &'ast syn::ExprClosure) {
            syn::visit::visit_expr_closure(self, e);
        }
    }
    let mut c = C(1);
    c.visit_block(block);
    c.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tests_are_stripped() {
        let f: syn::File = syn::parse_str(
            "fn a() {}\n#[cfg(test)] mod tests { fn b() {} }\n#[test] fn c() {}\nmod m { fn d() {} }\nimpl X { #[tokio::test] async fn e() {} fn g() {} }",
        )
        .unwrap();
        let s = without_tests(&f);
        let names: Vec<String> = functions(&s, "k").into_iter().map(|f| f.path).collect();
        assert_eq!(names, vec!["k::a", "k::m::d", "k::X::g"]);
    }

    #[test]
    fn complexity_counts_branches() {
        let f: syn::ItemFn = syn::parse_str(
            "fn a(x: u8) -> Option<u8> { if x > 1 && x < 5 { return None } for _ in 0..3 {} match x { 1 => {}, 2 => {}, _ => {} } Some(f()?) }",
        )
        .unwrap();
        assert_eq!(complexity(&f.block), 1 + 1 + 1 + 1 + 2 + 1);
    }
}
