# spirit_level_cli

The `spl` command, a thin layer over the `spirit_level` library.

```sh
spl check                    # the project here, with its spirit_level.toml
spl check --format json      # for CI and agents
spl check --fix              # apply the exact fixes, then check again
spl check --tier big         # try a tier without writing a config
spl explain crate-lines      # a lint's page: rule, why, parameters, tiers
spl lints                    # every lint with its level per tier
spl tiers                    # the shipped tiers
spl config                   # the resolved levels, overrides and exceptions with reasons and expiries
spl init --tier small        # write a spirit_level.toml
spl book docs                # write the book's generated pages (--check: fail when stale)
spl book stamp a.html a.js   # write the footer build stamp and the reader-clock age script
spl dependents               # the dependent repos and crates to retest after this change ([dependents] turns it on)
spl pages                    # reading metrics per Markdown page: words, reading time, longest run, headings
```

Exit codes: 0 clean (warnings allowed), 1 a denied finding, 2 a configuration or usage error or a project cargo cannot read, 3 a denied lint could not run (a tool or the crate map is missing).

Shell completion: `source <(COMPLETE=bash spl)` (zsh, fish alike); lint ids and tier names complete with their summaries, `--base` with branches and tags.
