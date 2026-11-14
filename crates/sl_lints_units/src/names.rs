//! Names and the tables that map their words to `exact_units` types.

use sl_core::Params;

/// The tables of the lints' parameters, parsed once per crate
pub(crate) struct Tables {
    units: Vec<(String, String)>,
    words: Vec<(String, String)>,
    qualifiers: Vec<String>,
    untyped: Vec<String>,
    plain: Vec<(String, String)>,
    pub durations: Vec<String>,
    ranges: Vec<(String, (i128, i128))>,
    pub unbounded: Vec<String>,
}

/// `word=value` entries; an entry without `=` is skipped, as the config check can't see inside strings
fn pairs(list: &[String]) -> Vec<(String, String)> {
    list.iter()
        .filter_map(|e| e.split_once('='))
        .map(|(w, t)| (w.trim().to_ascii_lowercase(), t.trim().to_owned()))
        .collect()
}

impl Tables {
    pub fn new(quantity: &Params, plain: &Params, bounds: &Params) -> Self {
        Self {
            units: pairs(quantity.list("units")),
            words: pairs(quantity.list("words")),
            qualifiers: quantity.list("qualifiers").to_vec(),
            untyped: quantity.list("untyped-units").to_vec(),
            plain: pairs(plain.list("words")),
            durations: plain.list("durations").to_vec(),
            ranges: pairs(bounds.list("ranges"))
                .into_iter()
                .filter_map(|(w, r)| parse_range(&r).map(|r| (w, r)))
                .collect(),
            unbounded: bounds.list("unbounded").to_vec(),
        }
    }

    /// The word that ends the name, before any qualifiers; a one-letter word only after another word
    /// (`temp_c`, never a bare `c`)
    fn last(&self, name: &str) -> Option<String> {
        let ws: Vec<String> = words(name)
            .into_iter()
            .filter(|w| !self.qualifiers.contains(w))
            .collect();
        let last = ws.last()?;
        (last.len() > 1 || ws.len() > 1).then(|| last.clone())
    }

    /// The unit word that ends the name and its type
    pub fn unit(&self, name: &str) -> Option<(String, &str)> {
        let last = self.last(name)?;
        lookup(&self.units, &last).map(|t| (last, t))
    }

    /// Whether the name ends in a unit word that has no type in the table (`frame_size_px`)
    pub fn untyped(&self, name: &str) -> bool {
        self.last(name).is_some_and(|l| self.untyped.contains(&l))
    }

    /// A quantity word anywhere in the name and its type
    pub fn quantity(&self, name: &str) -> Option<(String, &str)> {
        words(name)
            .into_iter()
            .find_map(|w| lookup(&self.words, &w).map(|t| (w, t)))
    }

    /// A word of a position or a count anywhere in the name and its type
    pub fn plain(&self, name: &str) -> Option<(String, &str)> {
        words(name)
            .into_iter()
            .find_map(|w| lookup(&self.plain, &w).map(|t| (w, t)))
    }

    /// The range a word of the name gives the stored value
    pub fn range(&self, name: &str) -> Option<(String, (i128, i128))> {
        words(name).into_iter().find_map(|w| {
            self.ranges
                .iter()
                .find(|(k, _)| *k == w)
                .map(|(_, r)| (w, *r))
        })
    }
}

fn lookup<'t>(table: &'t [(String, String)], word: &str) -> Option<&'t str> {
    table
        .iter()
        .find(|(w, _)| w == word)
        .map(|(_, t)| t.as_str())
}

/// The words of a `snake_case` or `SCREAMING_CASE` name, lowercase
pub(crate) fn words(name: &str) -> Vec<String> {
    name.trim_start_matches("r#")
        .split('_')
        .filter(|s| !s.is_empty())
        .map(str::to_ascii_lowercase)
        .collect()
}

/// `0..=100` or `0..100`
pub(crate) fn parse_range(text: &str) -> Option<(i128, i128)> {
    let (lo, hi, incl) = if let Some((a, b)) = text.split_once("..=") {
        (a, b, true)
    } else {
        let (a, b) = text.split_once("..")?;
        (a, b, false)
    };
    let lo = parse_int(lo.trim())?;
    let hi = parse_int(hi.trim())?;
    let hi = if incl { hi } else { hi - 1 };
    (lo <= hi).then_some((lo, hi))
}

/// An integer as Rust or prose writes it: `-40`, `4_095`, `0x0FFF`
pub(crate) fn parse_int(text: &str) -> Option<i128> {
    let (neg, t) = text.strip_prefix('-').map_or((false, text), |t| (true, t));
    let t = t.replace('_', "");
    let v = if let Some(h) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        i128::from_str_radix(h, 16).ok()?
    } else {
        t.parse().ok()?
    };
    Some(if neg { -v } else { v })
}

/// The template with each `T` replaced by the storage
pub(crate) fn render(template: &str, storage: &str) -> String {
    let mut out = String::with_capacity(template.len() + storage.len());
    let b = template.as_bytes();
    for (i, c) in template.char_indices() {
        let word = |j: usize| {
            b.get(j)
                .is_some_and(|x| x.is_ascii_alphanumeric() || *x == b'_')
        };
        if c == 'T' && !word(i + 1) && (i == 0 || !word(i - 1)) {
            out.push_str(storage);
        } else {
            out.push(c);
        }
    }
    out
}

/// The type with `Range<lo, hi>` (or `RangeDec` for a float) as the bounds of its first `Fixed`, `Index` or `IndexFrom1`
pub(crate) fn with_bounds(ty: &str, range: (i128, i128), float: bool) -> String {
    let bound = if float {
        format!("RangeDec<{}, {}, 0>", range.0, range.1)
    } else {
        format!("Range<{}, {}>", range.0, range.1)
    };
    let Some(start) = ["Fixed<", "IndexFrom1<", "Index<"]
        .iter()
        .find_map(|k| ty.find(k).map(|i| i + k.len()))
    else {
        return ty.to_owned();
    };
    let mut depth = 1;
    for (i, c) in ty[start..].char_indices() {
        match c {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    let at = start + i;
                    return format!("{}, {bound}{}", &ty[..at], &ty[at..]);
                }
            }
            _ => {}
        }
    }
    ty.to_owned()
}

/// The range a numeric storage holds, `None` for floats
pub(crate) fn storage_range(storage: &str) -> Option<(i128, i128)> {
    Some(match storage {
        "u8" => (0, u8::MAX.into()),
        "u16" => (0, u16::MAX.into()),
        "u32" => (0, u32::MAX.into()),
        "u64" | "usize" => (0, u64::MAX.into()),
        "i8" => (i8::MIN.into(), i8::MAX.into()),
        "i16" => (i16::MIN.into(), i16::MAX.into()),
        "i32" => (i32::MIN.into(), i32::MAX.into()),
        "i64" | "isize" => (i64::MIN.into(), i64::MAX.into()),
        _ => return None,
    })
}

/// Whether the storage is a float
pub(crate) fn is_float(storage: &str) -> bool {
    matches!(storage, "f32" | "f64")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_storage_but_not_words_with_t() {
        assert_eq!(
            render("Fixed<Per<Turn, Minute>, One, T>", "u16"),
            "Fixed<Per<Turn, Minute>, One, u16>"
        );
    }

    #[test]
    fn bounds_go_into_the_first_fixed() {
        assert_eq!(
            with_bounds(
                "Point<Celsius, Fixed<Kelvin, Centi, i16>>",
                (-4000, 12500),
                false
            ),
            "Point<Celsius, Fixed<Kelvin, Centi, i16, Range<-4000, 12500>>>"
        );
        assert_eq!(
            with_bounds("Index<Of, u8>", (0, 7), false),
            "Index<Of, u8, Range<0, 7>>"
        );
    }

    #[test]
    fn ranges_parse_inclusive_and_exclusive() {
        assert_eq!(parse_range("0..=100"), Some((0, 100)));
        assert_eq!(parse_range("0..8"), Some((0, 7)));
        assert_eq!(parse_range("-40..=0x7F"), Some((-40, 127)));
    }
}
