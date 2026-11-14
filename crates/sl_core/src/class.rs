//! Task classes of firmware modules: what a broken invariant may do in a module, declared per path in the
//! `[embedded.classes]` table of `spirit_level.toml`.

use crate::glob;

/// What a broken invariant may do in a module of firmware
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskClass {
    /// Drives a power stage, motion, a heater or a switch: asserts, and the panic handler puts the outputs into
    /// the board's safe state first
    Control,
    /// Indication, logging, telemetry, settings: never reaches the panic handler in release
    Service,
    /// Parses input from outside: refuses and counts bad input, asserts its own state
    Protocol,
    /// Init before any output is enabled: fails fast
    Boot,
}

impl TaskClass {
    /// Every class, in the order the table lists them
    pub const ALL: [Self; 4] = [Self::Control, Self::Service, Self::Protocol, Self::Boot];

    /// The name in `spirit_level.toml` (`service`)
    pub fn name(self) -> &'static str {
        match self {
            Self::Control => "control",
            Self::Service => "service",
            Self::Protocol => "protocol",
            Self::Boot => "boot",
        }
    }

    /// The class of a name
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.name() == s)
    }
}

/// The class of each module path: globs relative to the crate directory (`src/led.rs`, `src/foc/**`)
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct Classes {
    /// Each class with its globs, in table order
    pub globs: Vec<(TaskClass, Vec<String>)>,
}

impl Classes {
    /// Whether no class is declared
    pub fn is_empty(&self) -> bool {
        self.globs.is_empty()
    }

    /// The class of a file, by its path relative to its crate's directory; the first class in table order whose
    /// glob matches
    pub fn class_of(&self, in_crate: &str) -> Option<TaskClass> {
        self.globs
            .iter()
            .find(|(_, g)| g.iter().any(|p| glob::matches(p, in_crate)))
            .map(|(c, _)| *c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_matching_class_wins() {
        let c = Classes {
            globs: vec![
                (TaskClass::Service, vec!["src/ww/telemetry.rs".into()]),
                (TaskClass::Protocol, vec!["src/ww/**".into()]),
            ],
        };
        assert_eq!(c.class_of("src/ww/telemetry.rs"), Some(TaskClass::Service));
        assert_eq!(c.class_of("src/ww/api.rs"), Some(TaskClass::Protocol));
        assert_eq!(c.class_of("src/main.rs"), None);
    }
}
