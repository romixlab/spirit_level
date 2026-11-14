//! The PCB side of the board lints: which net each MCU pin is on, read from boardmap's geometry JSON (`embedded_bedrock`
//! `bedrock_board::Geometry`). Only the footprints' pads are read; every other field is left alone.

use std::collections::BTreeMap;
use std::fmt;

use serde::Deserialize;

use crate::chip::ChipData;

/// The MCU's pins on the PCB.
#[derive(Debug, Clone, Default)]
pub struct McuPins {
    /// The MCU's reference (`U201`).
    pub reference: String,
    /// GPIO name (`PC14`, not the symbol's `PC14-OSC32_IN`) to its net; `None` when nothing else is on the net
    /// (a `KiCad` net named `unconnected-...`).
    pub nets: BTreeMap<String, Option<String>>,
}

impl McuPins {
    /// The net of `pin`: `None` when the PCB has no such pad, `Some(None)` when the pad is not connected.
    pub fn net(&self, pin: &str) -> Option<Option<&str>> {
        self.nets.get(pin).map(Option::as_deref)
    }

    /// Reads the pads of footprint `reference` from geometry JSON; with `chip`, a pad's pin name is matched against
    /// its GPIOs, without it against the STM32 pin pattern (`PA0`, `PA0_C`).
    pub fn from_geometry(
        json: &str,
        reference: &str,
        chip: Option<&ChipData>,
    ) -> Result<Self, PcbError> {
        let geo: RawGeometry = serde_json::from_str(json).map_err(PcbError::Json)?;
        let Some(fp) = geo.footprints.get(reference) else {
            return Err(PcbError::NoFootprint(geo.footprints.into_keys().collect()));
        };
        let mut nets = BTreeMap::new();
        for pad in &fp.pads {
            let Some(name) = pad.pin.as_deref() else {
                continue;
            };
            let candidates = [Some(name), name.split_once('-').map(|(p, _)| p)];
            let gpio = candidates.into_iter().flatten().find(|p| match chip {
                Some(c) => c.gpios.contains(*p),
                None => is_stm32_gpio(p),
            });
            if let Some(gpio) = gpio {
                let net = pad
                    .net
                    .clone()
                    .filter(|n| !n.is_empty() && !is_unconnected(n));
                nets.insert(gpio.to_owned(), net);
            }
        }
        if nets.is_empty() {
            return Err(PcbError::NoGpioPads);
        }
        Ok(Self {
            reference: reference.to_owned(),
            nets,
        })
    }
}

/// Why a PCB's geometry gave no MCU pins.
#[derive(Debug)]
pub enum PcbError {
    /// The file is not boardmap's geometry JSON
    Json(serde_json::Error),
    /// No footprint has the MCU's reference; the references it has
    NoFootprint(Vec<String>),
    /// The MCU's pads carry no GPIO names (an imported vendor board)
    NoGpioPads,
}

impl fmt::Display for PcbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(_) => f.write_str("not boardmap geometry JSON"),
            Self::NoFootprint(refs) => {
                write!(
                    f,
                    "no footprint of the MCU; footprints: {}",
                    refs.join(", ")
                )
            }
            Self::NoGpioPads => f.write_str("no pad of the MCU carries a GPIO name"),
        }
    }
}

impl std::error::Error for PcbError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Json(e) => Some(e),
            Self::NoFootprint(_) | Self::NoGpioPads => None,
        }
    }
}

/// Whether the description's `net = "..."` names the PCB's net: the whole name, or its last path segment
/// (`LED_R` is `/MCU_H7/LED_R`).
pub fn net_matches(pcb: &str, described: &str) -> bool {
    pcb == described || pcb.rsplit('/').next() == Some(described)
}

/// The name `KiCad` gives a net with one pad
fn is_unconnected(net: &str) -> bool {
    net.to_ascii_lowercase().starts_with("unconnected")
}

/// `P`, a port letter, a number, optionally `_C` (`PA0`, `PC14`, `PA0_C`)
fn is_stm32_gpio(name: &str) -> bool {
    let name = name.strip_suffix("_C").unwrap_or(name);
    let mut chars = name.chars();
    chars.next() == Some('P')
        && chars.next().is_some_and(|c| c.is_ascii_uppercase())
        && !chars.as_str().is_empty()
        && chars.all(|c| c.is_ascii_digit())
}

#[derive(Deserialize)]
struct RawGeometry {
    #[serde(default)]
    footprints: BTreeMap<String, RawFootprint>,
}

#[derive(Deserialize)]
struct RawFootprint {
    #[serde(default)]
    pads: Vec<RawPad>,
}

#[derive(Deserialize)]
struct RawPad {
    #[serde(default)]
    pin: Option<String>,
    #[serde(default)]
    net: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const GEO: &str = r#"{"source": "x", "bbox": [0, 0, 1, 1], "footprints": {"U1": {"value": "MCU", "pads": [
        {"number": "1", "pin": "PA0", "net": "/MCU/LED"},
        {"number": "2", "pin": "PC14-OSC32_IN", "net": "unconnected-(U1-PC14-Pad2)"},
        {"number": "3", "pin": "VSS", "net": "GND"},
        {"number": "4", "pin": "PA0_C", "net": ""}
    ]}}}"#;

    #[test]
    fn pads_to_gpio_nets() {
        let m = McuPins::from_geometry(GEO, "U1", None).unwrap();
        assert_eq!(m.net("PA0"), Some(Some("/MCU/LED")));
        assert_eq!(m.net("PC14"), Some(None));
        assert_eq!(m.net("PA0_C"), Some(None));
        assert_eq!(m.net("VSS"), None);
        assert!(net_matches("/MCU/LED", "LED"));
        assert!(!net_matches("/MCU/LED", "MCU"));
    }

    #[test]
    fn wrong_reference_names_the_footprints() {
        let e = McuPins::from_geometry(GEO, "U2", None).unwrap_err();
        assert_eq!(e.to_string(), "no footprint of the MCU; footprints: U1");
        assert!(matches!(
            McuPins::from_geometry("1", "U1", None),
            Err(PcbError::Json(_))
        ));
    }
}
