## Why

`bedrock_crash::exc_return!()` called anywhere in the `HardFault` handler other than as its very first statement.

`exc_return!()` reads the link register to recover the exception return information the crash record needs; any code that runs before it (even an apparently harmless local variable) can change what's in `LR` by the time it's finally called, corrupting the one piece of context that says what the CPU was doing when the fault happened.

## Example

The template's generated `main.rs` places the macro as literally the first line of the handler for exactly this reason.

## How to fix

Move the call to the top of the handler, before anything else, including seemingly-inert local bindings.

## How it is checked

`src easy`: a syntactic check that `exc_return!()` is the first statement in the function body.

## Source

Ported from embedded_bedrock's firmware check catalogue, `FLT-3` (`book/checks.md`, detail in `book/checks/flt.md`). Found on: template main.rs.
