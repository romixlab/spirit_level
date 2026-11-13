//! Files committed by mistake: leftovers of merges and editors, agent configuration with secrets or broad
//! permissions, session transcripts and per-person agent files. Read from git's list of tracked files.

use sl_analysis::Context;
use sl_core::{Confidence, Diagnostic, Span, glob};
use std::process::Command;

const LINTS: &[&str] = &[
    "leftover-files",
    "agent-config-secrets",
    "agent-config-permissions",
    "agent-session-files",
];

/// The tracked files under the project root, relative to it
fn tracked(cx: &Context<'_>) -> Result<Vec<String>, String> {
    let out = Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(&cx.project.root)
        .output()
        .map_err(|e| format!("cannot run git: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect())
}

pub(crate) fn run(cx: &Context<'_>) {
    if !LINTS.iter().any(|l| cx.enabled(l)) {
        return;
    }
    let files = match tracked(cx) {
        Ok(f) => f,
        Err(e) => {
            for id in LINTS {
                cx.emit(
                    Diagnostic::new(id, "git", format!("tracked files not checked: {e}"))
                        .span(Span::new(".git", 0))
                        .confidence(Confidence::Candidate)
                        .help("run spl inside the project's git checkout"),
                );
            }
            return;
        }
    };
    matching(cx, &files, "leftover-files", |f| {
        (
            format!("{f} is a leftover (merge, patch, editor or scratch file)"),
            "delete it and add the pattern to .gitignore".to_string(),
        )
    });
    matching(cx, &files, "agent-session-files", |f| {
        (format!("{f} is an agent session file (transcript, history or per-person settings)"), "remove it from git (`git rm --cached`) and ignore it; transcripts hold private context and secrets".to_string())
    });
    secrets(cx, &files);
    permissions(cx, &files);
}

fn matching(cx: &Context<'_>, files: &[String], id: &str, say: impl Fn(&str) -> (String, String)) {
    if !cx.enabled(id) {
        return;
    }
    let pats = cx.params(id).list("patterns");
    for f in files
        .iter()
        .filter(|f| pats.iter().any(|p| glob::matches(p, f)))
    {
        let (msg, help) = say(f);
        cx.emit(
            Diagnostic::new(id, format!("file:{f}"), msg)
                .span(Span::new(f.clone(), 0))
                .help(help),
        );
    }
}

/// A value that looks like a credential: a known token prefix, then at least 16 characters with letters and
/// digits
fn token_like(v: &str, prefixes: &[String]) -> bool {
    prefixes.iter().any(|p| {
        v.strip_prefix(p.as_str()).is_some_and(|t| {
            t.len() >= 16
                && t.chars().any(|c| c.is_ascii_digit())
                && t.chars().any(|c| c.is_ascii_alphabetic())
        })
    })
}

/// A placeholder rather than a value: empty, a variable reference, an angle-bracket or all-x stand-in
fn placeholder(v: &str) -> bool {
    let v = v.trim();
    v.is_empty()
        || v.starts_with('$')
        || v.starts_with("${")
        || (v.starts_with('<') && v.ends_with('>'))
        || v.chars().all(|c| matches!(c, 'x' | 'X' | '*' | '.' | '-'))
}

/// `(key, value)` pairs of a config line: every `"key": "value"` or `key = "value"`, and `KEY=value` (an env
/// file)
fn pairs(line: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut rest = line;
    while let Some(i) = rest.find([':', '=']) {
        let key = rest[..i]
            .rsplit(|c: char| c.is_whitespace() || matches!(c, '{' | ',' | '['))
            .find(|k| !k.is_empty())
            .unwrap_or("")
            .trim_matches(['"', '\''])
            .to_string();
        let after = rest[i + 1..].trim_start();
        let (val, next) = if let Some(nested) = after.strip_prefix(['{', '[']) {
            (String::new(), nested)
        } else if let Some(q) = after.strip_prefix(['"', '\'']) {
            let close = q.find(['"', '\'']).unwrap_or(q.len());
            (q[..close].to_string(), &q[(close + 1).min(q.len())..])
        } else {
            let end = after.find([',', '}', ' ']).unwrap_or(after.len());
            (after[..end].to_string(), &after[end..])
        };
        if !key.is_empty() && !val.is_empty() {
            out.push((key, val));
        }
        rest = next;
    }
    out
}

fn secrets(cx: &Context<'_>, files: &[String]) {
    let id = "agent-config-secrets";
    if !cx.enabled(id) {
        return;
    }
    let p = cx.params(id);
    let (pats, keys, prefixes) = (p.list("files"), p.list("keys"), p.list("prefixes"));
    for f in files
        .iter()
        .filter(|f| pats.iter().any(|g| glob::matches(g, f)))
    {
        let Some(text) = cx.project.read(f) else {
            continue;
        };
        for (n, line) in text.lines().enumerate() {
            let words =
                line.split(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.')));
            let by_prefix = words
                .into_iter()
                .find(|w| token_like(w, prefixes))
                .map(|w| format!("a value starting `{}`", &w[..w.len().min(6)]));
            let by_key = pairs(line).into_iter().find_map(|(k, v)| {
                let kl = k.to_ascii_lowercase();
                keys.iter()
                    .any(|s| kl.contains(s.as_str()))
                    .then_some(())
                    .filter(|_| {
                        !placeholder(&v) && !v.split_whitespace().last().is_some_and(placeholder)
                    })
                    .map(|_| format!("`{k}` with a literal value"))
            });
            if let Some(why) = by_prefix.or(by_key) {
                cx.emit(
                    Diagnostic::new(id, format!("{f}:{n}"), format!("{f} holds a secret: {why}"))
                        .span(Span::new(f.clone(), n as u32 + 1))
                        .help("revoke it, remove it from the file and the history, and read it from the environment (`$VAR`)"),
                );
            }
        }
    }
}

fn permissions(cx: &Context<'_>, files: &[String]) {
    let id = "agent-config-permissions";
    if !cx.enabled(id) {
        return;
    }
    let p = cx.params(id);
    let (pats, broad) = (p.list("files"), p.list("broad"));
    for f in files
        .iter()
        .filter(|f| pats.iter().any(|g| glob::matches(g, f)))
    {
        let Some(text) = cx.project.read(f) else {
            continue;
        };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
            continue;
        };
        let line_of = |needle: &str| {
            text.lines()
                .position(|l| l.contains(needle))
                .map_or(0, |i| i as u32 + 1)
        };
        let mut found: Vec<(String, u32)> = Vec::new();
        if let Some(allow) = json
            .pointer("/permissions/allow")
            .and_then(|a| a.as_array())
        {
            for e in allow.iter().filter_map(|e| e.as_str()) {
                if broad.iter().any(|b| b == e) {
                    found.push((format!("allows `{e}`"), line_of(&format!("\"{e}\""))));
                }
            }
        }
        if json
            .pointer("/permissions/defaultMode")
            .and_then(|m| m.as_str())
            == Some("bypassPermissions")
        {
            found.push((
                "sets defaultMode = bypassPermissions".into(),
                line_of("defaultMode"),
            ));
        }
        for k in [
            "dangerouslySkipPermissions",
            "skipDangerousModePermissionPrompt",
        ] {
            if json.get(k).and_then(|v| v.as_bool()) == Some(true) {
                found.push((format!("sets {k}"), line_of(k)));
            }
        }
        for (what, line) in found {
            cx.emit(
                Diagnostic::new(id, format!("{f}:{what}"), format!("{f} {what}: every agent in this repo may run anything"))
                    .span(Span::new(f.clone(), line))
                    .help("allow the commands the project needs by name (`Bash(cargo test:*)`); keep wider grants in personal settings"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{pairs, placeholder};

    #[test]
    fn config_pairs() {
        assert_eq!(
            pairs("  \"API_KEY\": \"abc\","),
            vec![("API_KEY".into(), "abc".into())]
        );
        assert_eq!(
            pairs("\"env\": { \"A\": \"1\", \"DB_PASSWORD\": \"h:x\" }"),
            vec![
                ("A".into(), "1".into()),
                ("DB_PASSWORD".into(), "h:x".into())
            ]
        );
        assert_eq!(pairs("TOKEN=abc"), vec![("TOKEN".into(), "abc".into())]);
        assert_eq!(pairs("token = \"x\""), vec![("token".into(), "x".into())]);
        assert!(placeholder("${GITHUB_TOKEN}"));
        assert!(placeholder("<your key>"));
        assert!(!placeholder("s3cr3t"));
    }
}
