## Why

A function marked `async fn` whose body never reaches an `.await` — common in generated request-handler dispatch tables where most handlers are simple and synchronous but the trait requires `async fn`.

Every `async fn`, awaiting or not, becomes a state machine in the embassy dispatcher future and costs flash for its state and poll logic; on a flash-constrained part that adds up across a large handler table even though none of these functions ever actually suspend.

## Example

A fw-size measurement found 115 of 127 generated handlers never awaited, and making them plain synchronous functions (called from the one `async fn` that does the dispatch) saved 17.4 KiB at `opt-level = "s"`.

## How to fix

Make the handler a plain `fn` and call it synchronously from the dispatcher; only the dispatcher itself needs to be `async`.

## How it is checked

`src easy`: syntactically walk each `async fn` body for the presence of any `.await` expression; flag the ones with none.

## Source

Ported from embedded_bedrock's firmware check catalogue, `EXE-12` (`book/checks.md`, detail in `book/checks/exe.md`). Found on: fw-size measurement: 115 of 127 handlers, -17.4 KiB at z.
