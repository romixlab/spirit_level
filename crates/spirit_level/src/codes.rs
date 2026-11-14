//! The registry of lint codes, `codes.toml`.
//!
//! Every code ever given, with the lint id it belongs to, its group and whether the lint is active or retired. A code
//! is never reused or changed; a retired lint keeps its row, so its code stays reserved. [`check`] holds the registry
//! to the lints and to an earlier copy of itself.

use sl_core::LintMeta;
use std::collections::BTreeMap;

/// The registry as shipped
pub const REGISTRY: &str = include_str!("../codes.toml");

/// Whether a code's lint still exists
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// The lint exists
    Active,
    /// The lint is gone; its code stays reserved
    Retired,
}

impl Status {
    /// `active`, `retired`
    pub fn name(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Retired => "retired",
        }
    }
}

/// One row of the registry
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    /// The code (`S0001`)
    pub code: String,
    /// The lint id (`crate-lines`)
    pub id: String,
    /// The group's name (`size`)
    pub group: String,
    /// Active or retired
    pub status: Status,
}

/// Reads a registry; every row it can read, and a problem for each it cannot
pub fn parse(text: &str) -> (Vec<Row>, Vec<String>) {
    let mut rows = Vec::new();
    let mut problems = Vec::new();
    let doc = match text.parse::<toml_edit::DocumentMut>() {
        Ok(d) => d,
        Err(e) => return (rows, vec![format!("codes.toml does not parse: {e}")]),
    };
    for (code, item) in doc.iter() {
        let field = |k: &str| item.get(k).and_then(|v| v.as_str()).map(str::to_owned);
        let status = match field("status").as_deref() {
            Some("active") => Status::Active,
            Some("retired") => Status::Retired,
            s => {
                problems.push(format!(
                    "{code}: status is {s:?}, not \"active\" or \"retired\""
                ));
                continue;
            }
        };
        match (field("id"), field("group")) {
            (Some(id), Some(group)) => rows.push(Row {
                code: code.to_owned(),
                id,
                group,
                status,
            }),
            _ => problems.push(format!("{code}: needs id and group")),
        }
    }
    (rows, problems)
}

/// Every row of the shipped registry
pub fn rows() -> Vec<Row> {
    parse(REGISTRY).0
}

/// The problems of `registry` against the lints, and against `base` when given.
///
/// `base` is the registry of an earlier commit. The problems: a lint without a code or with another code than its
/// row, a code or an id twice, a code that does not fit its group, a retired row whose lint still exists, an active
/// row without a lint, and a row of `base` that changed or went (only `active` to `retired` may change).
pub fn check(lints: &[&LintMeta], registry: &str, base: Option<&str>) -> Vec<String> {
    let (rows, mut problems) = parse(registry);
    let mut by_code: BTreeMap<&str, &Row> = BTreeMap::new();
    let mut by_id: BTreeMap<&str, &Row> = BTreeMap::new();
    for r in &rows {
        match sl_core::valid_code(&r.code) {
            None => problems.push(format!(
                "{}: not a code (a group letter and four digits)",
                r.code
            )),
            Some(g) if g != r.group => problems.push(format!(
                "{}: the letter is group {g}, the row says {}",
                r.code, r.group
            )),
            Some(_) => {}
        }
        if by_code.insert(&r.code, r).is_some() {
            problems.push(format!("{}: given twice", r.code));
        }
        if let Some(other) = by_id.insert(&r.id, r) {
            problems.push(format!(
                "{} has two codes: {} and {}",
                r.id, other.code, r.code
            ));
        }
    }
    for m in lints {
        match by_id.get(m.id) {
            None => problems.push(format!(
                "{} ({}) has no row in codes.toml: add `{} = {{ id = \"{}\", group = \"{}\", status = \"active\" }}`",
                m.id,
                m.code,
                m.code,
                m.id,
                sl_core::valid_code(m.code).unwrap_or("?")
            )),
            Some(r) if r.code != m.code => problems.push(format!(
                "{} declares code {}, codes.toml gives it {}: a code never changes",
                m.id, m.code, r.code
            )),
            Some(r) if r.status == Status::Retired => problems.push(format!(
                "{} ({}) is retired in codes.toml but the lint exists",
                m.id, m.code
            )),
            Some(_) => {}
        }
    }
    for r in rows.iter().filter(|r| r.status == Status::Active) {
        if !lints.iter().any(|m| m.id == r.id) {
            problems.push(format!(
                "{} ({}) has no lint: set status = \"retired\" and keep the row",
                r.code, r.id
            ));
        }
    }
    if let Some(base) = base {
        for old in parse(base).0 {
            match by_code.get(old.code.as_str()) {
                None => problems.push(format!(
                    "{} ({}) was removed from codes.toml: a code stays reserved, retire it instead",
                    old.code, old.id
                )),
                Some(r) if r.id != old.id || r.group != old.group => problems.push(format!(
                    "{} changed from {} to {}: a code is never reused",
                    old.code, old.id, r.id
                )),
                Some(r) if old.status == Status::Retired && r.status == Status::Active => problems
                    .push(format!(
                        "{} ({}) came back from retired: a new lint takes a new code",
                        r.code, r.id
                    )),
                Some(_) => {}
            }
        }
    }
    problems
}

#[cfg(test)]
mod tests {
    use super::*;
    use sl_core::{FixKind, Source};

    static A: LintMeta = LintMeta {
        code: "S0001",
        id: "crate-lines",
        category: "size",
        summary: "s",
        docs: "d",
        params: &[],
        needs: &[],
        fix: FixKind::None,
        source: Source::Native,
    };
    static B: LintMeta = LintMeta {
        code: "M0001",
        id: "readme-per-crate",
        ..A
    };

    const OK: &str = "S0001 = { id = \"crate-lines\", group = \"size\", status = \"active\" }\nM0001 = { id = \"readme-per-crate\", group = \"manifest\", status = \"active\" }\nM0002 = { id = \"gone\", group = \"manifest\", status = \"retired\" }\n";

    #[test]
    fn the_shipped_registry_matches_the_lints() {
        assert_eq!(check(&crate::lints(), REGISTRY, None), Vec::<String>::new());
    }

    #[test]
    fn a_consistent_registry_passes() {
        assert_eq!(check(&[&A, &B], OK, Some(OK)), Vec::<String>::new());
    }

    #[test]
    fn a_lint_without_a_row_fails() {
        let p = check(&[&A, &B], &OK.replace("M0001", "M0003"), None);
        assert!(
            p.iter()
                .any(|s| s.contains("readme-per-crate declares code M0001")),
            "{p:?}"
        );
        let p = check(&[&A, &B], OK.lines().next().unwrap(), None);
        assert!(p.iter().any(|s| s.contains("has no row")), "{p:?}");
    }

    #[test]
    fn duplicates_fail() {
        let twice = format!(
            "{OK}M0003 = {{ id = \"crate-lines\", group = \"manifest\", status = \"active\" }}\n"
        );
        let p = check(&[&A, &B], &twice, None);
        assert!(
            p.iter()
                .any(|s| s.contains("crate-lines has two codes: S0001 and M0003")),
            "{p:?}"
        );
    }

    #[test]
    fn a_wrong_group_fails() {
        let p = check(&[&A, &B], &OK.replace("\"size\"", "\"code\""), None);
        assert!(p.iter().any(|s| s.contains("group size")), "{p:?}");
    }

    #[test]
    fn changed_removed_and_revived_codes_fail_against_the_base() {
        let changed = OK.replace("id = \"gone\"", "id = \"new-lint\"");
        let p = check(&[&A, &B], &changed, Some(OK));
        assert!(
            p.iter()
                .any(|s| s.contains("M0002 changed from gone to new-lint")),
            "{p:?}"
        );
        let removed: String = OK.lines().take(2).flat_map(|l| [l, "\n"]).collect();
        let p = check(&[&A, &B], &removed, Some(OK));
        assert!(
            p.iter().any(|s| s.contains("M0002 (gone) was removed")),
            "{p:?}"
        );
        let revived = OK.replace("\"retired\"", "\"active\"");
        let p = check(&[&A, &B], &revived, Some(OK));
        assert!(
            p.iter().any(|s| s.contains("came back from retired")),
            "{p:?}"
        );
    }

    #[test]
    fn retiring_is_allowed() {
        let retired = OK.replace(
            "{ id = \"readme-per-crate\", group = \"manifest\", status = \"active\" }",
            "{ id = \"readme-per-crate\", group = \"manifest\", status = \"retired\" }",
        );
        assert_eq!(check(&[&A], &retired, Some(OK)), Vec::<String>::new());
    }
}
