//! fx_dylib is pulled in only to force fx_internal to link as a shared library; nothing calls it.

#[cfg(feature = "dynamic_linking")]
#[allow(unused_imports)]
use fx_dylib;

pub use fx_internal::hi;
