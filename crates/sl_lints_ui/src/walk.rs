//! The walk over one file of a UI crate: every check the UI lints make on syntax.

use crate::paths::{Uses, is_literal, macro_args, method_in, number, path_in, size_numbers, texts};
use crate::text::{self, Edit};
use sl_analysis::Context;
use sl_analysis::syntax::{Range, range, type_idents};
use sl_core::Confidence;
use sl_model::Package;
use syn::visit::{self, Visit};
use syn::{Expr, Stmt};

/// One finding of a file
pub(crate) struct Found {
    pub lint: &'static str,
    pub range: Range,
    pub message: String,
    pub help: String,
    pub confidence: Confidence,
    /// An exact fix: what it does, and its edits
    pub fix: Option<(String, Vec<Edit>)>,
}

/// What a file gave
pub(crate) struct Out {
    pub found: Vec<Found>,
    /// Calls that start an app (`egui-shared-theme`)
    pub entries: Vec<Range>,
    /// Whether the file installs the shared theme
    pub has_setup: bool,
}

/// The parameters of the running lints, read once per file; a lint that is not on has empty lists
#[derive(Default)]
struct P {
    spawn_paths: Vec<String>,
    spawn_ui_types: Vec<String>,
    wrapper: String,
    block_paths: Vec<String>,
    block_methods: Vec<String>,
    cmd_paths: Vec<String>,
    cmd_methods: Vec<String>,
    spawners: Vec<String>,
    skip_fns: Vec<String>,
    block_ui_types: Vec<String>,
    colour_paths: Vec<String>,
    colour_consts: Vec<String>,
    colour_allowed: Vec<String>,
    colour_macros: Vec<String>,
    hex: bool,
    size_calls: Vec<String>,
    size_fields: Vec<String>,
    size_allowed: Vec<f64>,
    disable_calls: Vec<String>,
    reason_calls: Vec<String>,
    destructive_calls: Vec<String>,
    verbs: Vec<String>,
    confirm_words: Vec<String>,
    more_words: Vec<String>,
    fold_calls: Vec<String>,
    text_calls: Vec<String>,
    button_calls: Vec<String>,
    debug_calls: Vec<String>,
    unwrap_calls: Vec<String>,
    panic_methods: Vec<String>,
    entry_calls: Vec<String>,
    setup_calls: Vec<String>,
    glyph_calls: Vec<String>,
    glyphs: Vec<String>,
    glyph_replacement: String,
    id_calls: Vec<String>,
    unsalted: Vec<String>,
    salt_methods: Vec<String>,
    loop_methods: Vec<String>,
    repaint_calls: Vec<String>,
    repaint_after: Vec<String>,
    ui_types: Vec<String>,
    repaint_ms_min: f64,
    panel_paths: Vec<String>,
    context_types: Vec<String>,
    context_method: String,
    app_traits: Vec<String>,
    app_method: String,
    panel_replacement: String,
    menu_calls: Vec<String>,
    checkbox_calls: Vec<String>,
    min_checkboxes: usize,
    checklist_widget: String,
    tooltip_label_calls: Vec<String>,
    tooltip_hover_calls: Vec<String>,
}

impl P {
    fn read(cx: &Context<'_>, pkg: &Package, on: &[&str]) -> Self {
        let get = |id: &str| on.contains(&id).then(|| cx.params_for(id, pkg));
        let mut p = Self::default();
        if let Some(q) = get("ui-raw-spawn") {
            p.spawn_paths = q.list("functions").to_vec();
            p.wrapper = q.str("wrapper").to_owned();
            p.spawn_ui_types = q.list("ui-types").to_vec();
        }
        if let Some(q) = get("ui-blocking-call") {
            p.block_paths = q.list("functions").to_vec();
            p.block_methods = q.list("methods").to_vec();
            p.cmd_paths = q.list("command-paths").to_vec();
            p.cmd_methods = q.list("command-methods").to_vec();
            p.spawners = q.list("spawners").to_vec();
            p.skip_fns = q.list("skip-fns").to_vec();
            p.block_ui_types = q.list("ui-types").to_vec();
        }
        if let Some(q) = get("ui-colour-literal") {
            p.colour_paths = q.list("constructors").to_vec();
            p.colour_consts = q.list("consts").to_vec();
            p.colour_allowed = q.list("allowed").to_vec();
            p.colour_macros = q.list("macros").to_vec();
            p.hex = q.bool("hex-strings");
        }
        if let Some(q) = get("ui-size-literal") {
            p.size_calls = q.list("calls").to_vec();
            p.size_fields = q.list("fields").to_vec();
            p.size_allowed = q
                .list("allowed")
                .iter()
                .filter_map(|v| v.parse().ok())
                .collect();
        }
        if let Some(q) = get("ui-disabled-without-reason") {
            p.disable_calls = q.list("disable-calls").to_vec();
            p.reason_calls = q.list("reason-calls").to_vec();
        }
        if let Some(q) = get("ui-destructive-unconfirmed") {
            p.destructive_calls = q.list("calls").to_vec();
            p.verbs = q.list("verbs").to_vec();
            p.confirm_words = q
                .list("confirm-words")
                .iter()
                .map(|w| w.to_lowercase())
                .collect();
        }
        if let Some(q) = get("ui-more-count") {
            p.more_words = q.list("words").to_vec();
            p.fold_calls = q.list("fold-calls").to_vec();
        }
        if let Some(q) = get("ui-label-punctuation") {
            p.text_calls = q.list("calls").to_vec();
            p.button_calls = q.list("button-calls").to_vec();
        }
        if let Some(q) = get("ui-debug-text") {
            p.debug_calls = q.list("calls").to_vec();
        }
        if let Some(q) = get("ui-unwrap-external-data") {
            p.unwrap_calls = q.list("calls").to_vec();
            p.panic_methods = q.list("panic-methods").to_vec();
        }
        if let Some(q) = get("egui-shared-theme") {
            p.entry_calls = q.list("entry-calls").to_vec();
            p.setup_calls = q.list("setup-calls").to_vec();
        }
        if let Some(q) = get("egui-status-glyph") {
            p.glyph_calls = q.list("calls").to_vec();
            p.glyphs = q.list("glyphs").to_vec();
            p.glyph_replacement = q.str("replacement").to_owned();
        }
        if let Some(q) = get("egui-id-in-loop") {
            p.id_calls = q.list("id-calls").to_vec();
            p.unsalted = q.list("unsalted").to_vec();
            p.salt_methods = q.list("salt-methods").to_vec();
            p.loop_methods = q.list("loop-methods").to_vec();
        }
        if let Some(q) = get("egui-repaint-every-frame") {
            p.repaint_calls = q.list("calls").to_vec();
            p.repaint_after = q.list("after-calls").to_vec();
            p.ui_types = q.list("ui-types").to_vec();
            p.repaint_ms_min = q.int("interval-ms-min") as f64;
        }
        if let Some(q) = get("egui-panel-api") {
            p.panel_paths = q.list("constructors").to_vec();
            p.context_types = q.list("context-types").to_vec();
            p.context_method = q.str("context-method").to_owned();
            p.app_traits = q.list("app-traits").to_vec();
            p.app_method = q.str("app-method").to_owned();
            p.panel_replacement = q.str("replacement").to_owned();
        }
        if let Some(q) = get("egui-checklist-menu") {
            p.menu_calls = q.list("menu-calls").to_vec();
            p.checkbox_calls = q.list("checkbox-calls").to_vec();
            p.min_checkboxes = q.count("min-checkboxes");
            p.checklist_widget = q.str("widget").to_owned();
        }
        if let Some(q) = get("ui-tooltip-repeats-label") {
            p.tooltip_label_calls = q.list("label-calls").to_vec();
            p.tooltip_hover_calls = q.list("hover-calls").to_vec();
        }
        p
    }
}

/// The function being walked
struct FnScope {
    name: String,
    /// Some name in it shows a confirmation or an undo
    confirms: bool,
    /// It is UI code for `ui-raw-spawn`: it takes one of its `ui-types`, or that list is empty
    spawn_ui: bool,
    /// It is UI code for `ui-blocking-call`
    block_ui: bool,
}

/// What a call does with its closure arguments
#[derive(Clone, Copy, PartialEq, Eq)]
enum Arg {
    /// Runs them on the UI thread, outside any loop
    Plain,
    /// Runs them off the UI thread
    Spawned,
    /// Runs them once per item
    LoopBody,
}

/// One level of statements: whether the statement holds a reason call, whether a later one of its block does, and
/// whether the statement binds a value (`let r = ui.add_enabled(..);`, whose reason may follow on `r`)
#[derive(Clone, Copy)]
struct Reasons {
    here: bool,
    later: bool,
    binds: bool,
}

struct Walk<'a> {
    p: P,
    uses: Uses,
    text: &'a str,
    out: Out,
    fns: Vec<FnScope>,
    /// Inside a closure or async block that a spawner runs
    off_thread: u32,
    /// Inside a loop body, not inside a closure of it
    loop_depth: u32,
    /// Inside the receiver chain of a salt method
    salted: bool,
    reasons: Vec<Reasons>,
    /// The start of the method call that is a whole statement being walked (`ui.disable();`)
    bare: Option<(u32, u32)>,
    /// Inside the arguments of a fold call: its header may count what it holds
    in_fold: u32,
}

/// Walks one file of a UI crate for the lints in `on`
pub(crate) fn run(
    cx: &Context<'_>,
    pkg: &Package,
    on: &[&str],
    text: &str,
    file: &syn::File,
) -> Out {
    let mut w = Walk {
        p: P::read(cx, pkg, on),
        uses: Uses::of(file),
        text,
        out: Out {
            found: Vec::new(),
            entries: Vec::new(),
            has_setup: false,
        },
        fns: Vec::new(),
        off_thread: 0,
        loop_depth: 0,
        salted: false,
        reasons: Vec::new(),
        bare: None,
        in_fold: 0,
    };
    w.visit_file(file);
    w.out
}

impl Walk<'_> {
    fn push(
        &mut self,
        lint: &'static str,
        r: Range,
        message: String,
        help: String,
        confidence: Confidence,
    ) {
        self.found(Found {
            lint,
            range: r,
            message,
            help,
            confidence,
            fix: None,
        });
    }

    /// Records a finding, once per lint and place: a literal can be reached through two calls
    fn found(&mut self, f: Found) {
        if !self
            .out
            .found
            .iter()
            .any(|g| g.lint == f.lint && g.range == f.range)
        {
            self.out.found.push(f);
        }
    }

    /// Whether the code being walked is UI code for `ui-raw-spawn`
    fn spawn_scope(&self) -> bool {
        self.fns
            .last()
            .map_or(self.p.spawn_ui_types.is_empty(), |f| f.spawn_ui)
    }

    /// Whether the code being walked runs on the UI thread for `ui-blocking-call`
    fn block_scope(&self) -> bool {
        self.off_thread == 0
            && !self.in_skipped_fn()
            && self
                .fns
                .last()
                .map_or(self.p.block_ui_types.is_empty(), |f| f.block_ui)
    }

    fn in_skipped_fn(&self) -> bool {
        self.fns
            .last()
            .is_some_and(|f| self.p.skip_fns.contains(&f.name))
    }

    /// Whether a call's name is one of `spawners` (its closures run off the UI thread)
    fn is_spawner(&self, name: &str) -> bool {
        self.p.spawners.iter().any(|s| s == name)
    }

    /// Visits a call's argument: a closure or async block of a spawner runs off the UI thread; a closure of a loop
    /// method is a loop body; any other closure is neither
    fn visit_arg(&mut self, arg: &Expr, kind: Arg) {
        let spawner = kind == Arg::Spawned;
        let looping = kind == Arg::LoopBody;
        let body = match arg {
            Expr::Closure(c) => &*c.body,
            Expr::Async(_) if spawner => arg,
            _ => {
                self.visit_expr(arg);
                return;
            }
        };
        let saved = self.loop_depth;
        self.loop_depth = if looping { saved + 1 } else { 0 };
        self.off_thread += u32::from(spawner);
        self.visit_expr(body);
        self.off_thread -= u32::from(spawner);
        self.loop_depth = saved;
    }

    /// The checks of a call that shows text: punctuation, debug form, status glyphs, destructive labels
    fn check_texts(
        &mut self,
        name_matches: impl Fn(&[String]) -> bool,
        args: &[&Expr],
        call: Range,
    ) {
        let any = [
            &self.p.text_calls,
            &self.p.button_calls,
            &self.p.debug_calls,
            &self.p.glyph_calls,
            &self.p.destructive_calls,
        ]
        .iter()
        .any(|l| name_matches(l));
        if !any {
            return;
        }
        let mut literals = Vec::new();
        for a in args {
            texts(a, &mut literals);
        }
        let shown = name_matches(&self.p.text_calls);
        let button = name_matches(&self.p.button_calls);
        let debug = name_matches(&self.p.debug_calls);
        let glyph = name_matches(&self.p.glyph_calls);
        let destructive = name_matches(&self.p.destructive_calls);
        for lit in &literals {
            let v = lit.value();
            let r = range(lit);
            let mut edits = Vec::new();
            let mut what = Vec::new();
            if shown || button {
                let e = text::ellipsis_edits(self.text, lit);
                if !e.is_empty() {
                    what.push("writes the ellipsis as three dots");
                    edits.extend(e);
                }
            }
            if button && let Some(e) = text::full_stop_edit(self.text, lit) {
                what.push("ends a button label with a full stop");
                edits.push(e);
            }
            if !edits.is_empty() {
                self.found(Found {
                    lint: "ui-label-punctuation",
                    range: r,
                    message: format!("{v:?} {}", what.join(" and ")),
                    help:
                        "write \"…\" as one character; a button label is an action, not a sentence"
                            .into(),
                    confidence: Confidence::Certain,
                    fix: Some(("write \"…\", drop the full stop".into(), edits)),
                });
            }
            if debug && text::has_debug(&v) {
                self.push("ui-debug-text", r, format!("{v:?} shows a value in Rust's debug form to a person"), "format the value for people (`Display`, or the fields that matter); an error says what happened and what to do".into(), Confidence::Estimate);
            }
            if glyph
                && let Some(g) = self
                    .p
                    .glyphs
                    .iter()
                    .find(|g| v.contains(g.as_str()))
                    .cloned()
            {
                let rep = self.p.glyph_replacement.clone();
                self.push("egui-status-glyph", r, format!("status glyph {g} written into the text {v:?}"), format!("paint it in its tone with `{rep}`, beside the text, with a tooltip saying what it means"), Confidence::Certain);
            }
            if destructive
                && !self.fns.last().is_some_and(|f| f.confirms)
                && let Some(verb) = text::destructive_verb(&v, &self.p.verbs)
            {
                self.push("ui-destructive-unconfirmed", call, format!("{v:?} throws work away ({verb}) with no confirmation or undo in this function"), "ask first, saying what will be lost, or let it be undone for a few seconds; a label that opens a confirming dialog ends in \"…\"".into(), Confidence::Estimate);
            }
        }
    }

    /// The check of `ui-tooltip-repeats-label`: a hover call (`name`, one of `hover-calls`) whose text exactly
    /// repeats the label of the control reached through its receiver chain (a label call, one of `label-calls`)
    fn check_tooltip_label(&mut self, name: &str, receiver: &Expr, args: &[&Expr]) {
        if !method_in(&self.p.tooltip_hover_calls, name) {
            return;
        }
        let Some(&arg) = args.first() else { return };
        let Some(hover) = first_text(Some(arg)) else {
            return;
        };
        let Some(label) = label_text(receiver, &self.p.tooltip_label_calls, &self.uses) else {
            return;
        };
        if label == hover {
            self.push(
                "ui-tooltip-repeats-label",
                range(arg),
                format!("{label:?} tooltip only repeats the control's label"),
                "say what the label cannot: what it does, a shortcut, a constraint, or what happens next".into(),
                Confidence::Certain,
            );
        }
    }

    fn check_sizes(&mut self, what: &str, args: &[&Expr]) {
        let mut nums = Vec::new();
        for a in args {
            size_numbers(a, &mut nums);
        }
        for (v, r) in nums {
            if !self.p.size_allowed.contains(&v) {
                self.push("ui-size-literal", r, format!("{what} takes the size {v} as a number"), "take it from the theme's tokens (spacing, radius, stroke, type scale), or learn a width from the content".into(), Confidence::Certain);
            }
        }
    }

    fn check_disable(&mut self, name: &str, args: &[&Expr], r: Range) {
        if !method_in(&self.p.disable_calls, name) {
            return;
        }
        if args.first().is_some_and(
            |a| matches!(a, Expr::Lit(syn::ExprLit { lit: syn::Lit::Bool(b), .. }) if b.value),
        ) {
            return;
        }
        let here = self
            .reasons
            .last()
            .is_some_and(|x| x.here || (x.binds && x.later));
        // `ui.disable();` disables what follows it: the reasons may come later in its block
        let region = self.bare == Some(r.0)
            && args.len() <= 1
            && !args.iter().any(|a| matches!(a, Expr::Closure(_)));
        let later = region && self.reasons.iter().any(|x| x.later);
        if here || later {
            return;
        }
        let reason = self.p.reason_calls.first().cloned().unwrap_or_default();
        self.push(
            "ui-disabled-without-reason",
            r,
            format!("`{name}` disables controls without saying why"),
            format!(
                "attach the reason and what would enable them (`{reason}`) to the disabled control"
            ),
            Confidence::Estimate,
        );
    }

    /// The repaint checks of a function that runs every frame: an unconditional request among its statements
    fn check_repaint(&mut self, block: &syn::Block) {
        for s in &block.stmts {
            let Stmt::Expr(Expr::MethodCall(m), _) = s else {
                continue;
            };
            let name = m.method.to_string();
            if self.p.repaint_calls.contains(&name) && m.args.is_empty() {
                self.push("egui-repaint-every-frame", range(m), format!("`{name}()` on every frame keeps the UI repainting without pause"), "request a repaint where something changed (a worker that has news, an animation still running), or `request_repaint_after` a delay".into(), Confidence::Estimate);
            }
            if self.p.repaint_after.contains(&name)
                && let Some(ms) = m.args.first().and_then(duration_ms)
                && ms < self.p.repaint_ms_min
            {
                self.push("egui-repaint-every-frame", range(m), format!("`{name}` every {ms} ms on every frame is a busy loop"), "repaint where something changed, or wait at least the `interval-ms-min` parameter".into(), Confidence::Estimate);
            }
        }
    }
}

/// The milliseconds of a `Duration::from_*` with a literal
fn duration_ms(e: &Expr) -> Option<f64> {
    let Expr::Call(c) = e else { return None };
    let Expr::Path(p) = &*c.func else { return None };
    let f = p.path.segments.last()?.ident.to_string();
    let v = number(c.args.first()?)?;
    Some(match f.as_str() {
        "from_millis" => v,
        "from_micros" => v / 1000.0,
        "from_nanos" => v / 1e6,
        "from_secs" | "from_secs_f32" | "from_secs_f64" => v * 1000.0,
        _ => return None,
    })
}

/// The label text of a control-building call reachable from `e`: a call matching `label_calls` (a method, or a
/// function `Type::new`), read from inside out, so a builder wrapped in another call (`ui.add(Button::new("x"))`)
/// is still found
fn label_text(e: &Expr, label_calls: &[String], uses: &Uses) -> Option<String> {
    if label_calls.is_empty() {
        return None;
    }
    match e {
        Expr::Call(c) => {
            if let Expr::Path(p) = &*c.func
                && path_in(label_calls, &uses.resolve(&p.path))
                && let Some(l) = first_text(c.args.first())
            {
                return Some(l);
            }
            c.args.iter().find_map(|a| label_text(a, label_calls, uses))
        }
        Expr::MethodCall(m) => {
            let name = m.method.to_string();
            if method_in(label_calls, &name)
                && let Some(l) = first_text(m.args.first())
            {
                return Some(l);
            }
            label_text(&m.receiver, label_calls, uses)
                .or_else(|| m.args.iter().find_map(|a| label_text(a, label_calls, uses)))
        }
        Expr::Reference(r) => label_text(&r.expr, label_calls, uses),
        Expr::Paren(p) => label_text(&p.expr, label_calls, uses),
        Expr::Group(g) => label_text(&g.expr, label_calls, uses),
        _ => None,
    }
}

/// The value of the first string literal `texts` finds in an argument
fn first_text(arg: Option<&Expr>) -> Option<String> {
    let mut list = Vec::new();
    texts(arg?, &mut list);
    list.first().map(syn::LitStr::value)
}

/// Whether a statement holds a call of one of `names`
fn has_call(s: &Stmt, names: &[String]) -> bool {
    struct V<'a>(&'a [String], bool);
    impl<'ast> Visit<'ast> for V<'_> {
        fn visit_expr_method_call(&mut self, m: &'ast syn::ExprMethodCall) {
            self.1 |= method_in(self.0, &m.method.to_string());
            visit::visit_expr_method_call(self, m);
        }
        fn visit_macro(&mut self, m: &'ast syn::Macro) {
            for a in macro_args(m) {
                self.visit_expr(&a);
            }
        }
    }
    let mut v = V(names, false);
    v.visit_stmt(s);
    v.1
}

/// The number of `checkbox-calls` method calls inside `body`
fn checkbox_count(body: &Expr, methods: &[String]) -> usize {
    struct V<'a>(&'a [String], usize);
    impl<'ast> Visit<'ast> for V<'_> {
        fn visit_expr_method_call(&mut self, m: &'ast syn::ExprMethodCall) {
            self.1 += usize::from(method_in(self.0, &m.method.to_string()));
            visit::visit_expr_method_call(self, m);
        }
        fn visit_macro(&mut self, m: &'ast syn::Macro) {
            for a in macro_args(m) {
                self.visit_expr(&a);
            }
        }
    }
    let mut v = V(methods, 0);
    v.visit_expr(body);
    v.1
}

/// Whether a function body names a confirmation or an undo
fn confirms(block: &syn::Block, words: &[String]) -> bool {
    struct V<'a>(&'a [String], bool);
    impl<'ast> Visit<'ast> for V<'_> {
        fn visit_ident(&mut self, i: &'ast proc_macro2::Ident) {
            let s = i.to_string().to_lowercase();
            self.1 |= self.0.iter().any(|w| s.contains(w.as_str()));
        }
        fn visit_macro(&mut self, m: &'ast syn::Macro) {
            visit::visit_macro(self, m);
            for a in macro_args(m) {
                self.visit_expr(&a);
            }
        }
    }
    if words.is_empty() {
        return false;
    }
    let mut v = V(words, false);
    v.visit_block(block);
    v.1
}

/// Whether a function takes a parameter of one of `types` (it runs every frame)
fn takes_ui(sig: &syn::Signature, types: &[String]) -> bool {
    sig.inputs.iter().any(|i| match i {
        syn::FnArg::Typed(t) => type_idents(&t.ty).iter().any(|n| types.contains(n)),
        syn::FnArg::Receiver(_) => false,
    })
}

impl<'ast> Visit<'ast> for Walk<'_> {
    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
        self.enter_fn(&f.sig, &f.block);
        visit::visit_item_fn(self, f);
        self.fns.pop();
    }

    fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
        self.enter_fn(&f.sig, &f.block);
        visit::visit_impl_item_fn(self, f);
        self.fns.pop();
    }

    fn visit_item_impl(&mut self, i: &'ast syn::ItemImpl) {
        if let Some((path, _)) = &i.trait_
            && let Some(trait_name) = self.uses.resolve(path).last().cloned()
            && self
                .p
                .app_traits
                .iter()
                .any(|a| a.rsplit("::").next() == Some(trait_name.as_str()))
        {
            for item in &i.items {
                if let syn::ImplItem::Fn(f) = item
                    && f.sig.ident == self.p.app_method
                {
                    self.push(
                        "egui-panel-api",
                        range(&f.sig.ident),
                        format!(
                            "`{trait_name}::{}` is the egui version in use replaced",
                            f.sig.ident
                        ),
                        format!("implement `{}`", self.p.panel_replacement),
                        Confidence::Estimate,
                    );
                }
            }
        }
        visit::visit_item_impl(self, i);
    }

    fn visit_block(&mut self, b: &'ast syn::Block) {
        let track = !self.p.disable_calls.is_empty();
        for (i, s) in b.stmts.iter().enumerate() {
            if track {
                let here = has_call(s, &self.p.reason_calls);
                let later = b.stmts[i + 1..]
                    .iter()
                    .any(|t| has_call(t, &self.p.reason_calls));
                let binds = matches!(s, Stmt::Local(_));
                self.reasons.push(Reasons { here, later, binds });
            }
            self.bare = match s {
                Stmt::Expr(Expr::MethodCall(m), Some(_)) => Some(range(&m.dot_token).0),
                _ => None,
            };
            self.visit_stmt(s);
            if track {
                self.reasons.pop();
            }
        }
    }

    fn visit_expr_closure(&mut self, c: &'ast syn::ExprClosure) {
        let saved = self.loop_depth;
        self.loop_depth = 0;
        visit::visit_expr_closure(self, c);
        self.loop_depth = saved;
    }

    fn visit_expr_for_loop(&mut self, f: &'ast syn::ExprForLoop) {
        self.visit_expr(&f.expr);
        self.loop_depth += 1;
        self.visit_block(&f.body);
        self.loop_depth -= 1;
    }

    fn visit_expr_while(&mut self, w: &'ast syn::ExprWhile) {
        self.visit_expr(&w.cond);
        self.loop_depth += 1;
        self.visit_block(&w.body);
        self.loop_depth -= 1;
    }

    fn visit_expr_loop(&mut self, l: &'ast syn::ExprLoop) {
        self.loop_depth += 1;
        self.visit_block(&l.body);
        self.loop_depth -= 1;
    }

    fn visit_expr_call(&mut self, c: &'ast syn::ExprCall) {
        let r = range(c);
        let args: Vec<&Expr> = c.args.iter().collect();
        let mut name = String::new();
        if let Expr::Path(fp) = &*c.func {
            let segs = self.uses.resolve(&fp.path);
            name = segs.last().cloned().unwrap_or_default();
            let shown = segs.join("::");
            if path_in(&self.p.spawn_paths, &segs) && self.spawn_scope() {
                let help = if self.p.wrapper.is_empty() {
                    "start it through the project's spawn wrapper, which names the work, reports a crash and repaints the UI".to_owned()
                } else {
                    format!(
                        "start it through `{}`, which names the work, reports a crash and repaints the UI",
                        self.p.wrapper
                    )
                };
                self.push(
                    "ui-raw-spawn",
                    r,
                    format!("`{shown}` starts work outside the spawn wrapper"),
                    help,
                    Confidence::Certain,
                );
            }
            if path_in(&self.p.block_paths, &segs) && self.block_scope() {
                self.push("ui-blocking-call", r, format!("`{shown}` blocks; called from UI code it freezes the UI until it returns"), "run it in a worker and poll the result each frame; the worker asks for a repaint when it has news".into(), Confidence::Estimate);
            }
            if path_in(&self.p.colour_paths, &segs)
                && !args.is_empty()
                && args.iter().all(|a| is_literal(a))
            {
                self.push(
                    "ui-colour-literal",
                    r,
                    format!("`{shown}` writes a colour as values"),
                    "take the colour from the theme's tokens; a new colour is added to the theme"
                        .into(),
                    Confidence::Certain,
                );
            }
            if path_in(&self.p.size_calls, &segs) {
                self.check_sizes(&format!("`{shown}`"), &args);
            }
            if path_in(&self.p.id_calls, &segs)
                && self.loop_depth > 0
                && !self.salted
                && args.first().is_some_and(|a| is_literal(a))
            {
                self.push("egui-id-in-loop", r, format!("`{shown}` gets the same id on every iteration of the loop"), "salt the id with the item (`id_salt((\"name\", i))`, `ui.push_id(i, ..)`), so each iteration keeps its own state".into(), Confidence::Estimate);
            }
            if path_in(&self.p.unsalted, &segs) && self.loop_depth > 0 && !self.salted {
                self.push(
                    "egui-id-in-loop",
                    r,
                    format!("`{shown}` in a loop takes the same id on every iteration"),
                    "give it `id_salt` with the item, so each iteration keeps its own scroll state"
                        .into(),
                    Confidence::Estimate,
                );
            }
            if path_in(&self.p.entry_calls, &segs) {
                self.out.entries.push(r);
            }
            if path_in(&self.p.setup_calls, &segs) {
                self.out.has_setup = true;
            }
            if path_in(&self.p.panel_paths, &segs) {
                self.push(
                    "egui-panel-api",
                    r,
                    format!("`{shown}` is panel code the egui version in use replaced"),
                    format!("use `{}`", self.p.panel_replacement),
                    Confidence::Estimate,
                );
            }
            self.check_texts(|l| path_in(l, &segs), &args, r);
        }
        self.visit_expr(&c.func);
        let kind = if self.is_spawner(&name) {
            Arg::Spawned
        } else {
            Arg::Plain
        };
        let fold = match &*c.func {
            Expr::Path(fp) => u32::from(path_in(&self.p.fold_calls, &self.uses.resolve(&fp.path))),
            _ => 0,
        };
        let salted = std::mem::replace(&mut self.salted, false);
        self.in_fold += fold;
        for a in &args {
            self.visit_arg(a, kind);
        }
        self.in_fold -= fold;
        self.salted = salted;
    }

    fn visit_expr_method_call(&mut self, m: &'ast syn::ExprMethodCall) {
        let name = m.method.to_string();
        let r = (range(&m.dot_token).0, range(&m.paren_token.span.join()).1);
        let args: Vec<&Expr> = m.args.iter().collect();
        if method_in(&self.p.block_methods, &name) && self.block_scope() {
            self.push(
                "ui-blocking-call",
                r,
                format!(
                    "`.{name}()` blocks; called from UI code it freezes the UI until it returns"
                ),
                "use the non-blocking form and poll it each frame, or move the work to a worker"
                    .into(),
                Confidence::Estimate,
            );
        }
        if method_in(&self.p.cmd_methods, &name)
            && self.block_scope()
            && self.command_root(&m.receiver)
        {
            self.push("ui-blocking-call", r, format!("`.{name}()` waits for a process; called from UI code it freezes the UI until the process ends"), "run the process in a worker and poll its result each frame".into(), Confidence::Estimate);
        }
        if method_in(&self.p.size_calls, &name) {
            self.check_sizes(&format!("`.{name}()`"), &args);
        }
        if name == self.p.context_method && self.chain_root(&m.receiver, &self.p.context_types) {
            self.push(
                "egui-panel-api",
                r,
                format!("`.{name}()` shows a panel directly on a Context; the egui version in use replaced it with `.show_inside` on a Ui"),
                format!("use `{}`", self.p.panel_replacement),
                Confidence::Estimate,
            );
        }
        if method_in(&self.p.menu_calls, &name)
            && let Some(Expr::Closure(c)) = args.last()
            && checkbox_count(&c.body, &self.p.checkbox_calls) >= self.p.min_checkboxes
        {
            self.push(
                "egui-checklist-menu",
                r,
                format!("`.{name}()` holds several checkboxes that close the menu on each click"),
                format!(
                    "use `{}`, which keeps the menu open while ticking",
                    self.p.checklist_widget
                ),
                Confidence::Estimate,
            );
        }
        if method_in(&self.p.panic_methods, &name)
            && self.chain_root(&m.receiver, &self.p.unwrap_calls)
        {
            self.push(
                "ui-unwrap-external-data",
                r,
                format!("`.{name}()` panics on a bad file, parse or connection"),
                "return the error (`?` from a function that returns `Result`) or show it to the person; a UI binary crashing on bad input is not a library's panic-in-library exemption".into(),
                Confidence::Estimate,
            );
        }
        self.check_disable(&name, &args, r);
        if method_in(&self.p.id_calls, &name)
            && self.loop_depth > 0
            && args.first().is_some_and(|a| is_literal(a))
        {
            self.push("egui-id-in-loop", r, format!("`.{name}()` gives the same id on every iteration of the loop"), "salt the id with the item (`(\"name\", i)`), so each iteration keeps its own state".into(), Confidence::Estimate);
        }
        self.check_texts(|l| method_in(l, &name), &args, r);
        self.check_tooltip_label(&name, &m.receiver, &args);
        let salted = self.salted;
        self.salted = salted || method_in(&self.p.salt_methods, &name);
        self.visit_expr(&m.receiver);
        self.salted = false;
        let kind = if self.is_spawner(&name) {
            Arg::Spawned
        } else if self.p.loop_methods.contains(&name) {
            Arg::LoopBody
        } else {
            Arg::Plain
        };
        let fold = u32::from(method_in(&self.p.fold_calls, &name));
        self.in_fold += fold;
        for a in &args {
            self.visit_arg(a, kind);
        }
        self.in_fold -= fold;
        self.salted = salted;
    }

    fn visit_expr_assign(&mut self, a: &'ast syn::ExprAssign) {
        if !self.p.size_fields.is_empty() {
            let mut fields = Vec::new();
            let mut e = &*a.left;
            while let Expr::Field(f) = e {
                if let syn::Member::Named(n) = &f.member {
                    fields.push(n.to_string());
                }
                e = &f.base;
            }
            if let Some(f) = fields.iter().find(|f| self.p.size_fields.contains(f)) {
                let f = f.clone();
                self.check_sizes(&format!("`{f}`"), &[&*a.right]);
            }
        }
        visit::visit_expr_assign(self, a);
    }

    fn visit_expr_path(&mut self, p: &'ast syn::ExprPath) {
        if !self.p.colour_consts.is_empty() {
            let segs = self.uses.resolve(&p.path);
            let constant = segs.last().is_some_and(|s| {
                s.chars().any(|c| c.is_ascii_uppercase())
                    && s.chars()
                        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
            });
            if constant
                && path_in(&self.p.colour_consts, &segs)
                && !path_in(&self.p.colour_allowed, &segs)
            {
                let shown = segs.join("::");
                self.push("ui-colour-literal", range(p), format!("`{shown}` is a fixed colour"), "take the colour from the theme's tokens, which follow the light and dark theme".into(), Confidence::Certain);
            }
        }
        visit::visit_expr_path(self, p);
    }

    fn visit_attribute(&mut self, _: &'ast syn::Attribute) {}

    fn visit_lit_str(&mut self, l: &'ast syn::LitStr) {
        let v = l.value();
        if self.p.hex && is_hex_colour(&v) {
            self.push(
                "ui-colour-literal",
                range(l),
                format!("{v:?} writes a colour as a value"),
                "take the colour from the theme's tokens; a new colour is added to the theme"
                    .into(),
                Confidence::Certain,
            );
        }
        if self.in_fold == 0 && text::more_count(&v, &self.p.more_words) {
            self.push("ui-more-count", range(l), format!("{v:?} says how many items are not shown instead of showing them"), "end the cut list in a fold, collapsed by default, whose header says what is inside and which opens to every item".into(), Confidence::Estimate);
        }
    }

    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        let name = m
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();
        if name == "macro_rules" {
            return;
        }
        if self.p.colour_macros.contains(&name) {
            self.push(
                "ui-colour-literal",
                range(m),
                format!("`{name}!` writes a colour as a value"),
                "take the colour from the theme's tokens; a new colour is added to the theme"
                    .into(),
                Confidence::Certain,
            );
            return;
        }
        for a in macro_args(m) {
            self.visit_expr(&a);
        }
    }
}

impl Walk<'_> {
    fn enter_fn(&mut self, sig: &syn::Signature, block: &syn::Block) {
        self.fns.push(FnScope {
            name: sig.ident.to_string(),
            confirms: confirms(block, &self.p.confirm_words),
            spawn_ui: self.p.spawn_ui_types.is_empty() || takes_ui(sig, &self.p.spawn_ui_types),
            block_ui: self.p.block_ui_types.is_empty() || takes_ui(sig, &self.p.block_ui_types),
        });
        if !self.p.repaint_calls.is_empty() && takes_ui(sig, &self.p.ui_types) {
            self.check_repaint(block);
        }
    }

    /// Whether a method chain starts at a process command constructor (`Command::new("git").arg(..)`)
    fn command_root(&self, mut e: &Expr) -> bool {
        while let Expr::MethodCall(m) = e {
            e = &m.receiver;
        }
        match e {
            Expr::Call(c) => match &*c.func {
                Expr::Path(p) => path_in(&self.p.cmd_paths, &self.uses.resolve(&p.path)),
                _ => false,
            },
            _ => false,
        }
    }

    /// Whether a chain of method calls, tries, parens and references is rooted in a call matching `list`
    /// (`std::fs::read_to_string(path).unwrap()`, `egui::CentralPanel::default().frame(f).show(ctx, ..)`)
    fn chain_root(&self, mut e: &Expr, list: &[String]) -> bool {
        loop {
            e = match e {
                Expr::MethodCall(m) => &m.receiver,
                Expr::Try(t) => &t.expr,
                Expr::Paren(p) => &p.expr,
                Expr::Reference(r) => &r.expr,
                Expr::Call(c) => {
                    return match &*c.func {
                        Expr::Path(p) => path_in(list, &self.uses.resolve(&p.path)),
                        _ => false,
                    };
                }
                _ => return false,
            };
        }
    }
}

/// Whether a string is a CSS colour: `#` and 3, 4, 6 or 8 hex digits
fn is_hex_colour(v: &str) -> bool {
    v.strip_prefix('#').is_some_and(|h| {
        matches!(h.len(), 3 | 4 | 6 | 8) && h.chars().all(|c| c.is_ascii_hexdigit())
    })
}
