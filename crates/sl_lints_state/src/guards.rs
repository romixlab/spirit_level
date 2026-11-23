//! Lock guards in blocks, read without types: a `let` whose value is a zero-argument `lock()`, `read()`,
//! `write()`, `try_lock()`, `borrow()` or `borrow_mut()` call (after peeling `.unwrap()`, `.expect(..)`, `?` and
//! `.await`) binds a guard that lives to the end of its block or a `drop(name)`. Later statements of the block
//! that `.await`, do I/O or take another lock while it lives are reported. A `read()` or `write()` under a `?`
//! or with a turbofish (`rd.read()?`, `rd.read::<T>()`) is a decoder's, not a lock's.
//!
//! A lock helper of the workspace ([`Helper`]) takes a guard too: a call of one that returns the guard binds it
//! like `lock()`, and the closure handed to one that passes the guard on runs while it lives.

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
    helpers: &'a [Helper],
    /// (lint, place, message)
    pub(crate) found: Vec<(&'static str, Span, String)>,
    /// `.lock().unwrap()`-style calls
    pub(crate) unwraps: Vec<Unwrap>,
    /// The functions being visited, innermost last
    frames: Vec<Frame>,
}

/// Line and column of a call's method name: tells two calls apart
type Pos = (usize, usize);

fn pos(m: &syn::ExprMethodCall) -> Pos {
    let at = m.method.span().start();
    (at.line, at.column)
}

/// One `.lock().unwrap()`-style call
pub(crate) struct Unwrap {
    pub(crate) span: Span,
    /// The function it is in, when that function is a lock helper: the call is its only one, and the guard is
    /// returned or handed to a closure parameter, so the function decides what poison means for its callers
    pub(crate) helper: Option<String>,
    at: Pos,
}

/// A function being visited
struct Frame {
    name: String,
    /// The call that makes it a helper, if any
    helper_at: Option<Pos>,
    /// Its own calls, as indices into `unwraps`
    calls: Vec<usize>,
}

impl<'a> Guards<'a> {
    pub(crate) fn new(file: &'a str, io: &'a [String], helpers: &'a [Helper]) -> Self {
        Guards {
            file,
            io,
            helpers,
            found: Vec::new(),
            unwraps: Vec::new(),
            frames: Vec::new(),
        }
    }

    fn enter(&mut self, name: String, sig: &syn::Signature, body: &syn::Block) {
        self.frames.push(Frame {
            name,
            helper_at: helper_call(sig, body, lock_unwrap).map(|(at, _)| at),
            calls: Vec::new(),
        });
    }

    fn exit(&mut self) {
        let Some(f) = self.frames.pop() else { return };
        if let ([i], Some(at)) = (f.calls.as_slice(), f.helper_at)
            && self.unwraps[*i].at == at
        {
            self.unwraps[*i].helper = Some(f.name);
        }
    }
}

/// The `.lock()`, `.read()` or `.write()` call under an `.unwrap()` or `.expect(..)`
fn lock_unwrap(e: &Expr) -> Option<&syn::ExprMethodCall> {
    let Expr::MethodCall(m) = e else { return None };
    lock_unwrap_call(m)
}

/// [`lock_unwrap`] of a method call
fn lock_unwrap_call(m: &syn::ExprMethodCall) -> Option<&syn::ExprMethodCall> {
    let Expr::MethodCall(inner) = &*m.receiver else {
        return None;
    };
    (["unwrap", "expect"].contains(&m.method.to_string().as_str())
        && inner.args.is_empty()
        && ["lock", "read", "write"].contains(&inner.method.to_string().as_str()))
    .then_some(inner)
}

/// The call that makes a function a lock helper, by `take` (which finds a lock call in an expression): its body
/// ends in the call (the guard is returned), or binds the guard with `let` and ends in a call of one of its
/// parameters that takes the guard (`f(&mut held)`); with that parameter's index among the non-receiver inputs
fn helper_call(
    sig: &syn::Signature,
    body: &syn::Block,
    take: fn(&Expr) -> Option<&syn::ExprMethodCall>,
) -> Option<(Pos, Option<usize>)> {
    let Some(Stmt::Expr(tail, None)) = body.stmts.last() else {
        return None;
    };
    if let Some(c) = take(tail) {
        return Some((pos(c), None));
    }
    let Expr::Call(call) = tail else { return None };
    let Expr::Path(f) = &*call.func else {
        return None;
    };
    let f = f.path.get_ident()?;
    let param = sig
        .inputs
        .iter()
        .filter_map(|a| match a {
            syn::FnArg::Typed(t) => Some(pat_name(&t.pat)),
            syn::FnArg::Receiver(_) => None,
        })
        .position(|p| p.is_some_and(|p| *f == p))?;
    let at = call.args.iter().find_map(|a| {
        let mut a = a;
        while let Expr::Reference(syn::ExprReference { expr, .. })
        | Expr::Paren(syn::ExprParen { expr, .. })
        | Expr::Unary(syn::ExprUnary {
            op: syn::UnOp::Deref(_),
            expr,
            ..
        }) = a
        {
            a = expr;
        }
        let Expr::Path(arg) = a else { return None };
        let arg = arg.path.get_ident()?;
        body.stmts.iter().find_map(|s| match s {
            Stmt::Local(l) if pat_name(&l.pat).is_some_and(|n| *arg == n) => {
                l.init.as_ref().and_then(|i| take(&i.expr)).map(pos)
            }
            _ => None,
        })
    })?;
    Some((at, Some(param)))
}

/// A function of the workspace that takes a lock for its callers: it returns the guard, or hands it to one of
/// its closure parameters (found by [`helper_call`] with any lock call, `unwrap_or_else` and `.await` included)
pub(crate) struct Helper {
    name: String,
    /// Whether it takes `self`
    method: bool,
    /// Its inputs other than `self`
    inputs: usize,
    /// The closure parameter it hands the guard to, among those inputs
    closure: Option<usize>,
}

impl Helper {
    /// The call, when its callee and argument count are this helper's (`self` is the first argument of a path
    /// call to a method)
    fn call<'h, 'e>(
        &'h self,
        callee: &syn::Ident,
        args: &'e Args,
        method_call: bool,
    ) -> Option<HelperCall<'h, 'e>> {
        let shift = usize::from(self.method && !method_call);
        (*callee == self.name && (self.method || !method_call) && args.len() == self.inputs + shift)
            .then(|| (self, self.closure.and_then(|c| args.iter().nth(c + shift))))
    }
}

/// The lock helpers of a file
pub(crate) fn helpers(ast: &syn::File) -> Vec<Helper> {
    struct Find(Vec<Helper>);
    impl Find {
        fn add(&mut self, sig: &syn::Signature, body: &syn::Block) {
            if let Some((_, closure)) = helper_call(sig, body, |e| guard_call(e, LOCK_METHODS)) {
                let method = sig.receiver().is_some();
                self.0.push(Helper {
                    name: sig.ident.to_string(),
                    method,
                    inputs: sig.inputs.len() - usize::from(method),
                    closure,
                });
            }
        }
    }
    impl<'ast> Visit<'ast> for Find {
        fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
            self.add(&f.sig, &f.block);
            syn::visit::visit_item_fn(self, f);
        }
        fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
            self.add(&f.sig, &f.block);
            syn::visit::visit_impl_item_fn(self, f);
        }
        fn visit_trait_item_fn(&mut self, f: &'ast syn::TraitItemFn) {
            if let Some(b) = &f.default {
                self.add(&f.sig, b);
            }
            syn::visit::visit_trait_item_fn(self, f);
        }
    }
    let mut f = Find(Vec::new());
    f.visit_file(ast);
    f.0
}

/// A helper call: the helper, and the closure argument it hands the guard to
type HelperCall<'h, 'e> = (&'h Helper, Option<&'e Expr>);

/// A call's arguments
type Args = syn::punctuated::Punctuated<Expr, syn::Token![,]>;

/// The helper a method call is to
fn method_helper<'h, 'e>(
    helpers: &'h [Helper],
    m: &'e syn::ExprMethodCall,
) -> Option<HelperCall<'h, 'e>> {
    if m.turbofish.is_some() {
        return None;
    }
    helpers
        .iter()
        .find_map(|h| h.call(&m.method, &m.args, true))
}

/// The helper a path call is to (`locked(&m)`, `Self::locked(self)`)
fn call_helper<'h, 'e>(helpers: &'h [Helper], c: &'e syn::ExprCall) -> Option<HelperCall<'h, 'e>> {
    let Expr::Path(p) = &*c.func else { return None };
    let callee = &p.path.segments.last()?.ident;
    helpers.iter().find_map(|h| h.call(callee, &c.args, false))
}

/// The helper a call expression is to
fn helper_of<'h, 'e>(helpers: &'h [Helper], e: &'e Expr) -> Option<HelperCall<'h, 'e>> {
    match e {
        Expr::MethodCall(m) => method_helper(helpers, m),
        Expr::Call(c) => call_helper(helpers, c),
        _ => None,
    }
}

/// The call a value expression ends in, after peeling unwrap/expect/?/await; `None` for a decoder's `read()?`
fn peel(mut e: &Expr) -> Option<&Expr> {
    loop {
        match e {
            Expr::MethodCall(m)
                if ["unwrap", "expect", "unwrap_or_else"]
                    .contains(&m.method.to_string().as_str()) =>
            {
                e = &m.receiver;
            }
            Expr::Try(t) if decoder_read(&t.expr) => return None,
            Expr::Try(t) => e = &t.expr,
            Expr::Await(a) => e = &a.base,
            _ => return Some(e),
        }
    }
}

/// The call of one of `methods` a value expression ends in, after [`peel`]
fn guard_call<'e>(e: &'e Expr, methods: &[&str]) -> Option<&'e syn::ExprMethodCall> {
    match peel(e)? {
        Expr::MethodCall(m) if takes_lock(m, methods) => Some(m),
        _ => None,
    }
}

/// The line and name of the guard a value expression takes: a guard method's, or a call of a helper that returns
/// the guard
fn guard_taken(e: &Expr, helpers: &[Helper]) -> Option<(u32, String)> {
    let e = peel(e)?;
    if let Expr::MethodCall(m) = e
        && takes_lock(m, GUARD_METHODS)
    {
        return Some((
            m.method.span().start().line as u32,
            format!(".{}()", m.method),
        ));
    }
    let (h, None) = helper_of(helpers, e)? else {
        return None;
    };
    Some((e.span().start().line as u32, format!("{}()", h.name)))
}

/// Whether a call is a zero-argument call of one of `methods` with no turbofish
fn takes_lock(m: &syn::ExprMethodCall, methods: &[&str]) -> bool {
    m.args.is_empty() && m.turbofish.is_none() && methods.contains(&m.method.to_string().as_str())
}

/// Whether an expression under a `?` is a `read()` or `write()` call: a decoder's, as a lock's returns the guard,
/// a `LockResult` (unwrapped, not `?`-ed) or a future
fn decoder_read(e: &Expr) -> bool {
    matches!(e, Expr::MethodCall(m) if ["read", "write"].contains(&m.method.to_string().as_str()))
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
    helpers: &'a [Helper],
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
        if takes_lock(m, LOCK_METHODS) || method_helper(self.helpers, m).is_some() {
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
        if call_helper(self.helpers, c).is_some() {
            self.fx.lock.get_or_insert(c.span().start().line as u32);
        }
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

    fn visit_expr_try(&mut self, t: &'ast syn::ExprTry) {
        match &*t.expr {
            Expr::MethodCall(m) if decoder_read(&t.expr) => {
                self.visit_expr(&m.receiver);
                for a in &m.args {
                    self.visit_expr(a);
                }
            }
            _ => syn::visit::visit_expr_try(self, t),
        }
    }

    // a closure or async block defined here runs later, not while the guard lives
    fn visit_expr_closure(&mut self, _: &'ast syn::ExprClosure) {}
    fn visit_expr_async(&mut self, _: &'ast syn::ExprAsync) {}
    fn visit_item(&mut self, _: &'ast syn::Item) {}
}

impl Guards<'_> {
    /// What a stretch of code does while guard `name` (from `what` on line `at`) lives; whether it ended the guard's
    /// watch (a finding or its drop)
    fn effects(&mut self, name: &str, what: &str, at: u32, fx: &Effects) -> bool {
        let found = if let Some(l2) = fx.awaits {
            (
                "lock-held-across",
                Span::new(self.file, l2).label(".await"),
                format!("guard `{name}` (from {what} on line {at}) is held across an .await"),
            )
        } else if let Some((l2, call)) = &fx.io {
            (
                "lock-held-across",
                Span::new(self.file, *l2).label(call.clone()),
                format!("guard `{name}` (from {what} on line {at}) is held across I/O ({call})"),
            )
        } else if let Some(l2) = fx.lock {
            (
                "nested-locks",
                Span::new(self.file, l2),
                format!("a second lock is taken while guard `{name}` (line {at}) is alive"),
            )
        } else {
            return fx.drops.iter().any(|d| d == name);
        };
        self.found.push(found);
        true
    }

    fn scan(&self, visit: impl FnOnce(&mut Scan<'_>)) -> Effects {
        let mut sc = Scan {
            io: self.io,
            helpers: self.helpers,
            fx: Effects::default(),
        };
        visit(&mut sc);
        sc.fx
    }

    /// The closure handed to a helper that passes it the guard runs while the guard lives
    fn helper_closure(&mut self, call: Option<HelperCall<'_, '_>>, line: u32) {
        let Some((h, Some(Expr::Closure(c)))) = call else {
            return;
        };
        let name = c
            .inputs
            .first()
            .and_then(pat_name)
            .unwrap_or_else(|| "_".to_owned());
        let fx = self.scan(|sc| sc.visit_expr(&c.body));
        self.effects(&name, &format!("{}()", h.name), line, &fx);
    }
}

impl<'ast> Visit<'ast> for Guards<'_> {
    fn visit_expr_method_call(&mut self, m: &'ast syn::ExprMethodCall) {
        if let Some(inner) = lock_unwrap_call(m) {
            let at = pos(inner);
            if let Some(f) = self.frames.last_mut() {
                f.calls.push(self.unwraps.len());
            }
            self.unwraps.push(Unwrap {
                span: Span::new(self.file, at.0 as u32)
                    .label(format!(".{}().{}()", inner.method, m.method)),
                helper: None,
                at,
            });
        }
        self.helper_closure(
            method_helper(self.helpers, m),
            m.method.span().start().line as u32,
        );
        syn::visit::visit_expr_method_call(self, m);
    }

    fn visit_expr_call(&mut self, c: &'ast syn::ExprCall) {
        self.helper_closure(call_helper(self.helpers, c), c.span().start().line as u32);
        syn::visit::visit_expr_call(self, c);
    }

    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
        self.enter(f.sig.ident.to_string(), &f.sig, &f.block);
        syn::visit::visit_item_fn(self, f);
        self.exit();
    }

    fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
        self.enter(f.sig.ident.to_string(), &f.sig, &f.block);
        syn::visit::visit_impl_item_fn(self, f);
        self.exit();
    }

    fn visit_block(&mut self, b: &'ast syn::Block) {
        let stmts = &b.stmts;
        for (i, s) in stmts.iter().enumerate() {
            let Stmt::Local(l) = s else { continue };
            let Some(init) = &l.init else { continue };
            let Some((at, what)) = guard_taken(&init.expr, self.helpers) else {
                continue;
            };
            let Some(name) = pat_name(&l.pat) else {
                continue;
            };
            if name == "_" {
                continue; // `let _ = m.lock()` drops the guard at once
            }
            for later in &stmts[i + 1..] {
                let fx = self.scan(|sc| sc.visit_stmt(later));
                if self.effects(&name, &what, at, &fx) {
                    break;
                }
            }
        }
        syn::visit::visit_block(self, b);
    }
}
