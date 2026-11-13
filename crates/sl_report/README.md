# sl_report

Turns an `sl_engine::Report` into output:

- `text`: for people, findings grouped by lint (or crate, or file), denied groups first, each by count: a header with level, code, id, count and rule, a source excerpt drawn with annotate-snippets (the file is read at print time), up to 5 places and the command that lists the rest; then the lints that did not run, then a summary line. Coloured through the `vc_ui::Paint` in its options (`Paint::stdout()` decides by `--color`, the terminal and `NO_COLOR`); the excerpts follow it.
- `json`: the report as one JSON document (`"schema": "spirit_level.report/1"`), for CI and agents.
- `config_errors`: configuration errors in the same style.
