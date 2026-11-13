# sl_model

The project a run looks at, read once and shared by every analysis:

- the workspace from `cargo metadata --no-deps` (members, their dependencies as declared, targets, features), and the resolved graph from a full `cargo metadata` on first use;
- every manifest as a `toml_edit` document, so a finding can point at the line of a key, and an exact fix can edit the text around it (inheritance with `.workspace = true` and `[lints]` are not in cargo metadata);
- every Rust source file of each member, parsed with `syn` on first use, with its module path and whether it is test code;
- line counts (non-blank, non-comment lines).

It depends on `sl_core` only among the workspace crates. Nothing here judges the project: that is the analyses' job.
