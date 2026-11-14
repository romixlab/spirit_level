## Why

A contract crate holds the types that cross a process or node boundary, and every party links it. Parties are built and deployed at different times, so each must know which version of the contract it speaks and what that version looks like. The version constant answers the first; the schema snapshot of each version, kept in git, answers the second and lets a test check that a reader still reads what an older writer wrote. A snapshot that changes after it reached the base branch breaks that promise silently: a party built against it now reads something else under the same number.

The crates are the ones the [crate map](../map.md) marks `contract = true`.

```text
crates/foo_wire/
  src/lib.rs          pub const VERSION: u32 = 2;
  schema/v1.json      the schema of version 1, unchanged since it landed
  schema/v2.json      the schema of version 2
```

## How to fix

- No version: add `pub const VERSION: u32 = 1;` to the library root (parameter `const` names it).
- No snapshot: write the schema of the current version to `schema/v<N>.json` (parameter `snapshot`) and commit it. A test that generates the schema from the types and compares it with the file keeps it current.
- Not in git: commit the snapshot.
- A snapshot changed against the base (parameter `base`): restore it, bump `VERSION`, and write the new schema to the next version's file. Readers tolerate unknown fields, so adding a field is a new version, not an edit.
