use super::c::{deep, deeper};

pub fn middle() -> u32 {
    deep() + deeper()
}
