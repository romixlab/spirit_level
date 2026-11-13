## Why

Taking a second lock while holding one creates a lock order that every other path must follow; one path that takes them the other way deadlocks. tpm's turn engine split its state over 16 mutexes and took several at once.

## How to fix

Put both pieces of state under one owner, or take the locks one after the other with the first guard dropped in between.
