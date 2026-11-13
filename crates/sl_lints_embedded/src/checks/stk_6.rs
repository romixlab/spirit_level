//! `stk-6` Stack frame over a budget (per function) or a worst-case stack depth over the stack region

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Need;
use sl_core::Params;
use sl_core::{Confidence, Diagnostic};
use sl_firmware::Firmware;
use sl_firmware::elf::CallDepths;
use sl_firmware::elf::bytes;

lint!(META, "F0096", "stk-6", "stk", "every stack frame stays within its budget and the deepest call chain within the stack",
    params: [("frame", Int, "2048", "bytes of one function's stack frame above which it is reported")], needs: [Need::Elf], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Elf, FwNeed::Thumb];


/// The bytes an exception entry stacks (r0-r3, r12, lr, pc, xPSR), without the FPU context
const EXCEPTION_FRAME: u64 = 32;

/// Frames from the prologues; each over the `frame` parameter is certain. The worst-case depth is an estimate: the
/// deepest path from `Reset`, plus the deepest function nothing calls directly (an interrupt or exception
/// handler, or code reached through a pointer: an async task's poll, a `dyn` method) on top of it with one
/// exception frame. Calls through pointers are not followed, so it can be low; adding the two can be high.
pub(crate) fn run(fw: &Firmware<'_>, params: &Params) -> Vec<Diagnostic> {
    let Some(elf) = &fw.elf else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for c in &elf.code {
        let Some(frame) = c.frame.filter(|&f| u64::from(f) > crate::limit(params, "frame")) else {
            continue;
        };
        let s = &elf.symbols[c.symbol];
        out.push(
            Diagnostic::new(
                "stk-6",
                format!("elf:frame:{}", s.name),
                format!(
                    "`{}` takes {} of stack in one frame (budget {})",
                    s.name,
                    bytes(frame as u64),
                    bytes(crate::limit(params, "frame"))
                ),
            )
            .span(fw.symbol_span(s))
            .help("build big values in place in a static (`const fn new`, init into `MaybeUninit`) instead of on the stack; split the function so the big locals don't live together; or raise the `frame` parameter of stk-6"),
        );
    }

    let Some(top) = elf.symbol("_stack_start").map(|s| s.addr) else {
        return out;
    };
    let Some(region) = fw.region_at(top.saturating_sub(1)) else {
        return out;
    };
    let floor = elf
        .alloc_sections()
        .filter(|s| s.write && region.contains(s.addr) && s.end() <= top)
        .map(|s| s.end())
        .max()
        .unwrap_or(region.origin);
    let stack = top - floor;
    let Some(reset) = elf
        .symbols
        .iter()
        .position(|s| s.name == "Reset")
        .or_else(|| elf.symbols.iter().position(|s| s.name == "main"))
    else {
        return out;
    };
    let mut called = vec![false; elf.symbols.len()];
    for c in &elf.code {
        for &t in &c.calls {
            called[t] = true;
        }
    }
    let mut d = CallDepths::new(elf);
    let thread = d.depth(reset);
    let mut top_root = None;
    let mut deepest = 0;
    for c in &elf.code {
        if c.symbol == reset || called[c.symbol] {
            continue;
        }
        let x = d.depth(c.symbol);
        if x > deepest {
            deepest = x;
            top_root = Some(c.symbol);
        }
    }
    let worst = thread
        + deepest
        + if top_root.is_some() {
            EXCEPTION_FRAME
        } else {
            0
        };
    if worst > stack {
        let mut msg = format!(
            "worst-case stack about {} over the {} of stack: {} for {}",
            bytes(worst),
            bytes(stack),
            bytes(thread),
            d.path_names(reset).join(" → ")
        );
        if let Some(r) = top_root {
            msg.push_str(&format!(
                ", plus {} for {}",
                bytes(deepest + EXCEPTION_FRAME),
                d.path_names(r).join(" → ")
            ));
        }
        out.push(
            Diagnostic::new("stk-6", "elf:stack-depth", msg)
                .span(fw.region_span(region))
                .help("make the deepest path shallower (big locals into statics, stk-5) or give the stack more RAM; measure the real peak on the board with a painted stack")
                .confidence(Confidence::Estimate),
        );
    }
    // only the firmware's own recursion: core's panic-in-panic path is not
    let own = |name: &str| {
        fw.sources.iter().any(|s| {
            name.starts_with(&format!("{}::", s.krate))
                || name.contains(&format!("<{}::", s.krate))
        })
    };
    let mut rec: Vec<&str> = d
        .recursive()
        .iter()
        .map(|&s| elf.symbols[s].name.as_str())
        .filter(|n| own(n))
        .collect();
    rec.sort_unstable();
    rec.dedup();
    for name in rec {
        out.push(
            Diagnostic::new(
                "stk-6",
                format!("elf:recursion:{name}"),
                format!("`{name}` calls itself (directly or through others): its stack depth has no bound the tool can see"),
            )
            .span(fw.elf_span(name.to_string()))
            .help("bound the recursion (a depth limit) or turn it into a loop")
            .confidence(Confidence::Candidate),
        );
    }
    out
}
