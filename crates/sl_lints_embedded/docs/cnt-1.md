## Why

An `Err` arm that's handled (not just propagated), a timeout, a retry, or a dropped frame or overrun, with no `cnt!` counter incremented on that path.

This is the generalised version of `dma-4`'s specific case: any branch that represents something unexpected happening (not the firmware's normal path) needs to leave a trace somewhere a build-info read can see, or the only way to learn it's happening at all is a user noticing the downstream symptom and reporting it — after which there's no history of how often it occurred.

## Example

The template's own conventions state this directly ("`cnt` counters for unexpected events"), and typec_dongle's `pd_rx_overrun` (FW-62) is the canonical case of a counter added only after the fact, once missing data was already being reported.

## How to fix

Add a `cnt!` counter, named for what it's counting, on every branch that represents something going wrong rather than a normal outcome; follow `dma-4`'s ring-overrun pattern as the template.

## How it is checked

`src hard`: needs to tell an *unexpected*-event error branch (worth counting) from ordinary, expected control flow (not worth counting on every single fallible call) — genuinely needs the firmware model and some judgement, hence the `hard` tier.

## Source

Ported from embedded_bedrock's firmware check catalogue, `CNT-1` (`book/checks.md`, detail in `book/checks/cnt.md`). Found on: template conventions; typec_dongle FW-62 (pd_rx_overrun added after).
