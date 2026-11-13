//! x
#[allow(dead_code, reason = "used by the macro below")]
fn a() {}
#[expect(clippy::too_many_arguments, reason = "mirrors the C API")]
pub fn b() {}
/// #[allow(x)] in a doc comment is not an attribute
pub fn c() {}
#[cfg_attr(not(test), allow(dead_code, reason = "used by the tests only"))]
fn d() {}
#[cfg_attr(feature = "x", derive(Debug))]
struct T;
