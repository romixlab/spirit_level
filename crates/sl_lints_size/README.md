# sl_lints_size

Size and complexity lints: a crate's code lines in `src/` (`crate-lines`, the ~20,000-line split point), its test code (`test-lines`), one file (`module-lines`), one function (`function-lines`) and a function's cyclomatic complexity (`function-complexity`). Every limit is a parameter; a crate that may be bigger gets a scoped exception with a reason and an expiry, and crates the map marks `generated` are exempt.

Code lines are lines that are not blank and not only a comment (`sl_model::count_lines`). It depends on `sl_analysis`, `sl_core`, `sl_model` and `syn`.
