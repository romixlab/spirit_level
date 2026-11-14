## Why

git shows a moved file as a move only while its content stays nearly the same. A move edited in the same commit shows as a deletion and a new file: the history of every line is cut, and a reviewer cannot tell the move from the change. Moving in one commit and editing in the next keeps both readable.

The lint looks at each file of `files` (Rust sources) that git finds renamed in a commit of the branch with changes. Changed lines that only follow the move (`use`, `mod`, `#[path]`, lines naming `crate::`, `super::` or `self::` paths) are allowed; any other changed line makes a finding. The line judgement is an estimate.

## How to fix

Split the commit: first the move with the path changes it needs, then the edit. On the branch, an interactive rebase or a reset and two commits does it.
