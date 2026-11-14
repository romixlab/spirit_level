//! Lock guards in blocks, read without types: a `let` whose value is a zero-argument `lock()`, `read()`,
//! `write()`, `try_lock()`, `borrow()` or `borrow_mut()` call (after peeling `.unwrap()`, `.expect(..)`, `?` and
//! `.await`) binds a guard that lives to the end of its block or a `drop(name)`. Later statements of the block
//! that `.await`, do I/O or take another lock while it lives are reported.

use sl_core::Span;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Expr, Pat, Stmt};

/// Methods that take a lock (not `borrow`: a second `RefCell` borrow is common and safe)
const LOCK_METHODS: &[&str] = &[
    "lock",
    "read",
    "write",
    "try_lock",
    "try_read",
    "try_write",
    "lock_arc",
];

const GUARD_METHODS: &[&str] = &[
    "lock",
    "read",
    "write",
    "try_lock",
    "try_read",
    "try_write",
    "borrow",
    "borrow_mut",
    "lock_arc",
];

pub(crate) struct Guards<'a> {
    file: &'a str,
    io: &'a [String],
    /// (lint, place, message)
    pub(crate) found: Vec<(&'static str, Span, String)>,
    /// `.lock().unwrap()`-style calls
    pub(crate) unwraps: Vec<Span>,
}

impl<'a> Guards<'a> {
    pub(crate) fn new(file: &'a str, io: &'a [String]) -> Self {
        Guards {
            file,
            io,
            found: Vec::new(),
            unwraps: Vec::new(),
        }
    }
}

/// The guard method a value expression ends in, after peeling unwrap/expect/?/await
fn guard_call(mut e: &Expr) -> Option<&syn::ExprMethodCall> {
    loop {
        match e {
            Expr::MethodCall(m)
                if ["unwrap", "expect", "unwrap_or_else"]
                    .contains(&m.method.to_string().as_str()) =>
            {
                e = &m.receiver;
            }
            Expr::Try(t) => e = &t.expr,
            Expr::Await(a) => e = &a.base,
            Expr::MethodCall(m)
                if m.args.is_empty() && GUARD_METHODS.contains(&m.method.to_string().as_str()) =>
            {
                return Some(m);
            }
            _ => return None,
        }
    }
}

fn pat_name(p: &Pat) -> Option<String> {
    match p {
        Pat::Ident(i) => Some(i.ident.to_string()),
        Pat::Type(t) => pat_name(&t.pat),
        _ => None,
    }
}

/// What a statement does that matters while a guard lives
#[derive(Default)]
struct Effects {
    awaits: Option<u32>,
    io: Option<(u32, String)>,
    lock: Option<u32>,
    drops: Vec<String>,
}

struct Scan<'a> {
    io: &'a [String],
    fx: Effects,
}

impl<'ast> Visit<'ast> for Scan<'_> {
    fn visit_expr_await(&mut self, e: &'ast syn::ExprAwait) {
        self.fx
            .awaits
            .get_or_insert(e.await_token.span.start().line as u32);
        syn::visit::visit_expr_await(self, e);
    }

    fn visit_expr_method_call(&mut self, m: &'ast syn::ExprMethodCall) {
        let name = m.method.to_string();
        if m.args.is_empty() && LOCK_METHODS.contains(&name.as_str()) {
            self.fx
                .lock
                .get_or_insert(m.method.span().start().line as u32);
        }
        if self.io.contains(&name) {
            self.fx
                .io
                .get_or_insert((m.method.span().start().line as u32, format!(".{name}()")));
        }
        syn::visit::visit_expr_method_call(self, m);
    }

    fn visit_expr_call(&mut self, c: &'ast syn::ExprCall) {
        if let Expr::Path(p) = &*c.func {
            let segs: Vec<String> = p
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect();
            if segs.len() == 1 && segs[0] == "drop" {
                if let Some(Expr::Path(a)) = c.args.first()
                    && let Some(i) = a.path.get_ident()
                {
                    self.fx.drops.push(i.to_string());
                }
            } else if segs.len() > 1 && segs[..segs.len() - 1].iter().any(|s| self.io.contains(s)) {
                self.fx
                    .io
                    .get_or_insert((c.span().start().line as u32, segs.join("::")));
            }
        }
        syn::visit::visit_expr_call(self, c);
    }

    // a closure or async block defined here runs later, not while the guard lives
    fn visit_expr_closure(&mut self, _: &'ast syn::ExprClosure) {}
    fn visit_expr_async(&mut self, _: &'ast syn::ExprAsync) {}
    fn visit_item(&mut self, _: &'ast syn::Item) {}
}

impl<'ast> Visit<'ast> for Guards<'_> {
    fn visit_expr_method_call(&mut self, m: &'ast syn::ExprMethodCall) {
        let name = m.method.to_string();
        if (name == "unwrap" || name == "expect")
            && let Expr::MethodCall(inner) = &*m.receiver
            && inner.args.is_empty()
            && ["lock", "read", "write"].contains(&inner.method.to_string().as_str())
        {
            self.unwraps.push(
                Span::new(self.file, inner.method.span().start().line as u32)
                    .label(format!(".{}().{name}()", inner.method)),
            );
        }
        syn::visit::visit_expr_method_call(self, m);
    }

    fn visit_block(&mut self, b: &'ast syn::Block) {
        let stmts = &b.stmts;
        for (i, s) in stmts.iter().enumerate() {
            let Stmt::Local(l) = s else { continue };
            let Some(init) = &l.init else { continue };
            let Some(call) = guard_call(&init.expr) else {
                continue;
            };
            let Some(name) = pat_name(&l.pat) else {
                continue;
            };
            if name == "_" {
                continue; // `let _ = m.lock()` drops the guard at once
            }
            let at = call.method.span().start().line as u32;
            let what = format!(".{}()", call.method);
            for later in &stmts[i + 1..] {
                let mut sc = Scan {
                    io: self.io,
                    fx: Effects::default(),
                };
                sc.visit_stmt(later);
                if let Some(l2) = sc.fx.awaits {
                    self.found.push((
                        "lock-held-across",
                        Span::new(self.file, l2).label(".await"),
                        format!(
                            "guard `{name}` (from {what} on line {at}) is held across an .await"
                        ),
                    ));
                    break;
                }
                if let Some((l2, call)) = sc.fx.io {
                    self.found.push((
                        "lock-held-across",
                        Span::new(self.file, l2).label(call.clone()),
                        format!(
                            "guard `{name}` (from {what} on line {at}) is held across I/O ({call})"
                        ),
                    ));
                    break;
                }
                if let Some(l2) = sc.fx.lock {
                    self.found.push((
                        "nested-locks",
                        Span::new(self.file, l2),
                        format!("a second lock is taken while guard `{name}` (line {at}) is alive"),
                    ));
                    break;
                }
                if sc.fx.drops.contains(&name) {
                    break;
                }
            }
        }
        syn::visit::visit_block(self, b);
    }
}
