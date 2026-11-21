## Why

Agent transcripts and histories hold the whole session: private context, pasted secrets, paths of the author's machine. Per-person files (`.claude/settings.local.json`, `CLAUDE.local.md`) are meant to stay local. Neither belongs in the repo.

The rule lists the tracked files that match one of `patterns`: places where agents write sessions, and `.jsonl` transcripts and histories. A source or docs file named after transcripts is not one.

## How to fix

`git rm --cached` the file, add the pattern to `.gitignore`, and check the history for secrets.
