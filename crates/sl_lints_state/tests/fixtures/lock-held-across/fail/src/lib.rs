//! x
use std::sync::Mutex;
pub async fn bad(m: &Mutex<Vec<u8>>, tx: &Tx) {
    let g = m.lock().unwrap();
    tx.send(g.clone()).await;
}
pub fn io(m: &Mutex<u8>) -> std::io::Result<()> {
    let g = m.lock().unwrap();
    std::fs::write("state", [*g])
}
pub struct Tx;
impl Tx { pub async fn send(&self, _v: Vec<u8>) {} }
