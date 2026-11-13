//! Error enums that are small or structured, and big enums that are no errors.

pub enum SmallError {
    A,
    B,
}

pub enum ConfigError {
    Missing,
    Unreadable,
}

/// Big, but grouped: one variant wraps the config layer's error
pub enum SyncError {
    Config(ConfigError),
    PeerTimeout,
    PeerRefused,
    Disk,
    Busy,
}

/// Big, but wraps an io error
pub enum StoreError {
    Io(std::io::Error),
    A,
    B,
    C,
    D,
}

/// Big, and no error
pub enum Colour {
    Red,
    Green,
    Blue,
    Cyan,
    Magenta,
}

#[cfg(test)]
mod tests {
    pub enum TestError {
        A,
        B,
        C,
        D,
        E,
    }
}
