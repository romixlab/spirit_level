//! The `[board]` table: where a firmware repo keeps its board descriptions and the chip data they are checked
//! against (the board lints).
//!
//! ```toml
//! [board]
//! descriptions = ["boards/*/board.toml"]   # globs of the descriptions, relative to the root
//! chip-data = "boards/chips"               # stm32-data chip JSON, one <CHIP>.json per chip, relative to the root
//! ```
//!
//! The PCB of each revision is named by the description itself (`pcb = "..."`, relative to it).

use crate::error::ConfigError;
use crate::reader::Reader;

/// The descriptions when the table names none
const DEFAULT_DESCRIPTIONS: &[&str] = &["boards/*/board.toml", "board.toml"];
/// The chip data directory when the table names none
const DEFAULT_CHIP_DATA: &str = "boards/chips";

/// Where the board descriptions and their chip data are
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Board {
    /// Globs of the descriptions, relative to the root
    pub descriptions: Vec<String>,
    /// The directory of stm32-data chip JSON files, relative to the root
    pub chip_data: String,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            descriptions: DEFAULT_DESCRIPTIONS
                .iter()
                .map(|s| (*s).to_owned())
                .collect(),
            chip_data: DEFAULT_CHIP_DATA.to_owned(),
        }
    }
}

impl Board {
    pub(crate) fn read(mut r: Reader<'_>, errs: &mut Vec<ConfigError>) -> Self {
        let d = Self::default();
        let descriptions = r
            .opt_list("descriptions", true, errs)
            .unwrap_or(d.descriptions);
        let chip_data = r.opt_str("chip-data", errs).unwrap_or(d.chip_data);
        if descriptions.is_empty() {
            errs.push(r.err(
                "descriptions",
                "board.descriptions is empty: name the descriptions, or drop the key for the default",
            ));
        }
        r.finish(errs);
        Self {
            descriptions,
            chip_data,
        }
    }
}
