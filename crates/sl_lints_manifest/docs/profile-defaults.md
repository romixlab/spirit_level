## Why

Measured on real projects: `split-debuginfo = "unpacked"` in the dev and test profiles took an edit-then-test build from 141 s to 11 s (test binaries stopped being I/O-bound), and an incremental deploy profile took a one-file deploy from 43 s to 15 s.

## How to fix

```toml
[profile.dev]
split-debuginfo = "unpacked"

[profile.test]
split-debuginfo = "unpacked"

[profile.deploy]
inherits = "release"
incremental = true
```

`just install` builds with `--profile deploy`. The deploy profile's name is the parameter `deploy-profile`.
