## Why

The subject is the one line that `git log --oneline`, review tools and release notes show. Aim for about 60 characters; 72 is the limit, and a subject that long is usually a body in disguise.

The commits checked are `base..HEAD` without merges, read with git. When `base` is not a commit in the repo, a candidate finding says so instead.

```text
commit subject of 82 characters (limit 72): wip: core types, config, model, engine, report, manifest lints
```

## How to fix

Name the change in the subject; move details into the body. Reword with `git commit --amend` or an interactive rebase on an unpushed branch.
