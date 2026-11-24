## Why

A waiver that allows nothing is left over from code that changed. It still reads as if the code needed an exception, and it would silently allow a new finding of the same lint that nobody looked at.

A waiver counts as unused only when every lint it names ran in this check, so `--only` or a lint that could not run never makes a waiver look unused. A commit's trailer counts as unused only when every lint it names checked that commit: once the commit lands on the base branch, its waiver is left alone.

## How to fix

Remove the comment.
