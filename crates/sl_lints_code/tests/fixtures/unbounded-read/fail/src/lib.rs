//! Fixture.

use std::io::Read;

/// Reads a reply
pub fn reply(stream: &mut std::net::TcpStream) -> std::io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf)?;
    let mut s = String::new();
    (&*stream).read_to_string(&mut s)?;
    Ok(buf)
}
