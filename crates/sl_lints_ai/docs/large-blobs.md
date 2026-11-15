## Why

A large file in git stays in every clone and every fetch forever, even after it is deleted: a database dump, a
build artifact, a dependency vendored by mistake, a video pasted into a commit. Unlike a working tree, history
cannot shrink without rewriting it.

The rule lists the tracked files (`git ls-files`) over `max-bytes` that match no glob in `allow`.

## How to fix

Remove the file from the working tree and keep it out of git (download it in a build step, generate it, or
replace it with a smaller fixture). A file that belongs in git on purpose (a release asset, a seeded corpus) gets
its glob added to `allow` with a reason.
