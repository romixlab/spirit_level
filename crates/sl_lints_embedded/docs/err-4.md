## Why

**ES0548 2.15.1 / ES0491 2.22.1** ("Desynchronization under specific condition with edge filtering enabled", G0 / H7), status **A** (workaround available):

> FDCAN may desynchronize and incorrectly receive the first bit of the frame if the edge filtering is enabled
> (the EFBI bit of the FDCAN_CCCR register is set), and the end of the integration phase coincides with a falling
> edge detected on the FDCAN_Rx input pin.

Workaround: **"Disable edge filtering or wait for frame retransmission."**

**ES0548 2.15.2 / ES0491 2.22.2** ("Tx FIFO messages inverted under specific buffer usage and priority setting"), same status, triggers only when "FDCAN uses both a dedicated Tx buffer and a Tx FIFO (the TFQM bit of the FDCAN_TXBC register is cleared)"; the sheets' second workaround is **"Use only a Tx FIFO."**

## Example

Vb135's FDCAN bring-up (`CAN-1`, `bedrock_can`) takes both workarounds as the only configuration it ever builds: edge filtering (`CCCR.EFBI`) stays off, and the TX side is always Tx-FIFO-only (`TFQM` set), never a dedicated buffer beside a FIFO — there's no runtime switch for either, so the erratum can't be re-introduced by a later config change.

## How to fix

Don't expose edge filtering or "dedicated buffer + FIFO" as configuration options at all on G0/H7 FDCAN; build the driver so the only reachable configuration is already the workaround.

## How it is checked

Check: src mid · chip: G0, H7 — needs reading the configuration path to confirm `EFBI` and `TFQM` can't be set to the erratum-triggering combination, not a single register value.

## Source

Ported from embedded_bedrock's firmware check catalogue, `ERR-4` (`book/checks.md`, detail in `book/checks/err.md`). Found on: template errata.md; vb135 PLAT-6.
