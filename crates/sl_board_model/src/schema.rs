//! The resolved description: what `board.toml` says, with every value's place in the file.

use std::collections::BTreeMap;

use crate::source::Located;

/// `[board]`: the board as a whole.
#[derive(Debug, Clone)]
pub struct BoardInfo {
    /// Board name as the hardware repository calls it (`B135`).
    pub name: String,
    /// Chip as stm32-data names it (`STM32H725IG`).
    pub chip: Located<String>,
    /// Package as stm32-data names it (`UFBGA176`); pins are checked against it when given.
    pub package: Option<Located<String>>,
    /// Reference of the MCU on the PCB (`U201`).
    pub mcu: Option<Located<String>>,
    /// PCB file relative to the description: a `.kicad_pcb` or boardmap's geometry JSON.
    pub pcb: Option<Located<String>>,
    /// Every revision of the board one firmware supports (`["A", "B"]`); empty: the board has one.
    pub revisions: Vec<Located<String>>,
}

/// A logic level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    /// Logic high.
    High,
    /// Logic low.
    Low,
}

/// An input's pull resistor inside the chip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Pull {
    /// Pull-up.
    Up,
    /// Pull-down.
    Down,
    /// None inside the chip: the board has its own, or a driver holds the line.
    None,
}

/// A pin, and optionally the PCB net it must be on (`net = "LED_R"` matches `/MCU_H7/LED_R`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinRef {
    /// The pin (`PB9`).
    pub pin: Located<String>,
    /// The net it must be on, when the description says.
    pub net: Option<Located<String>>,
}

/// `[gpio.NAME]`: a pin the firmware drives or reads itself.
#[derive(Debug, Clone)]
pub struct Gpio {
    /// The pin.
    pub pin: PinRef,
    /// Output, input or analog.
    pub mode: GpioMode,
    /// Free text about the pin.
    pub note: Option<String>,
}

/// What a GPIO is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpioMode {
    /// Driven from the first instructions after reset at `init`.
    Output {
        /// The level driven from reset.
        init: Level,
    },
    /// Read by the firmware.
    Input {
        /// The pull inside the chip.
        pull: Pull,
    },
    /// Analog: an ADC input or a pin kept off the digital input buffer.
    Analog,
}

/// `[periph.NAME]`: a peripheral instance and its signals' pins (`instance = "FDCAN1"`, `pins = { TX = "PB9" }`).
#[derive(Debug, Clone)]
pub struct Periph {
    /// The instance as stm32-data names it (`FDCAN1`).
    pub instance: Located<String>,
    /// Signal (as stm32-data names it: `TX`, `ULPI_D0`, `INP2`) to pin.
    pub pins: BTreeMap<String, PinRef>,
    /// Free text about the peripheral.
    pub note: Option<String>,
}

/// An LED's pins.
#[derive(Debug, Clone)]
pub enum LedPins {
    /// One LED.
    Mono(PinRef),
    /// Red, green, blue.
    Rgb([PinRef; 3]),
}

/// `[led.NAME]`.
#[derive(Debug, Clone)]
pub struct Led {
    /// Its pin, or its three pins.
    pub pins: LedPins,
    /// The level that lights it.
    pub active: Level,
    /// Free text about the LED.
    pub note: Option<String>,
}

/// `[button.NAME]`.
#[derive(Debug, Clone)]
pub struct Button {
    /// The pin.
    pub pin: PinRef,
    /// The level while it is pressed.
    pub active: Level,
    /// The pull inside the chip.
    pub pull: Pull,
    /// Free text about the button.
    pub note: Option<String>,
}

/// `[reserved.NAME]`: pins the firmware leaves in their reset function (SWD).
#[derive(Debug, Clone)]
pub struct Reserved {
    /// The pins.
    pub pins: Vec<PinRef>,
    /// What they are reserved for.
    pub note: Option<String>,
}

/// `[short.NAME]`: MCU pins joined on the board. At most one of them may be used; the others stay inputs.
#[derive(Debug, Clone)]
pub struct Short {
    /// The joined pins.
    pub pins: Vec<Located<String>>,
    /// Where they are joined, and which one stays an input.
    pub note: Option<String>,
}

/// Every entry of one configuration (or of the base, or of a patch).
#[derive(Debug, Clone, Default)]
pub struct Entries {
    /// `[gpio.NAME]` entries by name.
    pub gpio: BTreeMap<String, Gpio>,
    /// `[periph.NAME]` entries by name.
    pub periph: BTreeMap<String, Periph>,
    /// `[led.NAME]` entries by name.
    pub led: BTreeMap<String, Led>,
    /// `[button.NAME]` entries by name.
    pub button: BTreeMap<String, Button>,
    /// `[reserved.NAME]` entries by name.
    pub reserved: BTreeMap<String, Reserved>,
    /// `[short.NAME]` entries by name.
    pub short: BTreeMap<String, Short>,
    /// `[free]`: pins with a net but no job on the board (a header pin), by pin, with what they reach.
    pub free: BTreeMap<String, Located<String>>,
}

/// What a pin is used for in one entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Use<'a> {
    /// A GPIO of this mode.
    Gpio(GpioMode),
    /// A peripheral's signal.
    Signal {
        /// The instance (`FDCAN1`).
        instance: &'a str,
        /// The signal (`TX`).
        signal: &'a str,
    },
    /// An LED.
    Led,
    /// A button.
    Button,
    /// A reserved pin.
    Reserved,
}

/// A pin an entry uses.
#[derive(Debug, Clone)]
pub struct PinUse<'a> {
    /// The pin.
    pub pin: &'a PinRef,
    /// The entry and role, for messages: `periph.can1 TX`, `led.status green`.
    pub owner: String,
    /// What the entry does with it.
    pub what: Use<'a>,
}

impl Entries {
    /// Every pin an entry uses (not the free pins and not the shorted pairs), in a stable order.
    pub fn uses(&self) -> Vec<PinUse<'_>> {
        let mut out = Vec::new();
        for (name, g) in &self.gpio {
            out.push(PinUse {
                pin: &g.pin,
                owner: format!("gpio.{name}"),
                what: Use::Gpio(g.mode),
            });
        }
        for (name, p) in &self.periph {
            for (signal, pin) in &p.pins {
                out.push(PinUse {
                    pin,
                    owner: format!("periph.{name} {signal}"),
                    what: Use::Signal {
                        instance: &p.instance.value,
                        signal,
                    },
                });
            }
        }
        for (name, l) in &self.led {
            match &l.pins {
                LedPins::Mono(pin) => out.push(PinUse {
                    pin,
                    owner: format!("led.{name}"),
                    what: Use::Led,
                }),
                LedPins::Rgb(pins) => {
                    for (pin, colour) in pins.iter().zip(["red", "green", "blue"]) {
                        out.push(PinUse {
                            pin,
                            owner: format!("led.{name} {colour}"),
                            what: Use::Led,
                        });
                    }
                }
            }
        }
        for (name, b) in &self.button {
            out.push(PinUse {
                pin: &b.pin,
                owner: format!("button.{name}"),
                what: Use::Button,
            });
        }
        for (name, r) in &self.reserved {
            for pin in &r.pins {
                out.push(PinUse {
                    pin,
                    owner: format!("reserved.{name}"),
                    what: Use::Reserved,
                });
            }
        }
        out
    }
}

/// One configuration of the board: a revision with the reworks fitted, as a complete description.
#[derive(Debug, Clone)]
pub struct Board {
    /// `rev B`, `rev A + rework 5`, or the board's name when it has no revisions.
    pub label: String,
    /// The revision (`B`); None when the board has none.
    pub revision: Option<String>,
    /// The reworks fitted, in order.
    pub reworks: Vec<String>,
    /// The `note`s of the revision and reworks.
    pub notes: Vec<String>,
    /// The PCB file of this revision.
    pub pcb: Option<Located<String>>,
    /// Every entry, the patches applied.
    pub entries: Entries,
}
