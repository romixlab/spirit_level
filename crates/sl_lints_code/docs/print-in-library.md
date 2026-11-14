## Why

Whether and where to print is the binary's decision. A library that prints writes into a GUI's terminal, a JSON stream or an agent's tool output, and cannot be silenced. `dbg!` left in code is debugging that escaped.

## How to fix

Return the text, or log through `log` or `tracing`; the binary sets the output.

## Clippy

Clippy's `print_stdout` and `print_stderr` (in the medium, big and ai tiers' [Clippy set](M0025.md)) report every print, in binaries too, unless allowed there. This lint stays on beside them: it tells a library from a binary by itself and runs without building the code.
