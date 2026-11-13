//! A configuration error: a file, a line and what is wrong.

use std::fmt;

/// One problem in a configuration or tier file
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ConfigError {
    /// The file, relative to the project root, or `tier:<name>` for a shipped tier
    pub file: String,
    /// 1-based line, 0 when the whole file is meant
    pub line: u32,
    /// What is wrong and how to fix it
    pub message: String,
}

impl ConfigError {
    /// An error at a line of a file
    pub fn new(file: &str, line: u32, message: impl Into<String>) -> ConfigError {
        ConfigError {
            file: file.to_string(),
            line,
            message: message.into(),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 {
            write!(f, "{}: {}", self.file, self.message)
        } else {
            write!(f, "{}:{}: {}", self.file, self.line, self.message)
        }
    }
}

impl std::error::Error for ConfigError {}
