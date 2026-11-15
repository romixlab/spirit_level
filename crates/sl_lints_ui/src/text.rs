//! Checks of the text a person reads: counts of hidden items, punctuation, debug formatting, destructive verbs.

use sl_analysis::syntax::range;

/// One replacement: byte start, byte end, new text
pub(crate) type Edit = (usize, usize, String);

/// Whether a text says how many items are not shown: a number or a placeholder, then one of `words`
/// (`"{} more"`, `"and 3 others"`)
pub(crate) fn more_count(value: &str, words: &[String]) -> bool {
    let toks: Vec<&str> = value.split_whitespace().collect();
    toks.windows(2).any(|w| {
        let counted = w[0].ends_with('}') || w[0].ends_with(|c: char| c.is_ascii_digit());
        let word = w[1].trim_end_matches(|c: char| !c.is_alphanumeric());
        counted && words.iter().any(|x| x.eq_ignore_ascii_case(word))
    })
}

/// Whether a format string shows a value in Rust's debug form (`{:?}`, `{e:?}`, `{:#?}`)
pub(crate) fn has_debug(value: &str) -> bool {
    value.contains(":?}") || value.contains(":#?}")
}

/// Whether a literal shown to a person needs a translation catalogue entry: outside any `{..}` format placeholder,
/// it holds a letter, not just numbers, placeholders or punctuation
pub(crate) fn needs_locale(value: &str) -> bool {
    let mut depth = 0u32;
    let mut letter = false;
    for c in value.chars() {
        match c {
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            _ if depth == 0 && c.is_alphabetic() => letter = true,
            _ => {}
        }
    }
    letter
}

/// The verb a label starts with, when it is one of `verbs` and the label does not open a dialog (ends in "…")
pub(crate) fn destructive_verb(value: &str, verbs: &[String]) -> Option<String> {
    let v = value.trim();
    if v.ends_with('…') || v.ends_with("...") {
        return None;
    }
    let first = v
        .split_whitespace()
        .next()?
        .trim_matches(|c: char| !c.is_alphanumeric());
    verbs
        .iter()
        .any(|x| x.eq_ignore_ascii_case(first))
        .then(|| first.to_owned())
}

/// The byte range of a string literal's source text in `text`
fn literal_bytes(text: &str, lit: &syn::LitStr) -> Option<(usize, usize)> {
    let ((l0, c0), (l1, c1)) = range(lit);
    let start = byte_at(text, l0, c0)?;
    let end = byte_at(text, l1, c1)?;
    (start < end).then_some((start, end))
}

/// The byte offset of a 1-based line and character column
fn byte_at(text: &str, line: u32, col: u32) -> Option<usize> {
    let mut off = 0;
    for (i, l) in text.split_inclusive('\n').enumerate() {
        if i + 1 == line as usize {
            let c = col.checked_sub(1)? as usize;
            return match l.char_indices().nth(c) {
                Some((b, _)) => Some(off + b),
                None => (c == l.chars().count()).then_some(off + l.len()),
            };
        }
        off += l.len();
    }
    None
}

/// The edits that write each "..." of a literal as "…"
pub(crate) fn ellipsis_edits(text: &str, lit: &syn::LitStr) -> Vec<Edit> {
    let Some((s, e)) = literal_bytes(text, lit) else {
        return Vec::new();
    };
    text[s..e]
        .match_indices("...")
        .map(|(i, _)| (s + i, s + i + 3, "…".to_owned()))
        .collect()
}

/// The edit that drops a label's closing full stop, when it ends in one (not in "..")
pub(crate) fn full_stop_edit(text: &str, lit: &syn::LitStr) -> Option<Edit> {
    let v = lit.value();
    if !v.ends_with('.') || v.ends_with("..") {
        return None;
    }
    let (s, e) = literal_bytes(text, lit)?;
    let src = &text[s..e];
    let q = src.rfind('"')?;
    (src[..q].ends_with('.') && !src[..q].ends_with(".."))
        .then(|| (s + q - 1, s + q, String::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(w: &[&str]) -> Vec<String> {
        w.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn counts_after_a_number_or_placeholder() {
        let w = words(&["more", "others"]);
        assert!(more_count("and {} more", &w));
        assert!(more_count("+{n} more…", &w));
        assert!(more_count("3 others", &w));
        assert!(!more_count("Show more", &w));
        assert!(!more_count("more options", &w));
    }

    #[test]
    fn locale_needs_a_letter() {
        assert!(needs_locale("Save"));
        assert!(needs_locale("3 items left"));
        assert!(!needs_locale("{}"));
        assert!(!needs_locale("..."));
        assert!(!needs_locale(""));
        // a named placeholder's letters are not prose
        assert!(!needs_locale("{count}"));
        assert!(needs_locale("{count} items left"));
    }

    #[test]
    fn verbs_open_a_dialog_with_an_ellipsis() {
        let v = words(&["delete", "reset"]);
        assert_eq!(
            destructive_verb("Delete file", &v).as_deref(),
            Some("Delete")
        );
        assert_eq!(destructive_verb("Reset storage to defaults…", &v), None);
        assert_eq!(destructive_verb("Open", &v), None);
    }

    #[test]
    fn fixes_point_into_the_literal() {
        let text = "fn f() { b(\"Open...\"); c(\"Save.\"); }\n";
        let f: syn::File = syn::parse_str(text).unwrap();
        let mut literals = Vec::new();
        struct V<'a>(&'a mut Vec<syn::LitStr>);
        impl<'ast> syn::visit::Visit<'ast> for V<'_> {
            fn visit_lit_str(&mut self, l: &'ast syn::LitStr) {
                self.0.push(l.clone());
            }
        }
        syn::visit::Visit::visit_file(&mut V(&mut literals), &f);
        let e = ellipsis_edits(text, &literals[0]);
        assert_eq!(e.len(), 1);
        assert_eq!(&text[e[0].0..e[0].1], "...");
        let s = full_stop_edit(text, &literals[1]).unwrap();
        assert_eq!(&text[s.0..s.1], ".");
        assert_eq!(&text[s.1..=s.1], "\"");
    }
}
