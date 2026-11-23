//! Value-returning functions whose bodies end in a call named like a lock's
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};

/// A table kept in a file, changed under a file lock
pub struct Table {
    path: PathBuf,
}

impl Table {
    /// Opens the table at `path`
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// The file lock: a value, not a guard
    fn lock(&self) -> std::io::Result<File> {
        File::open(&self.path)
    }

    /// The table's bytes
    fn read(&self) -> std::io::Result<Vec<u8>> {
        std::fs::read(&self.path)
    }

    /// The table, read under the file lock: `self.lock()` and `self.read()` are the methods above
    pub fn load(&self) -> std::io::Result<Vec<u8>> {
        let _lock = self.lock()?;
        self.read()
    }
}

/// A list read from a folder
pub struct List(pub Vec<u8>);

impl List {
    /// Reads the list: an associated function named like the helper above
    pub fn load(dir: &Path) -> std::io::Result<Self> {
        std::fs::read(dir).map(Self)
    }
}

/// The folder's state: a free function named like a method that returns a value
pub fn state(t: &Table) -> std::io::Result<Vec<u8>> {
    t.load()
}

/// State behind a lock, taken through a method of the same name as the free function above
pub struct Shared(pub Mutex<u8>);

impl Shared {
    /// The guard
    pub fn state(&self) -> MutexGuard<'_, u8> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
