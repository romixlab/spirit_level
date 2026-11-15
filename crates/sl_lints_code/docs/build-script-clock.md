## Why

A `build.rs` that reads the wall clock (`SystemTime::now()`, `chrono::Utc::now()`, `chrono::Local::now()`) bakes a different value into every build, so cargo never sees a cached build and nothing downstream can reproduce it from the same source. A `build.rs` that assumes `.git` is a directory breaks the moment it is built from a git worktree, where `.git` is a file pointing at the real git dir elsewhere (`.git/worktrees/<name>`); the script either fails to find `HEAD` or watches the wrong file, and `cargo:rerun-if-changed` stops catching new commits.

## Example

```rust
// bad: reads the clock, and assumes .git is the real git dir
let now = std::time::SystemTime::now();
println!("cargo:rerun-if-changed=.git/HEAD");
println!("cargo:rerun-if-changed=.git/index");
```

```rust
// good: asks git for the real dir, which works in a worktree too
let git_dir = std::process::Command::new("git")
    .args(["rev-parse", "--git-dir"])
    .output()
    .ok()
    .filter(|o| o.status.success())
    .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned());
if let Some(dir) = git_dir {
    println!("cargo:rerun-if-changed={dir}/HEAD");
    println!("cargo:rerun-if-changed={dir}/index");
}
```

## How to fix

Drop the clock read; a build time nobody can reproduce is worse than none. Read the git commit time instead (through the same `git rev-parse --git-dir` call), or leave the field unset until something that wants a timestamp asks for one at run time.

Resolve the git dir with `git rev-parse --git-dir`, run from the build script's own directory (`CARGO_MANIFEST_DIR`), and watch `HEAD` and `index` under that resolved path. The lint is an estimate: it looks for a `.git` path component and flags it unless the file shows evidence (`git-dir`, `git-common-dir`) that it asked git first.
