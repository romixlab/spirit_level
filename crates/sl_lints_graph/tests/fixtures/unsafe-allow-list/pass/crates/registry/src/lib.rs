//! x
#[linkme::distributed_slice]
pub static COMMANDS: [&'static str];

#[expect(unsafe_code, reason = "linkme places the entry with link_section")]
#[linkme::distributed_slice(COMMANDS)]
static HELLO: &str = "hello";
