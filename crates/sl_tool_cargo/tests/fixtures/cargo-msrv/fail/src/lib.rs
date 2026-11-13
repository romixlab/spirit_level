//! A fixture crate.

/// Locks a file (File::lock is stable since 1.89)
pub fn lock(f: &std::fs::File) -> std::io::Result<()> {
    f.lock()
}
