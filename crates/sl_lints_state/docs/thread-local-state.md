## Why

`thread_local!` with a `Cell` or `RefCell` is global state in disguise: it works on one thread and silently differs on another, and it hides a side channel between functions that do not pass it. tpm's records store kept drafts in a thread-local side channel that one command filled and another read.

## How to fix

Pass the state down the call tree (a context argument). A per-thread buffer that only its own thread writes and a rare reader dumps is a fine use: say so in a waiver.
