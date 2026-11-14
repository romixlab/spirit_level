//! `cls-1` A file with an executor task has no task class

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::{Diagnostic, Params};
use sl_firmware::Firmware;
use std::collections::BTreeMap;

lint!(META, "F0106", "cls-1", "cls", "every file that holds an executor task has a task class in [embedded.classes]",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];

/// Each file with a task (`#[embassy_executor::task]`) that no glob of `[embedded.classes]` matches, by its path
/// relative to the crate; there is no default class. Without the table, one finding for the firmware.
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let mut files: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for t in &fw.facts.tasks {
        files.entry(t.loc.file.as_str()).or_default().push(&t.name);
    }
    let Some((first, _)) = files.first_key_value() else {
        return Vec::new();
    };
    if fw.classes.is_empty() {
        let t = &fw.facts.tasks.iter().find(|t| t.loc.file == *first).expect("a file of the map has a task").loc;
        return vec![
            Diagnostic::new(
                "cls-1",
                "no-classes",
                format!("no [embedded.classes] table: the {} files with tasks have no task class, so nothing says which may panic", files.len()),
            )
            .span(t.span())
            .help("declare the class of each module in spirit_level.toml: [embedded.classes] with control, service, protocol and boot globs, relative to the crate"),
        ];
    }
    let mut out = Vec::new();
    for (file, tasks) in files {
        if fw.class_of(file).is_some() {
            continue;
        }
        let loc = &fw.facts.tasks.iter().find(|t| t.loc.file == file).expect("a file of the map has a task").loc;
        out.push(
            Diagnostic::new(
                "cls-1",
                file,
                format!("{file} holds tasks ({}) but no glob of [embedded.classes] matches it", tasks.join(", ")),
            )
            .span(loc.span())
            .help("add the file to the class it belongs to: control (drives outputs), service (LED, log, telemetry), protocol (parses input), boot (init)"),
        );
    }
    out
}
