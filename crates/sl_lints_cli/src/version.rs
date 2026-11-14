//! The version line against a template such as `{name} {version} ({sha}, commit {time})`.

/// What a version line held, once it matched the template
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Fields {
    /// `{sha}`: short SHA, maybe `-dirty`, or `unknown`
    pub sha: Option<String>,
    /// `{time}`: `3 Oct 2026 18:20 UTC` or `unknown`
    pub time: Option<String>,
}

/// Matches `line` against `template`; Err says the first part that does not match
pub(crate) fn check(line: &str, template: &str, name: &str) -> Result<Fields, String> {
    let mut fields = Fields::default();
    let mut rest = line;
    let mut t = template;
    while !t.is_empty() {
        if let Some(open) = t.strip_prefix('{') {
            let Some((field, after)) = open.split_once('}') else {
                return Err(format!("the template has an unclosed {{ in `{template}`"));
            };
            // the field's text runs to the next literal of the template, or to the end
            let next_lit: String = after.chars().take_while(|c| *c != '{').collect();
            let end = if next_lit.is_empty() {
                rest.len()
            } else {
                rest.find(next_lit.as_str())
                    .ok_or_else(|| format!("no `{next_lit}` after {{{field}}}"))?
            };
            let value = &rest[..end];
            match field {
                "name" if value != name => {
                    return Err(format!("the name is `{value}`, the binary is `{name}`"));
                }
                "version" if !is_version(value) => {
                    return Err(format!("`{value}` is not a version"));
                }
                "sha" if !is_sha(value) => {
                    return Err(format!("`{value}` is not a short git SHA (or `unknown`)"));
                }
                "time" if !is_time(value) => {
                    return Err(format!(
                        "`{value}` is not a commit time like `3 Oct 2026 18:20 UTC`"
                    ));
                }
                "name" | "version" | "sha" | "time" => {}
                other => return Err(format!("the template names an unknown field {{{other}}}")),
            }
            match field {
                "sha" => fields.sha = Some(value.to_owned()),
                "time" => fields.time = Some(value.to_owned()),
                _ => {}
            }
            rest = &rest[end..];
            t = after;
        } else {
            let lit: String = t.chars().take_while(|c| *c != '{').collect();
            let Some(r) = rest.strip_prefix(lit.as_str()) else {
                return Err(format!("expected `{lit}` at `{rest}`"));
            };
            rest = r;
            t = &t[lit.len()..];
        }
    }
    if rest.is_empty() {
        Ok(fields)
    } else {
        Err(format!("`{rest}` follows the template's end"))
    }
}

/// `0.4.2`, `1.0.0-rc.1+build`: digits first, then version characters
fn is_version(v: &str) -> bool {
    v.starts_with(|c: char| c.is_ascii_digit())
        && v.contains('.')
        && v.chars()
            .all(|c| c.is_ascii_alphanumeric() || ".-+".contains(c))
}

/// 7 to 40 hex digits, maybe `-dirty`; or `unknown`
fn is_sha(v: &str) -> bool {
    let hex = v.strip_suffix("-dirty").unwrap_or(v);
    v == "unknown" || ((7..=40).contains(&hex.len()) && hex.bytes().all(|b| b.is_ascii_hexdigit()))
}

/// `3 Oct 2026 18:20 UTC` or `unknown`
fn is_time(v: &str) -> bool {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    if v == "unknown" {
        return true;
    }
    let parts: Vec<&str> = v.split(' ').collect();
    let digits = |s: &str, n: std::ops::RangeInclusive<usize>| {
        n.contains(&s.len()) && s.bytes().all(|b| b.is_ascii_digit())
    };
    let [day, month, year, hm, zone] = parts.as_slice() else {
        return false;
    };
    let clock = hm
        .split_once(':')
        .is_some_and(|(h, m)| digits(h, 2..=2) && digits(m, 2..=2));
    digits(day, 1..=2) && MONTHS.contains(month) && digits(year, 4..=4) && clock && *zone == "UTC"
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: &str = "{name} {version} ({sha}, commit {time})";

    #[test]
    fn the_house_format() {
        let f = check(
            "spl 0.2.3 (c28e0d7, commit 14 Nov 2026 09:27 UTC)",
            T,
            "spl",
        )
        .unwrap();
        assert_eq!(f.sha.as_deref(), Some("c28e0d7"));
        assert!(
            check(
                "tool 0.4.2 (a1b2c3d-dirty, commit 3 Oct 2026 18:20 UTC)",
                T,
                "tool"
            )
            .is_ok()
        );
        let f = check("cnt 0.5.1 (unknown, commit unknown)", T, "cnt").unwrap();
        assert_eq!(f.sha.as_deref(), Some("unknown"));
    }

    #[test]
    fn mismatches_say_where() {
        let e = check("cnt_cli 0.5.1 (unknown, commit unknown)", T, "cnt").unwrap_err();
        assert!(e.contains("the binary is `cnt`"), "{e}");
        let e = check("ww 0.5.0 (70ceb4f9f, committed 10 Nov 2026 19:20)", T, "ww").unwrap_err();
        assert!(e.contains("no `, commit ` after {sha}"), "{e}");
        let e = check(
            "tool 0.1.0 (abc1234, built 3 Oct 2026 18:20 UTC)",
            T,
            "tool",
        )
        .unwrap_err();
        assert!(e.contains("no `, commit `"), "{e}");
        let e = check("tool 0.1.0", T, "tool").unwrap_err();
        assert!(e.contains("no ` (`"), "{e}");
        assert!(check("tool 0.1.0 (abc1234, commit 3 Oct 2026 18:20)", T, "tool").is_err());
    }
}
