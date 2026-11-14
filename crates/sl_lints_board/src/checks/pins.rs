//! Pins used twice, free pins in use, shorted pins used together: the description against itself.

use std::collections::BTreeMap;

use sl_board::model::{Board, PinUse};

use crate::{Finding, ONE_JOB, SHORT_BOTH};

pub(crate) fn check(board: &Board, out: &mut Vec<Finding>) {
    let e = &board.entries;
    let uses = e.uses();
    let mut by_pin: BTreeMap<&str, Vec<&PinUse<'_>>> = BTreeMap::new();
    for u in &uses {
        by_pin.entry(u.pin.pin.as_str()).or_default().push(u);
    }
    for (pin, us) in &by_pin {
        if let [first, rest @ ..] = us.as_slice() {
            for u in rest {
                out.push(
                    Finding::new(
                        &ONE_JOB,
                        "pin-twice",
                        format!("{pin} {}", u.owner),
                        format!("{pin} is used by {} and by {}", first.owner, u.owner),
                    )
                    .at(&u.pin.pin.span)
                    .help("give the pin one job: drop or move the other entry"),
                );
            }
        }
    }
    for (pin, at) in &e.free {
        if let Some(us) = by_pin.get(pin.as_str()) {
            out.push(
                Finding::new(
                    &ONE_JOB,
                    "free-used",
                    pin.clone(),
                    format!("{pin} is listed free, but {} uses it", us[0].owner),
                )
                .at(&at.span)
                .help("drop it from [free]"),
            );
        }
    }
    let mut group_of: BTreeMap<&str, &str> = BTreeMap::new();
    for (name, s) in &e.short {
        let mut taken = Vec::new();
        for p in &s.pins {
            if let Some(other) = group_of.insert(p.as_str(), name) {
                out.push(
                    Finding::new(
                        &ONE_JOB,
                        "short-twice",
                        p.value.clone(),
                        format!("{p} is in short.{other} and short.{name}: make it one group"),
                    )
                    .at(&p.span)
                    .help("list every pin joined on one net in one [short] group"),
                );
            }
            if let Some(us) = by_pin.get(p.as_str()) {
                taken.push((p, us[0].owner.clone()));
            } else if e.free.contains_key(p.as_str()) {
                taken.push((p, "[free]".to_owned()));
            }
        }
        if let [(a, a_owner), rest @ ..] = taken.as_slice() {
            for (b, b_owner) in rest {
                out.push(
                    Finding::new(
                        &SHORT_BOTH,
                        "short-both",
                        format!("{name} {b}"),
                        format!(
                            "{a} and {b} are joined on the board (short.{name}), but both are used: {a_owner}, {b_owner}"
                        ),
                    )
                    .at(&b.span)
                    .help("use one of them; the other stays an input"),
                );
            }
        }
    }
}
