//! types
use std::fs;
pub fn load() -> Vec<u8> { fs::read("x").unwrap_or_default() }
