## Why

How a git dependency is pinned follows the stage of the repos on both ends. Cargo.lock fixes the commit whatever `Cargo.toml` says, so a pin in `Cargo.toml` decides only how a dependency moves on: with `cargo update -p NAME`, or with an edit by hand. The project states its policy once, and the lint holds every git dependency to it.

| Policy | A git dependency names | Fits when |
|---|---|---|
| `branch` (default) | a branch or nothing; no `rev` or `tag` | development is early: a break in the other repo shows at once, Cargo.lock holds the commit, no SHA is edited by hand |
| `default-branch` | nothing: no `branch`, `rev` or `tag` | the other repo's default branch is the one to follow, and a branch name in `Cargo.toml` would only go stale |
| `dev-branch` | `branch = "<dev-branch>"`, no `rev` or `tag` | the other repo works on a dev branch and keeps main quiet for its outside users; often makes tags unnecessary |
| `tag` | a `tag`, no `rev` or `branch` | the other repo has outside users and releases by tag; active work happens on a branch or behind a `[patch]` override kept out of git |
| `release` | nothing from git: a registry version | the other repo publishes to crates.io and the project takes its releases |
| `any` | anything | the lint should not check pins |

`policy` applies to other projects' repos and `own-policy` to the project's own, named by URL prefix in `own` (a host, or a host and an org). The two often differ: the project's own libraries move with it on a branch while outside crates come by tag or release. URLs compare without scheme, `git@`, `.git` and case, so `git@github.com:acme/x.git` is under `github.com/acme`.

## Example

A product whose own libraries follow their default branch, and whose outside git dependencies come by tag:

```toml
[lints.git-dependency-pins]
level = "deny"
reason = "own libraries move with us; outside ones by release tag"
own = ["github.com/acme", "git.acme.dev"]
own-policy = "default-branch"
policy = "tag"
```

## How to fix

Write the dependency the way the policy says: drop the `rev`, `tag` or `branch` it does not allow, add the one it asks for, or take a registry release. Then `cargo update -p NAME` moves Cargo.lock to it. When the stage of a repo changes, change the policy, not each dependency's exception.
