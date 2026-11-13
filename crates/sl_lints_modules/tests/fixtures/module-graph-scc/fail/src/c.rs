pub fn deep() -> u32 {
    3
}

pub fn deeper() -> u32 {
    // the one reference that closes the ring
    format!("{}", crate::a::helper()).len() as u32
}
