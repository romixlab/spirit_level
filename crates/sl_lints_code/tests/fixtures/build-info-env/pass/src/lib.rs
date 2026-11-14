//! Reads build info at run time; names that are not build info are fine.

/// The crate's own name and the manifest directory
pub fn name() -> (&'static str, &'static str) {
    (env!("CARGO_PKG_NAME"), env!("CARGO_MANIFEST_DIR"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn rev() {
        assert!(!env!("FX_REV").is_empty());
    }
}
