//! Fixture.

/// Holds the line low for a temporary 2 ms reset pulse; a temporary file is written beside the output.
pub fn reset() {
    // TODO(CFG-3): include keys
    // workaround for the erratum, see https://example.org/errata#2
    // a todo list is not a marker; hackathon and shimmer are other words
    let s = "TODO: in a string";
    let _ = s;
}

// The workspace-hack package builds each shared dependency once.
pub fn shared() {}
