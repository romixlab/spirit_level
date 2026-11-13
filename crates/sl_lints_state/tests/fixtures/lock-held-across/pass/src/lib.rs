//! x
use std::sync::Mutex;
pub async fn ok(m: &Mutex<Vec<u8>>, tx: &Tx) {
    let v = {
        let g = m.lock().unwrap();
        g.clone()
    };
    tx.send(v).await;
}
pub async fn dropped(m: &Mutex<u8>, tx: &Tx) {
    let g = m.lock().unwrap();
    let n = *g;
    drop(g);
    tx.send(vec![n]).await;
}
pub fn later(m: &Mutex<u8>) -> impl std::future::Future<Output = ()> + '_ {
    let g = m.lock().unwrap();
    let n = *g;
    async move { let _ = n; }
}
pub fn copied(m: &Mutex<u8>) -> std::io::Result<()> {
    let n = *m.lock().unwrap();
    std::fs::write("x", [n])
}
pub struct Tx;
impl Tx { pub async fn send(&self, _v: Vec<u8>) {} }
#[cfg(test)]
#[path = "held_tests.rs"]
mod tests;
#[cfg(test)]
mod more;
