## Why

A fix that names the commit which brought the bug tells which releases carry the bug, lets tools follow a bug back to its origin, and shows which changes needed fixing later. The name only helps when it is right: a hash too short to stay unique, a hash of another repository, or a subject copied wrong points nowhere.

```text
Reject frames longer than the buffer

Fixes: 3f2a9c1d0b7e ("Read frames into a fixed buffer")
```

The lint checks each `Fixes:` trailer of the branch's commits ([Commits and landing](../commits.md)): the hash has 12 or more hex digits, it resolves to a commit in the history of the fix, and the quoted subject is that commit's subject.

## How to fix

Write the trailer as `git log -1 --format='%h ("%s")' --abbrev=12 <hash>` prints it, in the last paragraph of the message. On the branch, `git commit --amend` or a rebase rewrites the message.
