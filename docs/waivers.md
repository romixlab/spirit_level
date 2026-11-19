# Waivers

A waiver allows one finding where it is, in the project's own file, with the reason next to the code.

```rust
// spirit-level: allow(static-interior-mutability) the signal handler's flag; one writer, read on exit
static STOP: AtomicBool = AtomicBool::new(false);

let n = digits.parse::<u32>().unwrap(); // spirit-level: allow(panic-in-library) the lexer only yields digits
```

```toml
# spirit-level: allow(default-features-off) the default TLS backend is rustls here
reqwest = "0.12"
```

- **Place.** At the end of a line, a waiver covers that line. On a line of its own it covers the next item: in Rust the whole item, field or statement that follows, from its first doc comment or attribute to its end, so a waiver above the docs, above the attributes, between them or directly above the item all apply to a finding at any of its lines; in TOML a table header covers its table.
- **Several lints:** `allow(lint-a, lint-b)`. A code names its lint too: `allow(S0001)`.
- **Expiry:** right after the list, `allow(crate-lines) expires=2027-03-31 split tracked as SPLIT-2`, or a release, `expires=v0.5.0`.
- **Checked.** A waiver without a reason, naming an unknown lint, or expired is reported by [waiver-invalid](lints/C0012.md); one that allows nothing by [waiver-unused](lints/C0013.md). Doc comments (`///`, `//!`) never hold waivers, so documentation can show them.

A commit is waived by a trailer in its message, `Spirit-Level: allow(lint) reason`: see [Commits and landing](commits.md).

For a whole crate or a set of paths, use an [exception](configuration.md) in `spirit_level.toml` instead.
