//! Flat error enums.

pub enum SyncError {
    ConfigMissing,
    ConfigUnreadable(String),
    PeerTimeout,
    PeerRefused { peer: String },
    Disk(u32),
}

/// Not named like an error, but it is one
#[derive(Debug)]
pub enum Failure {
    A,
    B,
    C,
    D,
    E,
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failure")
    }
}

impl std::error::Error for Failure {}
