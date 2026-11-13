## Why

Merge leftovers (`.orig`, `.rej`), editor backups (`~`, `.swp`, `.bak`), scratch files and an agent's session hand-off note do not belong in history. Agents create them while working and commit them with `git add -A`.

The rule lists the tracked files (`git ls-files`) that match one of `patterns`.

## How to fix

`git rm` the file and add its pattern to `.gitignore`.
