//! x
use std::collections::{BTreeMap, HashMap};
pub fn f() -> BTreeMap<u8, u8> { BTreeMap::new() }
pub type Fixed = HashMap<u8, u8, std::hash::BuildHasherDefault<std::collections::hash_map::DefaultHasher>>;
