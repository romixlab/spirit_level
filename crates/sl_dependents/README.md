# sl_dependents

The repos that build on a library, and what to retest in them when it changes (`spl dependents`, layer model):

- `changed`: the library's crates changed against a base revision (git), with the crates that depend on them.
- `lock`: a dependent's `Cargo.lock` as a graph, read without cargo or the network.
- `find`: every checkout the `[dependents]` table names, its crates that reach a changed crate, the retest command, and the library commit its lock holds.
