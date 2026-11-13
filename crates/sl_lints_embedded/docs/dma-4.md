## Why

A DMA ring-buffer consumer (UART RX ring, a sample ring) with no `cnt!` counter incremented on the overrun path — the case where the producer wraps past data the consumer hasn't read yet.

An overrun silently drops data; without a counter there is nothing in the build-info counters block to show it ever happened, so the first sign of trouble is a user noticing missing data days later with no way to tell how often it occurs.

## Example

Typec_dongle FW-26 added `pd_rx_overrun` after the fact, once the missing-data reports made clear the ring had been overrunning unnoticed.

## How to fix

Add a `cnt!` counter on the overrun branch of every ring consumer, named for what it's counting (see `cnt-1` for the same rule generalised to every error branch).

## How it is checked

`src mid`: from the firmware model, every DMA ring consumer's overrun branch (the wrap-detected path) must reach a `cnt!` call.

## Source

Ported from embedded_bedrock's firmware check catalogue, `DMA-4` (`book/checks.md`, detail in `book/checks/dma.md`). Found on: typec_dongle FW-26.
