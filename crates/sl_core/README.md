# sl_core

The types every other spirit_level crate shares: a lint's metadata (`LintMeta`: id, category, parameters, what it needs, whether its fix is exact), levels, parameter values, diagnostics with spans and metrics, exact fixes as text edits, dates and expiries, and the small glob matcher used for path scopes.

It holds types only: no I/O, no parsing of projects. It depends on serde and nothing else; every crate of the workspace may depend on it.
