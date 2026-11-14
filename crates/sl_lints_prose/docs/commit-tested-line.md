## Why

A commit that says how it was tested lets a reviewer weigh it and lets the next reader trust it. An untested change that says so is honest; one that says nothing looks tested. Agents in particular report outcomes as done; a `Tested:` line makes the claim checkable.

```text
Reject frames longer than the buffer

Tested: just test --changed; the new frame_len test fails without the fix.
```

The lint checks each commit of the branch ([Commits and landing](../commits.md)) for a `Tested:` trailer with a value.

## How to fix

End the message with `Tested: <what ran>`: the command, the reproducer, the bench. Nothing ran: `Tested: not tested`, and what could not be done (`needs the board`). On the branch, `git commit --amend` or a rebase rewrites the message.
