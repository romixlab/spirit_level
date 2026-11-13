## Why

Whether and where to print is the binary's decision. A library that prints writes into a GUI's terminal, a JSON stream or an agent's tool output, and cannot be silenced. `dbg!` left in code is debugging that escaped.

## How to fix

Return the text, or log through `log` or `tracing`; the binary sets the output.
