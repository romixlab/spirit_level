//! Fixture.

use std::io::Read;

/// The largest reply
const MAX_REPLY_BYTES: u64 = 1 << 20;

/// Reads a reply
pub fn reply(stream: &mut std::net::TcpStream) -> std::io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    stream.take(MAX_REPLY_BYTES).read_to_end(&mut buf)?;
    Ok(buf)
}
