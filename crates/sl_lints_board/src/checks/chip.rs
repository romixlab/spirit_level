//! The description against the chip: pins the package has, peripherals and signals the chip has, signals on pins
//! that can carry them.

use std::collections::BTreeSet;

use sl_board::model::{Board, BoardInfo, Located, Use};
use sl_board::{ChipData, Package};

use crate::{CHIP_KNOWS, Finding, SIGNAL_PIN};

/// `[board] package` names a package of the chip. Chip data without packages checks pins against the die.
pub(crate) fn package(info: &BoardInfo, chip: &ChipData, out: &mut Vec<Finding>) {
    let Some(p) = &info.package else { return };
    if !chip.packages.is_empty() && chip.package(&p.value).is_none() {
        let known: BTreeSet<&str> = chip.packages.iter().map(|p| p.package.as_str()).collect();
        out.push(
            Finding::new(
                &CHIP_KNOWS,
                "package-unknown",
                p.value.clone(),
                format!("{} comes in no package {}", chip.name, p.value),
            )
            .at(&p.span)
            .help(format!(
                "packages: {}",
                known.into_iter().collect::<Vec<_>>().join(", ")
            )),
        );
    }
}

pub(crate) fn check(board: &Board, info: &BoardInfo, chip: &ChipData, out: &mut Vec<Finding>) {
    let package = info.package.as_ref().and_then(|p| chip.package(&p.value));
    let e = &board.entries;
    // a free pin's place is its key in [free]: the span its note was given
    let free: Vec<Located<String>> = e
        .free
        .iter()
        .map(|(pin, at)| Located {
            value: pin.clone(),
            span: at.span.clone(),
        })
        .collect();
    let mut seen = BTreeSet::new();
    let uses = e.uses();
    let all = uses
        .iter()
        .map(|u| (&u.pin.pin, u.owner.clone()))
        .chain(free.iter().map(|p| (p, "[free]".to_owned())))
        .chain(
            e.short
                .iter()
                .flat_map(|(n, s)| s.pins.iter().map(move |p| (p, format!("short.{n}")))),
        );
    for (pin, owner) in all {
        if !seen.insert((pin.value.clone(), pin.span.start)) {
            continue;
        }
        if let Some(f) = unknown_pin(&pin.value, &owner, chip, package) {
            out.push(f.at(&pin.span));
        }
    }
    for u in &uses {
        let Use::Signal { instance, signal } = u.what else {
            continue;
        };
        let pin = &u.pin.pin;
        let Some(signals) = chip.peripherals.get(instance) else {
            continue;
        };
        let carriers: Vec<_> = chip.signal_pins(instance, signal).collect();
        if carriers.is_empty() {
            let names: BTreeSet<&str> = signals.iter().map(|s| s.signal.as_str()).collect();
            out.push(
                Finding::new(
                    &CHIP_KNOWS,
                    "signal-unknown",
                    format!("{instance} {signal}"),
                    format!("{instance} has no signal {signal} ({})", u.owner),
                )
                .at(&pin.span)
                .help(format!(
                    "{instance} signals: {}",
                    names.into_iter().collect::<Vec<_>>().join(", ")
                )),
            );
        } else if !carriers.iter().any(|c| c.pin == pin.value) {
            let on_package: Vec<String> = carriers
                .iter()
                .filter(|c| package.is_none_or(|p| p.gpios.contains(&c.pin)))
                .map(|c| match c.af {
                    Some(af) => format!("{} (AF{af})", c.pin),
                    None => c.pin.clone(),
                })
                .collect();
            out.push(
                Finding::new(
                    &SIGNAL_PIN,
                    "signal-pin",
                    format!("{instance} {signal} {}", pin.value),
                    format!("{instance} {signal} is not on {} ({})", pin.value, u.owner),
                )
                .at(&pin.span)
                .help(if on_package.is_empty() {
                    "no pin of this package carries it".to_owned()
                } else {
                    format!("pins that carry it: {}", on_package.join(", "))
                }),
            );
        }
    }
    for p in e.periph.values() {
        if !chip.peripherals.contains_key(&p.instance.value)
            && seen.insert((p.instance.value.clone(), p.instance.span.start))
        {
            out.push(unknown_instance(&p.instance, chip));
        }
    }
}

fn unknown_pin(
    pin: &str,
    owner: &str,
    chip: &ChipData,
    package: Option<&Package>,
) -> Option<Finding> {
    match package {
        _ if !chip.gpios.contains(pin) => Some(
            Finding::new(
                &CHIP_KNOWS,
                "pin-unknown",
                pin,
                format!("{pin} ({owner}) is no pin of {}", chip.name),
            )
            .help("name the pin as the chip's data does (`PA0`, `PA0_C`)"),
        ),
        Some(p) if !p.gpios.contains(pin) => Some(
            Finding::new(
                &CHIP_KNOWS,
                "pin-unknown",
                pin,
                format!("{pin} ({owner}) is not on the {} package", p.package),
            )
            .help(format!(
                "the {} die has it; {} does not bring it out",
                chip.name, p.name
            )),
        ),
        _ => None,
    }
}

fn unknown_instance(instance: &Located<String>, chip: &ChipData) -> Finding {
    let stem: String = instance
        .value
        .chars()
        .take_while(char::is_ascii_alphabetic)
        .collect();
    let alike: Vec<&str> = chip
        .peripherals
        .keys()
        .filter(|k| !stem.is_empty() && k.starts_with(&stem))
        .map(String::as_str)
        .collect();
    let f = Finding::new(
        &CHIP_KNOWS,
        "periph-unknown",
        instance.value.clone(),
        format!("{} has no peripheral {}", chip.name, instance.value),
    )
    .at(&instance.span);
    if alike.is_empty() {
        f.help("name the instance as the chip's data does (`FDCAN1`, `USB_OTG_HS`)")
    } else {
        f.help(format!("{} has {}", chip.name, alike.join(", ")))
    }
}
