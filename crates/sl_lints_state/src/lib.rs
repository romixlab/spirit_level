//! Shared-state lints, one analysis: [`STATE`].

mod guards;

use sl_analysis::syntax::{line, type_idents, without_tests};
use sl_analysis::{Analysis, Context, lint};
use sl_core::{Confidence, Diagnostic, LintMeta, Span};
use sl_model::FileKind;
use syn::visit::Visit;

lint!(STATIC_INTERIOR_MUTABILITY, "T0004", "static-interior-mutability", "state",
    "a static with interior mutability has one owner, or a reason why a global lock is right",
    params: [
        ("types", StrList, "[\"Mutex\", \"RwLock\", \"RefCell\", \"Cell\", \"UnsafeCell\", \"OnceCell\", \"DashMap\", \"ArcSwap\", \"ReentrantMutex\"]", "type names that make a static mutable"),
        ("atomics", Bool, "false", "also report statics of atomic types (counters and flags are usually fine)"),
    ],
    needs: [], fix: None);
lint!(STATIC_MUT, "T0005", "static-mut", "state", "no static mut",
    params: [], needs: [], fix: None);
lint!(THREAD_LOCAL_STATE, "T0006", "thread-local-state", "state", "thread_local! state has one owner, or a reason",
    params: [("types", StrList, "[\"Cell\", \"RefCell\", \"Mutex\", \"RwLock\", \"UnsafeCell\", \"OnceCell\"]", "type names that make a thread-local mutable")],
    needs: [], fix: None);
lint!(LOCK_UNWRAP, "T0002", "lock-unwrap", "state", "a crate keeps its lock().unwrap() calls under a limit",
    params: [("max", Int, "0", "calls allowed per crate before it is reported")],
    needs: [], fix: None);
lint!(LOCK_HELD_ACROSS, "T0001", "lock-held-across", "state", "no lock guard is held across an .await or an I/O call",
    params: [("io", StrList, "[\"File\", \"fs\", \"Command\", \"TcpStream\", \"UdpSocket\", \"UnixStream\", \"write_all\", \"read_to_end\", \"read_to_string\", \"flush\", \"sync_all\", \"output\", \"spawn\", \"send\", \"recv\", \"block_on\", \"sleep\"]", "path segments and method names counted as I/O")],
    needs: [], fix: None);
lint!(NESTED_LOCKS, "T0003", "nested-locks", "state", "no second lock is taken while a guard is alive",
    params: [], needs: [], fix: None);

/// The shared-state analysis
pub struct State;

/// The registered instance
pub static STATE: State = State;

static LINTS: &[&LintMeta] = &[
    &STATIC_INTERIOR_MUTABILITY,
    &STATIC_MUT,
    &THREAD_LOCAL_STATE,
    &LOCK_UNWRAP,
    &LOCK_HELD_ACROSS,
    &NESTED_LOCKS,
];

impl Analysis for State {
    fn name(&self) -> &'static str {
        "state"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        let p = cx.project;
        let types = cx
            .params("static-interior-mutability")
            .list("types")
            .to_vec();
        let atomics = cx.params("static-interior-mutability").bool("atomics");
        let tl_types = cx.params("thread-local-state").list("types").to_vec();
        let io = cx.params("lock-held-across").list("io").to_vec();
        for pkg in &p.packages {
            let mut unwraps: Vec<Span> = Vec::new();
            for f in p
                .sources(pkg)
                .iter()
                .filter(|f| f.kind == FileKind::Src && !p.is_test_file(pkg, f))
            {
                let Ok(ast) = f.ast() else { continue };
                let ast = without_tests(ast);
                let mut v = Items {
                    cx,
                    file: &f.rel,
                    krate: &pkg.name,
                    module: &f.module,
                    types: &types,
                    atomics,
                    tl_types: &tl_types,
                };
                v.visit_file(&ast);
                let mut g = guards::Guards::new(&f.rel, &io);
                g.visit_file(&ast);
                unwraps.extend(g.unwraps);
                for (lint, s, msg) in g.found {
                    cx.emit(
                        Diagnostic::new(lint, format!("{}:{}:{}", pkg.name, f.rel, s.line), msg)
                            .span(s)
                            .krate(&pkg.name)
                            .confidence(Confidence::Estimate)
                            .help(if lint == "nested-locks" {
                                "one owner for both pieces of state (an owner task), or take them one after the other; nested locks need a global lock order"
                            } else {
                                "drop the guard before it (copy what is needed out, or end the scope), or move the state to an owner task that does the I/O itself"
                            }),
                    );
                }
            }
            let max = cx.params_for("lock-unwrap", pkg).count("max");
            if unwraps.len() > max {
                let n = unwraps.len();
                let mut d = Diagnostic::new(
                    "lock-unwrap",
                    format!("crate:{}", pkg.name),
                    format!("{} has {n} lock().unwrap()-style calls (limit {max})", pkg.name),
                )
                .krate(&pkg.name)
                .metric("calls", n as f64, Some(max as f64))
                .help("each is a lock reached by many callers and a panic on poison; give the state one owner, or take kept locks through one helper that recovers from poison");
                for s in unwraps.into_iter().take(20) {
                    d = d.span(s);
                }
                cx.emit(d);
            }
        }
    }
}

struct Items<'a> {
    cx: &'a Context<'a>,
    file: &'a str,
    krate: &'a str,
    module: &'a str,
    types: &'a [String],
    atomics: bool,
    tl_types: &'a [String],
}

impl Items<'_> {
    fn item(&self, name: &str) -> String {
        if self.module.is_empty() {
            format!("{}::{name}", self.krate)
        } else {
            format!("{}::{}::{name}", self.krate, self.module)
        }
    }
}

impl<'ast> Visit<'ast> for Items<'_> {
    fn visit_item_static(&mut self, s: &'ast syn::ItemStatic) {
        let name = s.ident.to_string();
        let at = Span::new(self.file, line(s)).label(format!("static {name}"));
        if matches!(s.mutability, syn::StaticMutability::Mut(_)) {
            self.cx.emit(
                Diagnostic::new("static-mut", self.item(&name), format!("static mut {name} is global mutable state with no synchronisation"))
                    .span(at.clone())
                    .krate(self.krate)
                    .help("give the state one owner and pass it by value or &mut; a counter or flag is an atomic"),
            );
        }
        let idents = type_idents(&s.ty);
        let hit = idents.iter().find(|i| {
            self.types.iter().any(|t| t == *i) || (self.atomics && i.starts_with("Atomic"))
        });
        if let Some(t) = hit {
            self.cx.emit(
                Diagnostic::new(
                    "static-interior-mutability",
                    self.item(&name),
                    format!("static {name} holds a {t}: shared mutable state every caller reaches"),
                )
                .span(at)
                .krate(self.krate)
                .help("give it one owner (by value on a context, an owner task, a swapped snapshot, set once); if a lock is right here, say why with `// spirit-level: allow(static-interior-mutability) <reason>`"),
            );
        }
        syn::visit::visit_item_static(self, s);
    }

    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        if m.path
            .segments
            .last()
            .is_some_and(|s| s.ident == "thread_local")
        {
            let tokens = m.tokens.to_string();
            let words: Vec<&str> = tokens
                .split(|c: char| !c.is_alphanumeric() && c != '_')
                .collect();
            if let Some(t) = self.tl_types.iter().find(|t| words.contains(&t.as_str())) {
                let start = m
                    .path
                    .segments
                    .first()
                    .map_or(0, |s| s.ident.span().start().line as u32);
                let name = words
                    .windows(2)
                    .find(|w| w[0] == "static")
                    .map_or("thread_local", |w| w[1]);
                self.cx.emit(
                    Diagnostic::new(
                        "thread-local-state",
                        self.item(name),
                        format!("thread_local! {name} holds a {t}: hidden per-thread state"),
                    )
                    .span(Span::new(self.file, start))
                    .krate(self.krate)
                    .help("pass the state down the call tree; a per-thread buffer that only its thread touches is fine with a reason in a waiver"),
                );
            }
        }
        syn::visit::visit_macro(self, m);
    }
}
