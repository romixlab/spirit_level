# sl_report

Turns an `sl_engine::Report` into output:

- `text`: for people, findings grouped by lint (or crate, or file), denied groups first, each by count: a header with level, code, id, count and rule, a source excerpt drawn with annotate-snippets (the file is read at print time), up to 5 places and the command that lists the rest; then the lints that did not run, then a summary line. Styled with `vc_ui::style`; the caller writes it through `anstream`, which strips the colour when it is off.
- `json`: the report as one JSON document (`"schema": "spirit_level.report/1"`), for CI and agents.
- `config_errors`: configuration errors in the same style.
