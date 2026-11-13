## Why

`.cargo/config.toml` relying on `defmt-rtt`'s default buffer size and `DEFMT_LOG`'s default level instead of setting both explicitly.

The default buffer is small enough that a burst right at boot (every "I'm alive" log a firmware tends to emit during init) can overrun it before the host even attaches; a real measurement found 773 of 1000 init-time log lines lost to exactly this.

## Example

The template's own conventions call this out, and an internal log entry (Oct 2026) has the 773/1000 measurement that motivated making both settings explicit rather than left at their defaults.

## How to fix

Set `DEFMT_RTT_BUFFER_SIZE` large enough to absorb an init burst before the host attaches, and `DEFMT_LOG` to the level the firmware actually wants shipped, both explicitly.

## How it is checked

`cfg easy`: both `DEFMT_RTT_BUFFER_SIZE` and `DEFMT_LOG` must be set as explicit env vars in `.cargo/config.toml`, not left unset.

## Source

Ported from embedded_bedrock's firmware check catalogue, `LOG-2` (`book/checks.md`, detail in `book/checks/log.md`). Found on: template conventions; internal log Oct 2026.
