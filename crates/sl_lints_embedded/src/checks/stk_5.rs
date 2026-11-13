//! `stk-5` Big state built by value and moved into a static

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Need;
use sl_core::Params;
use sl_core::{Confidence, Diagnostic, Span};
use sl_firmware::Firmware;
use sl_firmware::elf::bytes;

lint!(META, "stk-5", "stk", "big state is built in place, not by value and moved into a static",
    params: [("move-frame", Int, "1024", "bytes of frame from which a function that moves a value into a static is reported")], needs: [Need::Elf], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources, FwNeed::Elf, FwNeed::Thumb];


/// The ELF part: a source function that calls `.init(..)` on a `StaticCell` static, whose code in the ELF (the
/// function, its async body or the embassy task it is) has a frame of the `move-frame` parameter bytes or more. An
/// estimate: the frame may hold other things too.
pub(crate) fn run(fw: &Firmware<'_>, params: &Params) -> Vec<Diagnostic> {
    let Some(elf) = &fw.elf else {
        return Vec::new();
    };
    let cells: Vec<&str> = fw
        .facts
        .statics
        .iter()
        .filter(|s| s.ty.contains("StaticCell"))
        .map(|s| s.name.as_str())
        .collect();
    if cells.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for f in &fw.facts.functions {
        let Some(text) = fw.source(&f.loc.file).map(|s| s.text.as_str()) else {
            continue;
        };
        let lines: Vec<&str> = text.lines().collect();
        let Some((call, cell)) = f.calls.iter().filter(|c| c.name == ".init").find_map(|c| {
            let line = lines.get(c.loc.line.saturating_sub(1) as usize)?;
            let cell = cells
                .iter()
                .find(|n| line.contains(&format!("{n}.init(")))?;
            Some((c, *cell))
        }) else {
            continue;
        };
        // the function itself, its async body, or the embassy task wrapping it
        let (module, name) = f.path.rsplit_once("::").unwrap_or(("", f.path.as_str()));
        let task = format!("{module}::__{name}_task");
        let main_task = format!("{module}::____embassy_{name}_task");
        let frame = elf
            .code
            .iter()
            .filter(|c| {
                let n = &elf.symbols[c.symbol].name;
                *n == f.path
                    || n.starts_with(&format!("{}::{{closure", f.path))
                    || n.contains(&task)
                    || n.contains(&main_task)
            })
            .filter_map(|c| c.frame)
            .max();
        let Some(frame) = frame.filter(|&fr| u64::from(fr) >= crate::limit(params, "move-frame")) else {
            continue;
        };
        out.push(
            Diagnostic::new(
                "stk-5",
                format!("{}:{cell}.init", f.path),
                format!(
                    "`{}` moves a value into the static `{cell}` with `.init(..)`; its frame in the ELF is {}: the value is built on the stack first",
                    f.path,
                    bytes(frame as u64)
                ),
            )
            .span(Span::at(&call.loc.file, call.loc.line, call.loc.column))
            .help("build it in place: a `const fn new` and a plain `static` / `ConstStaticCell`, or `uninit()` and initialize the fields through the `MaybeUninit`")
            .confidence(Confidence::Estimate),
        );
    }
    out
}
