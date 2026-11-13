## Why

Work commits put their entries under `## [Unreleased]`; the release step turns that section into `## [x.y.z] - date`. Without the section there is nowhere for a branch to write, and entries end up in the last release by mistake.

## How to fix

Add `## [Unreleased]` above the newest version section.
