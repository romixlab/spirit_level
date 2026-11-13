fn t(m: &std::sync::Mutex<u8>) -> std::io::Result<()> {
    let g = m.lock().unwrap();
    std::fs::write("x", [*g])
}
