//! UI lints, one analysis: [`UI`].
//!
//! The `ui` lints know no toolkit: the calls they look for are parameters, which the `egui` tier fills with egui's.
//! The `egui` lints exist only for egui. All of them look at UI crates only (see [`is_ui_crate`]), never at tests.

mod paths;
mod text;
mod walk;

use sl_analysis::Context;
use sl_core::{Confidence, Diagnostic, LintMeta, Span, glob};
use sl_model::{DepKind, FileKind, Package};

/// Declares a UI lint: [`sl_analysis::lint!`] plus the `toolkits` parameter every UI lint has
macro_rules! ui_lint {
    ($name:ident, $code:literal, $id:literal, $cat:literal, $summary:literal, params: [$($p:tt),* $(,)?], fix: $fix:ident) => {
        sl_analysis::lint!($name, $code, $id, $cat, $summary,
            params: [$($p,)* ("toolkits", StrList, "[\"egui\", \"eframe\", \"iced\", \"slint\", \"gtk4\", \"relm4\", \"dioxus\", \"leptos\", \"yew\", \"floem\", \"xilem\", \"makepad-widgets\"]", "UI toolkits: a crate that depends on one directly is a UI crate, unless the crate map says `ui = false`")],
            needs: [], fix: $fix);
    };
}

ui_lint!(RAW_SPAWN, "U0001", "ui-raw-spawn", "ui", "UI code starts threads and tasks only through the project's spawn wrapper",
    params: [
        ("functions", StrList, "[\"std::thread::spawn\", \"std::thread::Builder::new\", \"tokio::spawn\", \"tokio::task::spawn\", \"tokio::task::spawn_blocking\", \"tokio::task::spawn_local\", \"async_std::task::spawn\", \"smol::spawn\", \"rayon::spawn\", \"wasm_bindgen_futures::spawn_local\"]", "functions that start a thread or a task"),
        ("wrapper", Str, "\"\"", "the project's spawn wrapper, named in the help"),
        ("ui-types", StrList, "[]", "parameter types of a function that is UI code; empty: every function of a UI crate"),
    ],
    fix: None);
ui_lint!(BLOCKING_CALL, "U0002", "ui-blocking-call", "ui", "UI code never blocks the UI thread on files, the network, processes or sleeps",
    params: [
        ("functions", StrList, "[\"std::thread::sleep\", \"std::fs::read\", \"std::fs::read_to_string\", \"std::fs::write\", \"std::fs::read_dir\", \"std::fs::copy\", \"std::fs::rename\", \"std::fs::remove_file\", \"std::fs::remove_dir_all\", \"std::fs::create_dir_all\", \"std::fs::metadata\", \"std::fs::File::open\", \"std::fs::File::create\", \"std::net::TcpStream::connect\", \"reqwest::blocking::get\", \"ureq::get\", \"ureq::post\", \"futures::executor::block_on\", \"pollster::block_on\"]", "functions that block until a file, a peer, a process or a timer is done"),
        ("methods", StrList, "[\"blocking_read\", \"blocking_write\", \"blocking_lock\", \"blocking_recv\", \"blocking_send\", \"block_on\"]", "methods that block"),
        ("command-paths", StrList, "[\"std::process::Command::new\"]", "constructors of a process command"),
        ("command-methods", StrList, "[\"output\", \"status\", \"wait\", \"wait_with_output\"]", "methods of a command chain that wait for the process"),
        ("spawners", StrList, "[\"spawn\", \"spawn_blocking\", \"spawn_local\", \"execute\", \"scope\"]", "calls whose closures and async blocks run off the UI thread"),
        ("skip-fns", StrList, "[\"main\"]", "functions that run before the UI does"),
        ("ui-types", StrList, "[]", "parameter types of a function that runs on the UI thread; empty: every function of a UI crate"),
    ],
    fix: None);
ui_lint!(COLOUR_LITERAL, "U0003", "ui-colour-literal", "ui", "UI code takes its colours from the theme, never writes them as values",
    params: [
        ("constructors", StrList, "[]", "functions that build a colour from values (`Type::from_*`: `*` matches within a segment); a call is reported when every argument is a literal"),
        ("consts", StrList, "[]", "named colour constants (`Type::*` takes every upper-case constant of the type)"),
        ("allowed", StrList, "[]", "constants that are no hard-coded colour (transparent)"),
        ("macros", StrList, "[]", "macros that build a colour from a literal"),
        ("hex-strings", Bool, "true", "report CSS colour strings (`\"#1e1e2e\"`)"),
        ("theme-crates", StrList, "[\"*_theme\", \"*-theme\"]", "crates that are the theme, besides those the crate map marks `theme = true`"),
        ("theme-paths", StrList, "[\"**/theme.rs\", \"**/theme/**\"]", "files that are the theme"),
    ],
    fix: None);
ui_lint!(SIZE_LITERAL, "U0004", "ui-size-literal", "ui", "UI code takes spacing, sizes, radii and type sizes from the theme, never writes them as numbers",
    params: [
        ("calls", StrList, "[]", "methods (a name) and functions (`Type::name`) whose number arguments are sizes"),
        ("fields", StrList, "[]", "style fields whose assigned numbers are sizes"),
        ("allowed", StrList, "[\"0\"]", "numbers that are no size choice"),
        ("theme-crates", StrList, "[\"*_theme\", \"*-theme\"]", "crates that are the theme, besides those the crate map marks `theme = true`"),
        ("theme-paths", StrList, "[\"**/theme.rs\", \"**/theme/**\"]", "files that are the theme"),
    ],
    fix: None);
ui_lint!(DISABLED_WITHOUT_REASON, "U0005", "ui-disabled-without-reason", "ui", "a disabled control says why and what would enable it",
    params: [
        ("disable-calls", StrList, "[]", "methods that disable a control or a region"),
        ("reason-calls", StrList, "[]", "methods that attach the reason a disabled control shows"),
    ],
    fix: None);
ui_lint!(DESTRUCTIVE_UNCONFIRMED, "U0006", "ui-destructive-unconfirmed", "ui", "a control that throws work away asks first or can be undone",
    params: [
        ("calls", StrList, "[]", "calls that make a button from a label"),
        ("verbs", StrList, "[\"delete\", \"remove\", \"erase\", \"wipe\", \"reset\", \"format\", \"discard\", \"purge\", \"destroy\", \"overwrite\", \"uninstall\", \"kill\", \"revoke\", \"forget\"]", "first words of a label that throws work away"),
        ("confirm-words", StrList, "[\"confirm\", \"undo\", \"modal\", \"dialog\"]", "words in a name of the same function that show a confirmation or an undo"),
    ],
    fix: None);
ui_lint!(MORE_COUNT, "U0007", "ui-more-count", "ui", "a cut list ends in a fold that opens to every item, never in \"N more\"",
    params: [
        ("words", StrList, "[\"more\", \"others\"]", "words that, after a number, say how many items are not shown"),
        ("fold-calls", StrList, "[]", "calls that make a fold, whose header may count what it holds"),
    ],
    fix: None);
ui_lint!(LABEL_PUNCTUATION, "U0008", "ui-label-punctuation", "ui", "UI text writes the ellipsis as one character and button labels end without a full stop",
    params: [
        ("calls", StrList, "[]", "calls that show text to a person"),
        ("button-calls", StrList, "[]", "calls that make a button from a label"),
    ],
    fix: Exact);
ui_lint!(DEBUG_TEXT, "U0009", "ui-debug-text", "ui", "text shown to a person never holds Rust's debug formatting",
    params: [("calls", StrList, "[]", "calls that show text to a person")],
    fix: None);
ui_lint!(UNWRAP_EXTERNAL_DATA, "U0014", "ui-unwrap-external-data", "ui", "UI code returns an error for a bad file, parse or connection, never unwraps or expects it",
    params: [
        ("calls", StrList, "[\"std::fs::read\", \"std::fs::read_to_string\", \"std::fs::write\", \"std::fs::File::open\", \"std::fs::File::create\", \"std::fs::metadata\", \"std::fs::read_dir\", \"serde_json::from_str\", \"serde_json::from_slice\", \"serde_json::from_reader\", \"toml::from_str\", \"toml_edit::de::from_str\", \"ron::from_str\", \"ron::de::from_str\", \"bincode::deserialize\", \"std::net::TcpStream::connect\", \"reqwest::blocking::get\", \"reqwest::get\", \"ureq::get\", \"ureq::post\"]", "constructors and calls whose result a file, a parse or a connection decides, resolved through the file's `use` items"),
        ("panic-methods", StrList, "[\"unwrap\", \"expect\"]", "methods that panic on an `Err` or a `None`"),
    ],
    fix: None);
ui_lint!(SHARED_THEME, "U0010", "egui-shared-theme", "egui", "every egui app installs the shared theme",
    params: [
        ("entry-calls", StrList, "[\"eframe::run_native\", \"eframe::run_simple_native\"]", "calls that start an app"),
        ("setup-calls", StrList, "[\"ve_theme::setup\", \"ve_theme::setup_with\"]", "calls that install the shared theme"),
    ],
    fix: None);
ui_lint!(STATUS_GLYPH, "U0011", "egui-status-glyph", "egui", "a status glyph is painted in its tone, never written into a label",
    params: [
        ("calls", StrList, "[\"label\", \"button\", \"small_button\", \"heading\", \"selectable_label\", \"checkbox\", \"on_hover_text\", \"Label::new\", \"Button::new\", \"RichText::new\"]", "calls that show text to a person"),
        ("glyphs", StrList, "[\"\\u2705\", \"\\U0001F6A7\", \"\\U0001F9EA\", \"\\U0001F41B\", \"\\u26D4\", \"\\u23F3\", \"\\u274C\", \"\\u26A0\", \"\\U0001F7E2\", \"\\U0001F7E1\", \"\\U0001F534\"]", "characters that carry a status"),
        ("replacement", Str, "\"ve_theme::status_icon\"", "what paints a status glyph, named in the help"),
    ],
    fix: None);
ui_lint!(ID_IN_LOOP, "U0012", "egui-id-in-loop", "egui", "a widget made in a loop gets an id that differs per iteration",
    params: [
        ("id-calls", StrList, "[\"Grid::new\", \"ComboBox::from_id_salt\", \"ComboBox::from_label\", \"CollapsingHeader::new\", \"Window::new\", \"id_salt\", \"id_source\", \"push_id\"]", "calls whose first argument becomes the widget's id"),
        ("unsalted", StrList, "[\"ScrollArea::vertical\", \"ScrollArea::horizontal\", \"ScrollArea::both\", \"ScrollArea::new\"]", "constructors whose widget takes a fixed id unless a salt method follows"),
        ("salt-methods", StrList, "[\"id_salt\", \"id_source\", \"id\"]", "methods that give a widget its id"),
        ("loop-methods", StrList, "[\"for_each\", \"try_for_each\"]", "iterator methods whose closure is a loop body"),
    ],
    fix: None);
ui_lint!(REPAINT_EVERY_FRAME, "U0013", "egui-repaint-every-frame", "egui", "UI code asks for a repaint when something changed, not on every frame",
    params: [
        ("calls", StrList, "[\"request_repaint\"]", "methods that ask for the next frame now"),
        ("after-calls", StrList, "[\"request_repaint_after\"]", "methods that ask for a frame after a delay"),
        ("ui-types", StrList, "[\"Ui\", \"Frame\"]", "parameter types of a function that runs every frame"),
        ("interval-ms-min", Int, "100", "a delay below which an unconditional repaint request is a busy loop"),
    ],
    fix: None);

/// The UI analysis
pub struct Ui;

/// The registered instance
pub static UI: Ui = Ui;

static LINTS: &[&LintMeta] = &[
    &RAW_SPAWN,
    &BLOCKING_CALL,
    &COLOUR_LITERAL,
    &SIZE_LITERAL,
    &DISABLED_WITHOUT_REASON,
    &DESTRUCTIVE_UNCONFIRMED,
    &MORE_COUNT,
    &LABEL_PUNCTUATION,
    &DEBUG_TEXT,
    &UNWRAP_EXTERNAL_DATA,
    &SHARED_THEME,
    &STATUS_GLYPH,
    &ID_IN_LOOP,
    &REPAINT_EVERY_FRAME,
];

/// The lints that skip the theme
const THEMED: &[&str] = &["ui-colour-literal", "ui-size-literal"];

impl sl_analysis::Analysis for Ui {
    fn name(&self) -> &'static str {
        "ui"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        let p = cx.project;
        for pkg in &p.packages {
            let crate_on: Vec<&'static str> = LINTS
                .iter()
                .map(|m| m.id)
                .filter(|id| cx.enabled(id) && is_ui_crate(cx, id, pkg))
                .filter(|id| !(THEMED.contains(id) && is_theme_crate(cx, id, pkg)))
                .collect();
            if crate_on.is_empty() {
                continue;
            }
            let mut entries = Vec::new();
            let mut has_setup = false;
            for f in p.sources(pkg) {
                if f.kind != FileKind::Src || p.is_test_file(pkg, f) {
                    continue;
                }
                let Ok(ast) = f.ast() else { continue };
                let on: Vec<&'static str> = crate_on
                    .iter()
                    .copied()
                    .filter(|id| {
                        !(THEMED.contains(id)
                            && cx
                                .params(id)
                                .list("theme-paths")
                                .iter()
                                .any(|g| glob::matches(g, &f.rel)))
                    })
                    .collect();
                let ast = sl_analysis::syntax::without_tests(ast);
                let out = walk::run(cx, pkg, &on, &f.text, &ast);
                has_setup |= out.has_setup;
                entries.extend(out.entries.into_iter().map(|r| (f.rel.clone(), r)));
                for d in out.found {
                    let line = d.range.0.0;
                    let mut diag = Diagnostic::new(
                        d.lint,
                        format!("{}:{}:{line}", pkg.name, f.rel),
                        d.message,
                    )
                    .span(Span::range(f.rel.clone(), d.range.0, d.range.1))
                    .krate(&pkg.name)
                    .confidence(d.confidence)
                    .help(d.help);
                    if let Some(fix) = d.fix {
                        diag = diag.fix(sl_core::Fix {
                            description: fix.0,
                            edits: fix
                                .1
                                .into_iter()
                                .map(|(start, end, replacement)| sl_core::Edit {
                                    file: f.rel.clone(),
                                    start,
                                    end,
                                    replacement,
                                })
                                .collect(),
                        });
                    }
                    cx.emit(diag);
                }
            }
            if crate_on.contains(&"egui-shared-theme") && !has_setup {
                let setup = cx
                    .params("egui-shared-theme")
                    .list("setup-calls")
                    .join("`, `");
                for (file, r) in entries {
                    let line = r.0.0;
                    cx.emit(
                        Diagnostic::new("egui-shared-theme", format!("{}:{file}:{line}", pkg.name), format!("{} starts an app but never installs the shared theme", pkg.name))
                            .span(Span::range(file, r.0, r.1))
                            .krate(&pkg.name)
                            .confidence(Confidence::Estimate)
                            .help(format!("call `{setup}` with the egui context before the first frame, so the app takes the shared colours, spacing and type")),
                    );
                }
            }
        }
    }
}

/// Whether a lint holds `pkg` as a UI crate: the crate map's `ui` key when it has one, else a direct dependency on
/// one of the lint's `toolkits`
pub fn is_ui_crate(cx: &Context<'_>, lint: &str, pkg: &Package) -> bool {
    if let Some(ui) = cx
        .map()
        .and_then(|m| m.crates.get(&pkg.name))
        .and_then(|c| c.ui)
    {
        return ui;
    }
    let toolkits = cx.params(lint).list("toolkits");
    pkg.deps
        .iter()
        .any(|d| d.kind == DepKind::Normal && toolkits.contains(&d.name))
}

/// Whether `pkg` is the theme for a lint: marked `theme = true` in the crate map, or named like its `theme-crates`
fn is_theme_crate(cx: &Context<'_>, lint: &str, pkg: &Package) -> bool {
    cx.map()
        .and_then(|m| m.crates.get(&pkg.name))
        .is_some_and(|c| c.theme)
        || cx
            .params(lint)
            .list("theme-crates")
            .iter()
            .any(|g| glob::matches(g, &pkg.name))
}
