## Why

A CHANGELOG is read as a list: what changed in this release, one line each. A long entry explains the implementation, which the book and the commits already hold, and hides the change among the words.

The rule counts the lines of each bullet under `## [Unreleased]` (continuation lines included) and its words (runs of letters and digits). Released sections are left alone unless `all-sections` is on.

```markdown
- Config: `include` key reads another file (CFG-6).                        <- 7 words, passes

- Added a new configuration key called include which makes it possible to
  read another TOML file from the main one, merged before overrides (CFG-6). <- 2 lines, 27 words
```

## How to fix

Name the change and its FEATURES id in one line. Move the explanation into the book page of the feature.
