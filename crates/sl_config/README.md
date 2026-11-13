# sl_config

Reads `spirit_level.toml` and the tier files it extends into one resolved configuration: for every lint its level, parameters, reason and where that came from; the scoped exceptions; the crate map.

The format is strict. An unknown key, an unknown lint id, an unknown parameter or one of the wrong type, an entry without a reason, and an override or exception whose expiry has passed are all errors with the file and line, and the run stops before any lint runs.

It depends on `sl_core` (types) and `sl_model` (for `TomlFile` and line lookups). The format itself is documented in the book (`docs/configuration.md`).
