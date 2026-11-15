//! `crate-folder-layer`: a crate's folder follows its layer (and discipline) in the map.

use crate::map_line;
use sl_analysis::Context;
use sl_config::Map;
use sl_core::{Diagnostic, Span};
use sl_model::Project;

/// Folders a crate may sit below the root: its layer, then its discipline
const DEPTH_MAX: usize = 2;

pub(crate) fn crate_folder_layer(cx: &Context<'_>, p: &Project, map: &Map) {
    let params = cx.params("crate-folder-layer");
    let root = params.str("root");
    let prefix = format!("{}/", root.trim_end_matches('/'));
    let placed: Vec<_> = p
        .packages
        .iter()
        .filter_map(|pkg| {
            let rest = pkg.dir.strip_prefix(&prefix)?;
            let mut folders: Vec<&str> = rest.split('/').collect();
            folders.pop();
            Some((pkg, folders))
        })
        .collect();
    let layered = placed.iter().any(|(_, f)| !f.is_empty())
        || map.crates.values().any(|c| c.discipline.is_some());
    for (pkg, folders) in placed {
        let msg = if folders.len() > DEPTH_MAX {
            format!(
                "{} is {} folders below {root}/; crates sit at most two deep, in their layer and then their discipline",
                pkg.dir,
                folders.len()
            )
        } else {
            let Some(me) = map.crates.get(&pkg.name).filter(|_| layered) else {
                continue;
            };
            let mut want = vec![me.layer.as_str()];
            want.extend(me.discipline.as_deref());
            if folders == want {
                continue;
            }
            let by = me.discipline.as_ref().map_or_else(
                || format!("layer {}", me.layer),
                |d| format!("layer {}, discipline {d}", me.layer),
            );
            format!(
                "{} lives in {}; the map puts it in {prefix}{}/{} ({by})",
                pkg.name,
                pkg.dir,
                want.join("/"),
                pkg.name
            )
        };
        cx.emit(
            Diagnostic::new("crate-folder-layer", format!("crate:{}", pkg.name), msg)
                .span(Span::new(pkg.manifest.rel.clone(), 1))
                .span(map_line(map, &pkg.name).label("its map entry"))
                .krate(&pkg.name)
                .help(format!("move the crate to {prefix}<layer>/<crate>, or {prefix}<layer>/<discipline>/<crate> in a layer split by discipline; the map decides the path")),
        );
    }
}
