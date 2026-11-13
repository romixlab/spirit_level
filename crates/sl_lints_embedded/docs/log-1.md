## Why

An RTT channel left at its default mode, where probe-rs attaching sets the channel to blocking — so a log call waits for the host to drain the buffer, and blocks forever if nothing is attached or the host stalls.

Logging is supposed to be a side channel, not a dependency; a blocking RTT channel turns "nobody attached a probe" or "the host debugger hung" into the firmware itself hanging at its next log call, which is a surprising way to lose a device that has nothing wrong with it otherwise.

## Example

Typec_dongle FW-27, wire_weaver's STD-26, and embedded_bedrock ROB-8 all converge on the same fix: force the channel's mode explicitly rather than trust the default.

## How to fix

Configure the RTT channel as `NoBlockSkip`: drop whole frames when the buffer is full and count the drops, rather than ever waiting for the host.

## How it is checked

`src mid`: the RTT channel's mode must be explicitly set to `NoBlockSkip` (or an equivalent non-blocking mode), not left at whatever `rtt-target`/`defmt-rtt` defaults to.

## Source

Ported from embedded_bedrock's firmware check catalogue, `LOG-1` (`book/checks.md`, detail in `book/checks/log.md`). Found on: typec_dongle FW-27; wire_weaver STD-26; embedded_bedrock ROB-8.
