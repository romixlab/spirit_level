//! x
use std::cell::RefCell;
thread_local! {
    static DRAFTS: RefCell<Vec<u8>> = RefCell::new(Vec::new());
}
