//! Near-duplicate functions and modules: what a rewrite leaves behind when the old version is not deleted.
//!
//! A function's text (signature and body) is tokenized; local names become `I` and literals `L` (keywords,
//! punctuation and called or path names stay), so renamed copies compare equal. The similarity of two functions is the Jaccard index of
//! their sets of 4-token shingles.

use proc_macro2::{TokenStream, TokenTree};
use sl_analysis::Context;
use sl_analysis::syntax::{functions, without_tests};
use sl_core::{Diagnostic, Span};
use std::collections::{BTreeMap, HashSet};

/// Rust keywords, kept as themselves when normalizing
const KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while", "Some", "None", "Ok", "Err",
];

/// The normalized tokens of a piece of Rust: local names (bindings, parameters, the function's own name) become
/// `I`, literals `L`; keywords, punctuation and the names a function calls or reaches through a path (`x(`,
/// `.x`, `::x`) stay, so a renamed copy matches and two functions that call different things do not
pub fn tokens(text: &str) -> Vec<String> {
    fn walk(ts: TokenStream, out: &mut Vec<String>) {
        let trees: Vec<TokenTree> = ts.into_iter().collect();
        for (k, t) in trees.iter().enumerate() {
            match t {
                TokenTree::Ident(i) => {
                    let s = i.to_string();
                    let called = matches!(trees.get(k + 1), Some(TokenTree::Group(g)) if g.delimiter() == proc_macro2::Delimiter::Parenthesis)
                        && !matches!(out.last().map(String::as_str), Some("fn"));
                    let reached = matches!(out.last().map(String::as_str), Some(".") | Some(":"));
                    out.push(if KEYWORDS.contains(&s.as_str()) || called || reached {
                        s
                    } else {
                        "I".into()
                    });
                }
                TokenTree::Literal(_) => out.push("L".into()),
                TokenTree::Punct(p) => out.push(p.as_char().to_string()),
                TokenTree::Group(g) => {
                    let (open, close) = match g.delimiter() {
                        proc_macro2::Delimiter::Parenthesis => ("(", ")"),
                        proc_macro2::Delimiter::Brace => ("{", "}"),
                        proc_macro2::Delimiter::Bracket => ("[", "]"),
                        proc_macro2::Delimiter::None => ("", ""),
                    };
                    out.push(open.into());
                    walk(g.stream(), out);
                    out.push(close.into());
                }
            }
        }
    }
    let mut out = Vec::new();
    if let Ok(ts) = text.parse::<TokenStream>() {
        walk(ts, &mut out);
    }
    out.retain(|t| !t.is_empty());
    out
}

/// The set of `k`-token shingles, hashed
fn shingles(toks: &[String], k: usize) -> HashSet<u64> {
    use std::hash::{Hash, Hasher};
    toks.windows(k.min(toks.len()).max(1))
        .map(|w| {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            w.hash(&mut h);
            h.finish()
        })
        .collect()
}

/// The Jaccard index of two shingle sets
pub fn similarity(a: &HashSet<u64>, b: &HashSet<u64>) -> f64 {
    let inter = a.intersection(b).count();
    let union = a.len() + b.len() - inter;
    if union == 0 {
        0.0
    } else {
        inter as f64 / union as f64
    }
}

struct Func {
    krate: String,
    file: String,
    path: String,
    line: u32,
    len: usize,
    set: HashSet<u64>,
}

pub(crate) fn run(cx: &Context<'_>) {
    if !cx.enabled("near-duplicate-fns") && !cx.enabled("near-duplicate-modules") {
        return;
    }
    let pf = cx.params("near-duplicate-fns");
    let (min_sim, min_tokens) = (pf.float("min-similarity"), pf.int("min-tokens") as usize);
    let mut fns: Vec<Func> = Vec::new();
    let p = cx.project;
    for pkg in &p.packages {
        for src in p.sources(pkg) {
            if src.kind.is_test_like() || p.is_test_file(pkg, src) {
                continue;
            }
            let Ok(ast) = src.ast() else { continue };
            let ast = without_tests(ast);
            let lines: Vec<&str> = src.text.lines().collect();
            for f in functions(&ast, &src.module) {
                let (a, b) = (f.lines.0 as usize, f.lines.1 as usize);
                let text = lines
                    .get(a.saturating_sub(1)..b.min(lines.len()))
                    .unwrap_or(&[])
                    .join("\n");
                let toks = tokens(&text);
                if toks.len() < min_tokens {
                    continue;
                }
                fns.push(Func {
                    krate: pkg.name.clone(),
                    file: src.rel.clone(),
                    path: format!("{}::{}", pkg.name, f.path),
                    line: f.lines.0,
                    len: toks.len(),
                    set: shingles(&toks, 4),
                });
            }
        }
    }
    fns.sort_by_key(|f| f.len);
    let mut pairs: Vec<(usize, usize, f64)> = Vec::new();
    for i in 0..fns.len() {
        for j in i + 1..fns.len() {
            // the Jaccard index is at most the ratio of the sizes; sorted by length, later ones only grow
            if (fns[i].len as f64) < min_sim * fns[j].len as f64 {
                break;
            }
            let s = similarity(&fns[i].set, &fns[j].set);
            if s >= min_sim {
                pairs.push((i, j, s));
            }
        }
    }
    for &(i, j, s) in &pairs {
        let (a, b) = if (&fns[i].file, fns[i].line) <= (&fns[j].file, fns[j].line) {
            (&fns[i], &fns[j])
        } else {
            (&fns[j], &fns[i])
        };
        cx.emit(
            Diagnostic::new(
                "near-duplicate-fns",
                format!("{}~{}", a.path, b.path),
                format!("`{}` and `{}` are {:.0}% alike", b.path, a.path, s * 100.0),
            )
            .span(Span::new(b.file.clone(), b.line))
            .span(Span::new(a.file.clone(), a.line).label("the other copy"))
            .krate(&b.krate)
            .metric("similarity", s, Some(min_sim))
            .help("keep one: delete the old version, or make both call a shared function"),
        );
    }
    modules(cx, &fns, &pairs);
}

/// Files most of whose functions have a near-duplicate in one other file
fn modules(cx: &Context<'_>, fns: &[Func], pairs: &[(usize, usize, f64)]) {
    let id = "near-duplicate-modules";
    if !cx.enabled(id) {
        return;
    }
    let pm = cx.params(id);
    let (min_share, min_fns) = (pm.float("min-share"), pm.int("min-fns") as usize);
    let mut per_file: BTreeMap<&str, usize> = BTreeMap::new();
    for f in fns {
        *per_file.entry(&f.file).or_default() += 1;
    }
    // for each ordered pair of files, the functions of the first with a copy in the second
    let mut matched: BTreeMap<(&str, &str), HashSet<usize>> = BTreeMap::new();
    for &(i, j, _) in pairs {
        let (fi, fj) = (fns[i].file.as_str(), fns[j].file.as_str());
        if fi == fj {
            continue;
        }
        matched.entry((fi, fj)).or_default().insert(i);
        matched.entry((fj, fi)).or_default().insert(j);
    }
    for ((a, b), set) in &matched {
        if a > b {
            continue;
        }
        let back = matched.get(&(*b, *a)).map_or(0, HashSet::len);
        let (na, nb) = (per_file[a], per_file[b]);
        let share = (set.len() + back) as f64 / (na + nb) as f64;
        if set.len().min(back) >= min_fns && share >= min_share {
            let krate = fns
                .iter()
                .find(|f| f.file == *b)
                .map(|f| f.krate.clone())
                .unwrap_or_default();
            cx.emit(
                Diagnostic::new(
                    id,
                    format!("{a}~{b}"),
                    format!(
                        "{b} and {a} are near copies: {:.0}% of their functions match",
                        share * 100.0
                    ),
                )
                .span(Span::new(*b, 0))
                .span(Span::new(*a, 0).label("the other copy"))
                .krate(krate)
                .metric("share", share, Some(min_share))
                .help(
                    "finish the rewrite: delete the old module, or merge what both need into one",
                ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renamed_copies_match() {
        let a = tokens("fn load(path: &str) -> u32 { let x = read(path); x + 1 }");
        let b = tokens("fn load_v2(p: &str) -> u32 { let y = read(p); y + 2 }");
        assert_eq!(a, b);
        let c = tokens("fn other(n: u8) -> bool { n > 3 && n < 9 }");
        assert!(similarity(&shingles(&a, 4), &shingles(&c, 4)) < 0.3);
        let d = tokens(
            "fn run(cx: &C) { if cx.enabled(\"a\") { one(cx); } if cx.enabled(\"b\") { two(cx); } }",
        );
        let e = tokens(
            "fn run(cx: &C) { if cx.enabled(\"c\") { three(cx); } if cx.enabled(\"d\") { four(cx); } }",
        );
        assert_ne!(d, e);
    }
}
