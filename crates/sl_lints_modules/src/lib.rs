//! Module-graph lints. One analysis, [`MODULES`], builds each crate's module graph from its paths and `use` items
//! and reports `module-graph-scc`.

mod graph;
mod scc;

use sl_analysis::crate_tree::{CrateKind, crates};
use sl_analysis::{Analysis, Context, lint};
use sl_core::{Confidence, Diagnostic, LintMeta, Span};
use std::collections::BTreeMap;

lint!(MODULE_GRAPH_SCC, "module-graph-scc", "structure", "no module cycle holds most of a crate's code",
    params: [
        ("max-share", Float, "0.5", "share of a crate's code lines its biggest module cycle may hold"),
        ("min-lines", Int, "3000", "crates with fewer code lines are not checked"),
        ("depth", Int, "1", "module depth of the graph's nodes: 1 groups every module under its top-level module"),
        ("max-listed", Int, "12", "modules and cut edges the message names"),
    ],
    needs: [], fix: None);

/// The module-graph analysis
pub struct Modules;

/// The registered instance
pub static MODULES: Modules = Modules;

static LINTS: &[&LintMeta] = &[&MODULE_GRAPH_SCC];

impl Analysis for Modules {
    fn name(&self) -> &'static str {
        "modules"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        let p = cx.project;
        for pkg in &p.packages {
            if cx
                .map()
                .and_then(|m| m.crates.get(&pkg.name))
                .is_some_and(|c| c.generated)
            {
                continue;
            }
            let params = cx.params_for("module-graph-scc", pkg);
            let max_share = params.float("max-share");
            let min_lines = params.int("min-lines");
            let depth = usize::try_from(params.int("depth")).unwrap_or(1);
            let listed = usize::try_from(params.int("max-listed")).unwrap_or(12);
            for tree in crates(p, pkg) {
                let total = tree.code();
                if i64::from(total) < min_lines || total == 0 {
                    continue;
                }
                let g = graph::build(&tree, depth);
                let pairs: Vec<(usize, usize)> = g.edges.keys().copied().collect();
                let Some(big) = scc::components(g.nodes.len(), &pairs)
                    .into_iter()
                    .filter(|c| c.len() > 1)
                    .max_by_key(|c| c.iter().map(|n| g.nodes[*n].code).sum::<u32>())
                else {
                    continue;
                };
                let code: u32 = big.iter().map(|n| g.nodes[*n].code).sum();
                let share = f64::from(code) / f64::from(total);
                if share <= max_share {
                    continue;
                }
                let weights: BTreeMap<(usize, usize), u32> =
                    g.edges.iter().map(|(k, e)| (*k, e.refs)).collect();
                let mut cut = scc::cut(&big, &weights);
                cut.sort_by_key(|e| {
                    (
                        weights[e],
                        g.nodes[e.0].name.clone(),
                        g.nodes[e.1].name.clone(),
                    )
                });
                let cut_refs: u32 = cut.iter().map(|e| weights[e]).sum();
                let mut members: Vec<usize> = big.clone();
                members.sort_by_key(|n| {
                    (
                        std::cmp::Reverse(g.nodes[*n].code),
                        g.nodes[*n].name.clone(),
                    )
                });
                let mut names: Vec<String> = members
                    .iter()
                    .take(listed)
                    .map(|n| g.nodes[*n].name.clone())
                    .collect();
                if members.len() > listed {
                    names.push(format!("{} more", members.len() - listed));
                }
                let edges: Vec<String> = cut
                    .iter()
                    .take(listed)
                    .map(|e| {
                        format!(
                            "{} -> {} ({})",
                            g.nodes[e.0].name, g.nodes[e.1].name, weights[e]
                        )
                    })
                    .collect();
                let more = if cut.len() > listed {
                    format!(", {} more", cut.len() - listed)
                } else {
                    String::new()
                };
                let pct = (share * 100.0).round();
                let message = format!(
                    "{}: one module cycle holds {pct}% of its code lines ({code} of {total}), {} modules: {}; cutting {cut_refs} references over {} module pairs breaks it, the fewest first: {}{more} (from paths without macro expansion; the cut is a heuristic)",
                    tree.name,
                    big.len(),
                    names.join(", "),
                    cut.len(),
                    edges.join(", "),
                );
                let root = &tree.modules[0].file.rel;
                let mut d = Diagnostic::new(
                    "module-graph-scc",
                    match tree.kind {
                        CrateKind::Lib => format!("crate:{}", tree.name),
                        CrateKind::Bin => format!("bin:{}", tree.name),
                    },
                    message,
                )
                .span(Span::new(root.clone(), 0))
                .krate(&pkg.name)
                .confidence(Confidence::Estimate)
                .metric("scc-share", share, Some(max_share))
                .help("cut the listed references (move the item down, pass a function or a trait object in, or merge the two modules), until the cycle falls apart into modules that can become crates");
                for e in cut.iter().take(listed) {
                    let (file, line) = &g.edges[e].first;
                    d = d.span(Span::new(file.clone(), *line).label(format!(
                        "{} -> {}: {} references",
                        g.nodes[e.0].name, g.nodes[e.1].name, weights[e]
                    )));
                }
                cx.emit(d);
            }
        }
    }
}
