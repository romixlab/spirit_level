//! The board descriptions of a firmware repo as the board lints read them.
//!
//! - [`find`] lists the `board.toml` files the `[board]` table's globs select.
//! - [`load`] reads one through [`sl_board_model`] (the schema's one loader: spans, revisions
//!   and reworks as patches), then its inputs: the chip's data ([`ChipData`], stm32-data's chip JSON) and the MCU's
//!   nets of every PCB its configurations name ([`McuPins`], boardmap's geometry JSON).
//!
//! An input that cannot be had is [`Input::Missing`], with the reason, so the lints that need it report themselves
//! as not run; an input that is there but wrong is [`Input::Invalid`], a finding at the place the description names
//! it.

mod chip;
mod pcb;

use std::collections::BTreeMap;
use std::io::Read;
use std::ops::Range;
use std::path::Path;

pub use chip::{ChipData, ChipError, Package, SignalPin};
pub use pcb::{McuPins, PcbError, net_matches};
pub use sl_board_model as model;

use sl_board_model::{Description, LoadError, Located};

/// The largest chip JSON read; stm32-data's biggest chip files are under 1 MiB
const CHIP_JSON_BYTES_MAX: u64 = 16 << 20;
/// The largest geometry JSON read; a whole board's is a few hundred KiB
const GEOMETRY_JSON_BYTES_MAX: u64 = 16 << 20;

/// An input of the checks.
#[derive(Debug)]
pub enum Input<T> {
    /// Read.
    Ready(T),
    /// Not there, or of a kind spl does not read: why. The lints that need it do not run.
    Missing(String),
    /// There but wrong: what, where in the description it is named, and what helps.
    Invalid {
        /// What is wrong
        message: String,
        /// Byte range of the value in the description that names the input
        at: Range<usize>,
        /// What would fix it
        help: Option<String>,
    },
}

impl<T> Input<T> {
    /// The input when it was read.
    pub fn ready(&self) -> Option<&T> {
        match self {
            Self::Ready(t) => Some(t),
            Self::Missing(_) | Self::Invalid { .. } => None,
        }
    }
}

/// A description with its inputs.
#[derive(Debug)]
pub struct Loaded {
    /// The description; its `source.path` is the file under the root.
    pub description: Description,
    /// The chip's data.
    pub chip: Input<ChipData>,
    /// The MCU's nets of every PCB a configuration names, by the `pcb` value as the description writes it.
    pub pcbs: BTreeMap<String, Input<McuPins>>,
}

/// The descriptions under `root` that one of `globs` selects, relative to the root and sorted; `members` are the
/// directories of the workspace members (other nested cargo projects are not walked).
pub fn find(root: &Path, members: &[&str], globs: &[String]) -> Vec<String> {
    sl_docs::walk::files(root, members, globs, &[])
}

/// Reads the description `file` (relative to `root`) and its inputs; chip data comes from `chip_data` (a directory
/// relative to the root, one `<CHIP>.json` per chip).
pub fn load(root: &Path, file: &str, chip_data: &str) -> Result<Loaded, LoadError> {
    let description = Description::load(&root.join(file))?;
    let chip = chip(root, &description, chip_data);
    let pcbs = pcbs(root, &description, chip.ready());
    Ok(Loaded {
        description,
        chip,
        pcbs,
    })
}

/// `path` relative to `root` for messages, with `/`
fn shown(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn chip(root: &Path, desc: &Description, dir: &str) -> Input<ChipData> {
    let name = &desc.board.chip;
    let upper = name.value.to_ascii_uppercase();
    if !upper.starts_with("STM32") {
        return Input::Missing(format!(
            "no pin data for {}: chip data covers STM32 (stm32-data)",
            name.value
        ));
    }
    let path = root.join(dir).join(format!("{upper}.json"));
    let text = match read_bounded(&path, CHIP_JSON_BYTES_MAX) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Input::Missing(format!(
                "chip data missing: {} is not there (stm32-data's data/chips/{upper}.json)",
                shown(root, &path)
            ));
        }
        Err(e) => return invalid(format!("{}: {e}", shown(root, &path)), name, None),
    };
    match ChipData::from_stm32_data(&text) {
        Ok(c) => Input::Ready(c),
        Err(e) => invalid(
            format!("{}: {e}", shown(root, &path)),
            name,
            Some(format!(
                "replace it with stm32-data's data/chips/{upper}.json"
            )),
        ),
    }
}

fn invalid<T>(message: String, at: &Located<String>, help: Option<String>) -> Input<T> {
    Input::Invalid {
        message,
        at: at.span.clone(),
        help,
    }
}

fn pcbs(
    root: &Path,
    desc: &Description,
    chip: Option<&ChipData>,
) -> BTreeMap<String, Input<McuPins>> {
    let mut out = BTreeMap::new();
    for board in desc.configurations() {
        let Some(pcb) = &board.pcb else { continue };
        if !out.contains_key(&pcb.value) {
            out.insert(pcb.value.clone(), mcu_pins(root, desc, pcb, chip));
        }
    }
    out
}

fn mcu_pins(
    root: &Path,
    desc: &Description,
    pcb: &Located<String>,
    chip: Option<&ChipData>,
) -> Input<McuPins> {
    let Some(mcu) = &desc.board.mcu else {
        return invalid(
            "the PCB's MCU is not named".to_owned(),
            pcb,
            Some("name its reference in [board]: mcu = \"U1\"".to_owned()),
        );
    };
    let path = desc.dir().join(&pcb.value);
    if path.extension().is_some_and(|e| e == "kicad_pcb") {
        return Input::Missing(format!(
            "{} is a KiCad board: spl reads boardmap's geometry JSON (`boardmap extract`)",
            shown(root, &path)
        ));
    }
    let text = match read_bounded(&path, GEOMETRY_JSON_BYTES_MAX) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Input::Missing(format!("PCB missing: {} is not there", shown(root, &path)));
        }
        Err(e) => return invalid(format!("{}: {e}", shown(root, &path)), pcb, None),
    };
    match McuPins::from_geometry(&text, &mcu.value, chip) {
        Ok(m) => Input::Ready(m),
        Err(e @ PcbError::Json(_)) => invalid(format!("{}: {e}", shown(root, &path)), pcb, None),
        Err(e) => invalid(
            format!("{} ({}): {e}", shown(root, &path), mcu.value),
            mcu,
            None,
        ),
    }
}

fn read_bounded(path: &Path, max_bytes: u64) -> std::io::Result<String> {
    let mut text = String::new();
    std::fs::File::open(path)?
        .take(max_bytes + 1)
        .read_to_string(&mut text)?;
    if text.len() as u64 > max_bytes {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("longer than {max_bytes} bytes"),
        ));
    }
    Ok(text)
}
