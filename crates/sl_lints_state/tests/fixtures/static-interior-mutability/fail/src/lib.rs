//! x
use std::sync::{LazyLock, Mutex, RwLock, OnceLock};
use std::collections::HashMap;
static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());
static CACHE: LazyLock<RwLock<HashMap<String, u8>>> = LazyLock::new(Default::default);
static CELL: OnceLock<std::sync::Mutex<u8>> = OnceLock::new();
