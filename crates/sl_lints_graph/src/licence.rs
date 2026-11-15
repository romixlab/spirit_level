//! map-licence: every crate carries the licence the map sets for its layer or for itself.

use crate::map_line;
use sl_analysis::Context;
use sl_config::Map;
use sl_core::{Diagnostic, Edit, Fix};
use sl_model::{Package, Project, toml_edit};

pub(crate) fn check(cx: &Context<'_>, p: &Project, map: &Map) {
    let shared = p
        .root_manifest
        .get(&["workspace", "package", "license"])
        .and_then(|v| v.as_str());
    for pkg in &p.packages {
        let Some(want) = map.licence_of(&pkg.name) else {
            continue;
        };
        if pkg.license.as_deref() == Some(want) {
            continue;
        }
        let has = match (&pkg.license, pkg.manifest.get(&["package", "license-file"])) {
            (Some(l), _) => format!("is under {l}"),
            (None, Some(_)) => "names only a license-file".to_owned(),
            (None, None) => "states no licence".to_owned(),
        };
        let m = &pkg.manifest;
        let at = if m.get(&["package", "license"]).is_some() {
            m.span(&["package", "license"])
        } else {
            m.span(&["package"])
        };
        let mut d = Diagnostic::new(
            "map-licence",
            format!("crate:{}", pkg.name),
            format!("{} {has}; the map sets {want}", pkg.name),
        )
        .span(at)
        .span(map_line(map, &pkg.name).label("its map entry"))
        .krate(&pkg.name)
        .help(if shared == Some(want) {
            "write `license.workspace = true`: the workspace's licence is the one the map sets".to_owned()
        } else {
            format!("write `license = \"{want}\"`, or change the licence of its layer or its entry in the map")
        });
        if let Some(f) = fix(pkg, want, shared == Some(want)) {
            d = d.fix(f);
        }
        cx.emit(d);
    }
}

/// The exact edit: the `license` entry replaced by the map's licence, or written after `name`
fn fix(pkg: &Package, want: &str, inherit: bool) -> Option<Fix> {
    let m = &pkg.manifest;
    if m.get(&["package", "license-file"]).is_some() {
        return None;
    }
    let line = if inherit {
        "license.workspace = true".to_owned()
    } else {
        format!("license = {}", toml_edit::Value::from(want))
    };
    let edit = if let Some(ks) = m.key_span(&["package", "license"]) {
        // `license.workspace = true` is a dotted table whose own span ends at its key
        let vs = if m.get(&["package", "license"])?.is_table() {
            m.item_span(&["package", "license", "workspace"])
        } else {
            m.item_span(&["package", "license"])
        }?;
        Edit {
            file: m.rel.clone(),
            start: ks.start,
            end: vs.end,
            replacement: line,
        }
    } else {
        let ns = m.item_span(&["package", "name"])?;
        let end = m.text[ns.end..]
            .find('\n')
            .map_or(m.text.len(), |i| ns.end + i);
        Edit {
            file: m.rel.clone(),
            start: end,
            end,
            replacement: format!("\n{line}"),
        }
    };
    Some(Fix {
        description: format!("set the licence the map sets, {want}"),
        edits: vec![edit],
    })
}

#[cfg(test)]
mod tests {
    use sl_model::TomlFile;

    #[test]
    fn inline_inheritance_is_not_a_table() {
        let m = TomlFile::parse(
            "Cargo.toml",
            "[package]\nname = \"a\"\nlicense = { workspace = true }\n".into(),
        )
        .unwrap();
        assert!(!m.get(&["package", "license"]).unwrap().is_table());
    }

    #[test]
    fn dotted_inheritance_spans_from_key_to_value() {
        let m = TomlFile::parse(
            "Cargo.toml",
            "[package]\nname = \"a\"\nlicense.workspace = true\n".into(),
        )
        .unwrap();
        let ks = m.key_span(&["package", "license"]).unwrap();
        let vs = m.item_span(&["package", "license", "workspace"]).unwrap();
        assert_eq!(&m.text[ks.start..vs.end], "license.workspace = true");
    }
}
