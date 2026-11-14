## Why

rustc compiles the files that `mod` declarations reach from a crate root and ignores every other file. A file left under `src/` after a rename or a lost `mod` line is never built, never tested and never warned about, yet it reads like live code and agents edit it.

The lint follows `mod` declarations from each target root, `#[path]` and test-only modules included, and reports each `.rs` file under `src/` that none reaches. A file pulled in with `include!` counts as reached, and so does one whose name appears in a macro call of the module file that would declare it (a macro that writes the `mod` lines).

## How to fix

Declare the module where it belongs (`mod name;`), or delete the file. A file compiled another way (by a build script, a macro) goes in the `skip` list with a reason.
