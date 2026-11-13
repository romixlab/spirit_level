## Why

A WireWeaver (or similar) request-handler function that itself awaits a bus transfer (I²C, SPI, a slow external chip) instead of handing the request to the task that owns the bus and answering once that task reports back.

The server loop processes one request at a time; a handler that blocks on a bus transfer stalls every other request on the same server for the transfer's duration, including requests that have nothing to do with that bus.

## Example

Usb_io_fw's I²C handlers originally ran their transfers inline in the server loop (IF-2); every I²C request serialised the whole device API behind the bus's own timing.

## How to fix

Give the bus its own task; the handler sends the request over a channel and awaits the answer, so other handlers keep running while the transfer is in flight.

## How it is checked

From the firmware model: a request handler body that awaits a bus driver call directly, rather than sending to a channel owned by a bus task and awaiting the reply channel.

## Source

Ported from embedded_bedrock's firmware check catalogue, `EXE-8` (`book/checks.md`, detail in `book/checks/exe.md`). Found on: usb_io_fw IF-2 (I2C handlers ran transfers inline).
