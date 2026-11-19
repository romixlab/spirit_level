//! Size and complexity lints, one analysis: [`SIZE`].

use sl_analysis::syntax::{complexity, functions, without_tests};
use sl_analysis::{Analysis, Context, lint};
use sl_core::{Diagnostic, Level, LintMeta, Span};
use sl_model::{FileKind, LineCount, count_lines};
use std::collections::BTreeMap;

lint!(CRATE_LINES, "S0001", "crate-lines", "size", "a crate's src/ stays under its line limit",
    params: [("max-lines", Int, "20000", "code lines in src/ above which a crate is reported")],
    needs: [], fix: None);
lint!(TEST_LINES, "S0005", "test-lines", "size", "a crate's test code stays under its line limit",
    params: [("max-lines", Int, "20000", "code lines in tests/, benches/ and examples/ above which a crate is reported")],
    needs: [], fix: None);
lint!(MODULE_LINES, "S0004", "module-lines", "size", "a source file stays under its line limit",
    params: [("max-lines", Int, "1000", "code lines of one file above which it is reported")],
    needs: [], fix: None);
lint!(FUNCTION_LINES, "S0003", "function-lines", "size", "a function stays under its line limit",
    params: [("max-lines", Int, "150", "code lines of one function above which it is reported"),
             ("target-lines", Int, "0", "a lower limit to tighten toward: a function above it but within max-lines is reported as a warning at most; 0: none")],
    needs: [], fix: None);
lint!(CRATE_FOLDER_SIZE, "S0006", "crate-folder-size", "size", "a folder holds no more crate folders than its limit",
    params: [("max-flat", Int, "80", "crate folders side by side in a folder that is not a layer folder (a flat crates/) above which it is reported"),
             ("max-crates", Int, "100", "crate folders in one layer or discipline folder above which it is reported")],
    needs: [], fix: None);
lint!(FUNCTION_COMPLEXITY, "S0002", "function-complexity", "size", "a function's cyclomatic complexity stays under its limit",
    params: [("max", Int, "15", "the highest complexity allowed")],
    needs: [], fix: None);

/// The size analysis
pub struct Size;

/// The registered instance
pub static SIZE: Size = Size;

static LINTS: &[&LintMeta] = &[
    &CRATE_LINES,
    &TEST_LINES,
    &MODULE_LINES,
    &FUNCTION_LINES,
    &FUNCTION_COMPLEXITY,
    &CRATE_FOLDER_SIZE,
];

impl Analysis for Size {
    fn name(&self) -> &'static str {
        "size"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        let p = cx.project;
        folder_size(cx);
        for pkg in &p.packages {
            if cx
                .map()
                .and_then(|m| m.crates.get(&pkg.name))
                .is_some_and(|c| c.generated)
            {
                continue;
            }
            let files = p.sources(pkg);
            let (mut src, mut test) = (LineCount::default(), LineCount::default());
            for f in files {
                match f.kind {
                    FileKind::Build => {}
                    _ if p.is_test_file(pkg, f) => test += f.lines,
                    _ => src += f.lines,
                }
            }
            let root = files
                .iter()
                .find(|f| f.is_crate_root())
                .map_or_else(|| pkg.manifest.rel.clone(), |f| f.rel.clone());
            for (id, count, what) in [
                ("crate-lines", src.code, "src/"),
                ("test-lines", test.code, "tests, benches and examples"),
            ] {
                let max = cx.params_for(id, pkg).int("max-lines");
                if i64::from(count) > max {
                    cx.emit(
                        Diagnostic::new(id, format!("crate:{}", pkg.name), format!("{} has {count} code lines in {what} (limit {max})", pkg.name))
                            .span(Span::new(root.clone(), 0))
                            .krate(&pkg.name)
                            .metric("code-lines", f64::from(count), Some(max as f64))
                            .help(if id == "crate-lines" {
                                "split it along its seams (types, a heavy dependency, parts that change at their own pace); when it must stay big for a while, add an exception with a reason and an expiry"
                            } else {
                                "move tests into a crate of their own, or split the crate they test"
                            }),
                    );
                }
            }
            let module_max = cx.params_for("module-lines", pkg).int("max-lines");
            let fn_max = cx.params_for("function-lines", pkg).int("max-lines");
            let fn_target = cx.params_for("function-lines", pkg).int("target-lines");
            let cx_max = cx.params_for("function-complexity", pkg).int("max");
            for f in files
                .iter()
                .filter(|f| f.kind == FileKind::Src && !p.is_test_file(pkg, f))
            {
                if i64::from(f.lines.code) > module_max {
                    cx.emit(
                        Diagnostic::new("module-lines", format!("file:{}", f.rel), format!("{} has {} code lines (limit {module_max})", f.rel, f.lines.code))
                            .span(Span::new(f.rel.clone(), 0))
                            .krate(&pkg.name)
                            .metric("code-lines", f64::from(f.lines.code), Some(module_max as f64))
                            .help("split the module by what its parts are about; a type and its impls can move to a file of their own"),
                    );
                }
                if !(cx.enabled("function-lines") || cx.enabled("function-complexity")) {
                    continue;
                }
                let Ok(ast) = f.ast() else { continue };
                let ast = without_tests(ast);
                let lines: Vec<&str> = f.text.lines().collect();
                for func in functions(&ast, &f.module) {
                    let (a, b) = func.lines;
                    let body = lines
                        .get(a.saturating_sub(1) as usize..(b as usize).min(lines.len()))
                        .map(|l| l.join("\n"))
                        .unwrap_or_default();
                    let code = count_lines(&body).code;
                    let item = format!("{}::{}", pkg.name, func.path);
                    if i64::from(code) > fn_max {
                        cx.emit(
                            Diagnostic::new("function-lines", &item, format!("{} has {code} code lines (limit {fn_max})", func.path))
                                .span(Span::new(f.rel.clone(), a).to(b))
                                .krate(&pkg.name)
                                .metric("code-lines", f64::from(code), Some(fn_max as f64))
                                .help("split it into steps with names; a long match often wants a function per arm"),
                        );
                    } else if fn_target > 0 && i64::from(code) > fn_target {
                        cx.emit(
                            Diagnostic::new("function-lines", &item, format!("{} has {code} code lines, above the target {fn_target} (limit {fn_max})", func.path))
                                .span(Span::new(f.rel.clone(), a).to(b))
                                .krate(&pkg.name)
                                .metric("code-lines", f64::from(code), Some(fn_target as f64))
                                .at_most(Level::Warn)
                                .help("the limit is moving down to the target: split it into steps with names while it is in hand"),
                        );
                    }
                    let c = complexity(func.block);
                    if i64::from(c) > cx_max {
                        cx.emit(
                            Diagnostic::new("function-complexity", &item, format!("{} has cyclomatic complexity {c} (limit {cx_max})", func.path))
                                .span(Span::new(f.rel.clone(), a).to(b))
                                .krate(&pkg.name)
                                .metric("complexity", f64::from(c), Some(cx_max as f64))
                                .help("pull branches out into functions or a table; early returns flatten nesting"),
                        );
                    }
                }
            }
        }
    }
}

/// `crate-folder-size`: counts the crate folders in each folder that holds some
fn folder_size(cx: &Context<'_>) {
    let p = cx.project;
    let mut folders: BTreeMap<&str, u32> = BTreeMap::new();
    for pkg in p
        .packages
        .iter()
        .filter(|pkg| !pkg.dir.is_empty() && !cx.excluded(&pkg.name))
    {
        let parent = pkg.dir.rsplit_once('/').map_or("", |(parent, _)| parent);
        *folders.entry(parent).or_default() += 1;
    }
    let is_layer = |name: &str| cx.map().is_some_and(|m| m.layer(name).is_some());
    let params = cx.params("crate-folder-size");
    let root = &p.root_manifest;
    let span = Span::new(root.rel.clone(), root.line(&["workspace", "members"]));
    for (folder, count) in folders {
        let mut names = folder.rsplit('/');
        let name = names.next().unwrap_or_default();
        let layered = is_layer(name) || names.next().is_some_and(is_layer);
        let (max, help) = if layered {
            (
                params.int("max-crates"),
                "split the layer folder by discipline, crates/<layer>/<discipline>/<crate>, with the map key `discipline`",
            )
        } else {
            (
                params.int("max-flat"),
                "group the crates by layer, crates/<layer>/<crate>, as the crate map's layers say; a layer of about 100 crates then splits by discipline",
            )
        };
        if i64::from(count) > max {
            let shown = if folder.is_empty() {
                "the workspace root"
            } else {
                folder
            };
            cx.emit(
                Diagnostic::new(
                    "crate-folder-size",
                    format!("folder:{folder}"),
                    format!("{shown} holds {count} crate folders (limit {max})"),
                )
                .span(span.clone())
                .metric("crate-folders", f64::from(count), Some(max as f64))
                .help(help),
            );
        }
    }
}
