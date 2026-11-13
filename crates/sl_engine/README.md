# sl_engine

Runs a check. Given the project model, the resolved configuration and the registered analyses, it:

1. selects the lints the configuration enables (and `--only` keeps);
2. reports a lint whose needs are missing (a resolved graph, the crate map, a tool, nightly) as not run, with the reason;
3. runs each analysis that has a runnable lint;
4. sets each finding's level from the configuration and its scoped exceptions;
5. applies the source waivers (`// spirit-level: allow(lint) reason`), and reports waivers that are malformed, expired or unused through its own lints `waiver-invalid` and `waiver-unused`;
6. orders the findings and returns a `Report`.

`fix` applies the exact fixes of a report's findings to the files. Output formatting lives in `sl_report`.
