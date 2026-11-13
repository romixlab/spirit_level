//! `irq-6` Nested disable / enable of interrupts by hand

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use super::util::{text, walk_exprs, walk_macros};
use sl_core::{Confidence, Diagnostic};
use sl_firmware::Firmware;
use sl_firmware::context::bodies;
use syn::spanned::Spanned;
use syn::{Expr, Lit};

lint!(META, "irq-6", "irq", "interrupts are disabled through critical sections, never nested by hand",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// `cortex_m::interrupt::disable()` and `::enable()` (or `cpsid` / `cpsie`, as a call or inline asm) both
/// called in the same function: a hand-rolled critical section, which an outer one or a nested enable can
/// reopen early; `critical_section::with` tracks nesting by itself
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for fb in bodies(&fw.sources) {
        let mut disable = None;
        let mut enable = None;
        walk_exprs(fb.block, &mut |e| {
            let Expr::Call(c) = e else { return };
            let Expr::Path(p) = &*c.func else { return };
            let path = text(&p.path);
            let Some(last) = p.path.segments.last().map(|s| s.ident.to_string()) else {
                return;
            };
            let interrupt_like = path.to_ascii_lowercase().contains("interrupt");
            let slot = match last.as_str() {
                "disable" if interrupt_like => &mut disable,
                "cpsid" => &mut disable,
                "enable" if interrupt_like => &mut enable,
                "cpsie" => &mut enable,
                _ => return,
            };
            if slot.is_none() {
                *slot = Some((fb.loc(p.path.span()), path));
            }
        });
        walk_macros(fb.block, &mut |m| {
            if m.path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .as_deref()
                != Some("asm")
            {
                return;
            }
            let Ok(args) = m.parse_body_with(
                syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated,
            ) else {
                return;
            };
            let Some(Expr::Lit(l)) = args.first() else {
                return;
            };
            let Lit::Str(s) = &l.lit else { return };
            let asm = s.value();
            if disable.is_none() && asm.contains("cpsid") {
                disable = Some((fb.loc(m.span()), "asm!(\"cpsid ..\")".to_string()));
            }
            if enable.is_none() && asm.contains("cpsie") {
                enable = Some((fb.loc(m.span()), "asm!(\"cpsie ..\")".to_string()));
            }
        });
        let (Some((dloc, dtext)), Some((eloc, etext))) = (disable, enable) else {
            continue;
        };
        out.push(
            Diagnostic::new(
                "irq-6",
                fb.path.clone(),
                format!(
                    "{dtext} and {etext} both disable and re-enable interrupts by hand in the same function: \
                     nested under an outer critical section, the inner enable turns interrupts back on too \
                     early"
                ),
            )
            .span(dloc.span().label(dtext))
            .span(eloc.span().label(etext))
            .help("use `critical_section::with(|cs| ..)`, which nests correctly")
            .confidence(Confidence::Estimate),
        );
    }
    out
}
