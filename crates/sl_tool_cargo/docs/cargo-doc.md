## Why

A broken intra-doc link, an unclosed HTML tag or a bare URL in a doc comment renders wrong on docs.rs and in the IDE. `cargo doc` warns about them; this lint reports every warning, as `RUSTDOCFLAGS="-D warnings"` in CI would.

## How to fix

Fix the doc comment the warning points at: link with the path rustdoc can resolve (`` [`crate::Foo`] ``), escape brackets that are not links.
