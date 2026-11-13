# spirit_level_cli

The `spirit-level` command, a thin layer over the `spirit_level` library.

```sh
spirit-level check                    # the project here, with its spirit_level.toml
spirit-level check --format json      # for CI and agents
spirit-level check --fix              # apply the exact fixes, then check again
spirit-level check --tier big         # try a tier without writing a config
spirit-level explain crate-lines      # a lint's page: rule, why, parameters, tiers
spirit-level lints                    # every lint with its level per tier
spirit-level tiers                    # the shipped tiers
spirit-level config                   # the resolved levels, overrides and exceptions with reasons and expiries
spirit-level init --tier small        # write a spirit_level.toml
spirit-level book docs                # write the book's generated pages (--check: fail when stale)
```

Exit codes: 0 clean (warnings allowed), 1 a denied finding, 2 a configuration or usage error or a project cargo cannot read, 3 a denied lint could not run (a tool or the crate map is missing).

Shell completion: `source <(COMPLETE=bash spirit-level)` (zsh, fish alike); lint ids and tier names complete with their summaries.
