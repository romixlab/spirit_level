# sl_report

Turns an `sl_engine::Report` into output:

- `text`: for people, one block per finding (level, lint id, place, message, help, metric), then the lints that did not run with the reason, then a summary line. Styled with `anstyle`; the caller writes it through `anstream`, which strips the colour when the output is not a terminal or `NO_COLOR` is set.
- `json`: the report as one JSON document (`"schema": "spirit_level.report/1"`), for CI and agents.
- `config_errors`: configuration errors in the same style.
