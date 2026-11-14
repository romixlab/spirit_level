//! The description against the PCB's nets.

use std::collections::{BTreeMap, BTreeSet};

use sl_board::model::{Board, Located, PinUse};
use sl_board::{McuPins, net_matches};

use crate::{Finding, PCB_COVERED, PCB_NETS};

pub(crate) fn check(board: &Board, mcu: &McuPins) -> Vec<Finding> {
    let mut out = Vec::new();
    let e = &board.entries;
    let mut named: BTreeSet<&str> = BTreeSet::new();
    for u in &e.uses() {
        let pin = &u.pin.pin;
        named.insert(pin.as_str());
        out.extend(used(u, mcu));
    }
    for (pin, at) in &e.free {
        named.insert(pin.as_str());
        out.extend(free(pin, at, mcu));
    }
    let mut groups: Vec<BTreeSet<&str>> = Vec::new();
    for (name, s) in &e.short {
        for p in &s.pins {
            named.insert(p.as_str());
        }
        if let Some(f) = short_apart(name, s.pins.iter().map(|p| p.as_str()), mcu) {
            out.push(f.at(&s.pins[0].span));
        }
        groups.push(s.pins.iter().map(|p| p.as_str()).collect());
    }
    let mut by_net: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (pin, net) in &mcu.nets {
        if let Some(net) = net {
            by_net.entry(net).or_default().push(pin);
            if !named.contains(pin.as_str()) {
                out.push(
                    Finding::new(
                        &PCB_COVERED,
                        "pcb-undescribed",
                        pin.clone(),
                        format!("{pin} is on net {net}, but the description does not name it"),
                    )
                    .help(format!(
                        "describe its job, or add it to [free]: {pin} = \"what it reaches\""
                    )),
                );
            }
        }
    }
    for (net, pins) in by_net {
        if pins.len() > 1 && !groups.iter().any(|g| pins.iter().all(|p| g.contains(p))) {
            out.push(
                Finding::new(
                    &PCB_NETS,
                    "pcb-shared",
                    net,
                    format!(
                        "{} are on one net, {net}, but no [short] says so",
                        pins.join(" and ")
                    ),
                )
                .help(format!("[short.NAME] pins = {pins:?}")),
            );
        }
    }
    out
}

/// A used pin against its pad and net
fn used(u: &PinUse<'_>, mcu: &McuPins) -> Option<Finding> {
    let pin = &u.pin.pin;
    match mcu.net(pin.as_str()) {
        None => Some(
            Finding::new(
                &PCB_NETS,
                "pcb-pad",
                pin.value.clone(),
                format!("{} ({}) has no pad on {}", pin.value, u.owner, mcu.reference),
            )
            .at(&pin.span)
            .help("the MCU's footprint has no pad of that GPIO: check the pin and the package"),
        ),
        Some(None) => Some(
            Finding::new(
                &PCB_NETS,
                "pcb-unconnected",
                pin.value.clone(),
                format!("{} ({}) is not connected on the PCB", pin.value, u.owner),
            )
            .at(&pin.span)
            .help("the firmware would drive or read nothing: use the pin the PCB connects, or drop the entry"),
        ),
        Some(Some(net)) => {
            let want = u.pin.net.as_ref()?;
            (!net_matches(net, &want.value)).then(|| {
                Finding::new(
                    &PCB_NETS,
                    "pcb-net",
                    pin.value.clone(),
                    format!("{} ({}) is on net {net}, not {}", pin.value, u.owner, want.value),
                )
                .at(&want.span)
                .help("use the pin that is on that net, or correct `net`")
            })
        }
    }
}

/// A `[free]` pin against its pad
fn free(pin: &String, at: &Located<String>, mcu: &McuPins) -> Option<Finding> {
    match mcu.net(pin) {
        None => Some(
            Finding::new(
                &PCB_NETS,
                "pcb-pad",
                pin.clone(),
                format!("free pin {pin} has no pad on {}", mcu.reference),
            )
            .at(&at.span)
            .help("drop it from [free]"),
        ),
        Some(None) => Some(
            Finding::new(
                &PCB_COVERED,
                "pcb-free-unconnected",
                pin.clone(),
                format!("free pin {pin} is not connected on the PCB"),
            )
            .at(&at.span)
            .help("a pin that reaches nothing is not worth listing: drop it from [free]"),
        ),
        Some(Some(_)) => None,
    }
}

/// A `[short]` group whose pins are not all on one net
fn short_apart<'a>(
    name: &str,
    mut pins: impl Iterator<Item = &'a str>,
    mcu: &McuPins,
) -> Option<Finding> {
    let first = pins.next()?;
    let first_net = mcu.net(first);
    let show = |n: Option<Option<&str>>| match n {
        None => "no pad".to_owned(),
        Some(None) => "no net".to_owned(),
        Some(Some(n)) => format!("net {n}"),
    };
    for pin in pins {
        let net = mcu.net(pin);
        if net != first_net || net.is_none_or(|n| n.is_none()) {
            return Some(
                Finding::new(
                    &PCB_NETS,
                    "pcb-short",
                    name,
                    format!(
                        "short.{name}: {first} is on {}, {pin} on {}",
                        show(first_net),
                        show(net)
                    ),
                )
                .help("list only pins the PCB joins on one net"),
            );
        }
    }
    None
}
