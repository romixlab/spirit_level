//! Fixture.

pub fn parse_header(_data: &[u8]) {}
pub fn checksum(_data: &[u8]) {}
pub fn leaked(_data: &[u8]) {}

#[cfg(test)]
mod tests {
    #[test]
    fn replays_parse_header_corpus() {}

    #[test]
    fn replays_leaked_corpus() {}
}
