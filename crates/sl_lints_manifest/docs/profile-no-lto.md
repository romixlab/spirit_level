## Why

Link-time optimisation runs over the whole program at every link, single-threaded for `fat`, and an incremental build cannot reuse it. In the profiles a developer or an agent builds again and again (dev, test, and the deploy profile used to install on the developer's own machines) it turns a link of a second into minutes. Materialize cut CI from 23 minutes to 1.5 by taking LTO out of its test builds. The lint follows `inherits`: a deploy profile that inherits `release` also inherits its `lto`.

## How to fix

```toml
[profile.deploy]
inherits = "release"
incremental = true
lto = false
```

Keep LTO, if anywhere, in `release` or a profile of its own for the artifacts that ship. Firmware is the exception: there LTO is part of fitting the image into flash, and the `embedded` tier does not enable this lint. The profiles checked are the parameter `profiles`.
