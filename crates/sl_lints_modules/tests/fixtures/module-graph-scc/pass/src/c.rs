pub struct Thing(pub u32);

impl Thing {
    pub fn new() -> Thing {
        Thing(1)
    }
}

pub fn deep() -> u32 {
    2
}

#[cfg(test)]
mod tests {
    #[test]
    fn back_up() {
        // test code may reach anywhere
        assert_eq!(crate::a::start(), 3);
    }
}
