## Why

**ES0491 2.22.3** ("DAR mode transmission failure due to lost arbitration", H7), status A:

> In DAR mode, the transmission may fail due to lost arbitration at the first two identifier bits.

Workaround: **"clear the corresponding Tx buffer transmission request bit TRPx of the FDCAN_TXBRP register and set the corresponding cancellation finished bit CFx of the FDCAN_TXBCF register, then restart the transmission."**

## Example

DAR (one-shot transmission, no automatic hardware retransmit) is `bedrock_can`'s `one_shot` API option on vb135. The workaround itself — clear TRPx/CFx and restart from software — is **not implemented**: a frame lost this way under DAR is simply dropped (tracked as `TPL-53` in this repo's FEATURES.md), which is an accepted gap, not a silent one.

## How to fix

Either implement the documented software retransmit for DAR mode, or, if one-shot semantics are only ever used where an occasional dropped frame at lost arbitration is acceptable, say so explicitly next to the `one_shot` option and count the drop.

## How it is checked

Check: src mid · chip: H7 — recognizing "DAR is on and no retransmit path exists" needs the driver's state machine, not a single line.

## Source

Ported from embedded_bedrock's firmware check catalogue, `ERR-5` (`book/checks.md`, detail in `book/checks/err.md`). Found on: vb135 PLAT-6.
