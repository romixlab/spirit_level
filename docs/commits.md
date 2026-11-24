# Commits and landing

Most lints read the files of the working tree. The lints of the `commits` category read the branch's commits instead, and a few lints judge a run that lands a branch more strictly than one in the middle of the work.

## Which commits

A commit lint checks the commits in `base..HEAD`, merges left out: the work of the branch, as `just test --changed` sees it. `base` is a parameter of each commit lint, `main` by default; a project whose work merges into another branch sets it:

```toml
[lints.fixes-tag]
base = "origin/main"
```

History that cannot be rewritten stays out with `after`, a commit or tag: only the commits after it are checked. A repo that adopts a commit lint, or tightens its limit, on a branch whose earlier commits are already pushed names the last of them:

```toml
[lints.commit-subject-length]
after = "v0.5.0"
```

On the base branch itself the range is empty and the commit lints find nothing. When `base` or `after` is not a commit in the repository, the lint reports one candidate finding that names it and fails nothing.

A finding about a commit is placed at `commit:<12 hex>`, with the line of its message: the subject is line 1.

## Where they run

`spl check` runs the commit lints with every other enabled lint, so the same run checks a branch while it is in work and at its landing. The landing gate runs `spl check` on the branch before it merges it, with the base it merges into.

## Waiving a commit

A commit cannot hold a comment, so its waiver is a trailer in its message, written like a [source waiver](waivers.md):

```text
Fix the frame length check

Spirit-Level: allow(fix-without-test) the reproducer needs the board on the bench
```

The trailer covers the whole commit. [waiver-invalid](lints/C0012.md) and [waiver-unused](lints/C0013.md) check it as they check a comment. A commit already on the base branch is never checked again, so its waiver is needed, and judged, only while the commit is on the branch.

## A landing run

`spl check --landing`, or `SPIRIT_LEVEL_LANDING=1` in the environment, marks a run as the landing of a branch. spirit_level does not know the tool that lands: the gate passes the flag.

Lints that allow something while a branch is in work report it as a candidate in an ordinary run, and as a finding at its level in a landing run. [fixme-before-land](lints/A0011.md) is one: a `FIXME` is a note to oneself on the branch, and must be gone before the branch lands.
