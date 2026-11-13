## Why

A rules file that states a limit the lints check differently, or do not check at all, teaches agents a rule the repo does not hold. The text and the configuration must say the same.

The rule reads each line of `files` for a number with a unit, given to the nearest subject before it on the line:

| Subject | Unit | Lint and parameter |
|---|---|---|
| commit subject | chars, characters | `commit-subject-length.max-chars` |
| commit body | lines | `commit-body-lines.max-lines` |
| CHANGELOG entry | lines | `changelog-entry-length.max-lines` |
| CHANGELOG entry | words | `changelog-entry-length.max-words` |
| AGENTS.md, this file | lines | `agent-file-lines.max-lines` |
| sentence | words | `sentence-words.max-words` |
| paragraph | sentences | `paragraph-sentences.max-sentences` |

A limit is reported when its lint is off, or when the lint's parameter differs.

```markdown
Limits: CHANGELOG entry 1 line and 20 words, commit subject 60 characters, this file 150 lines.
```

## How to fix

Set the lint's parameter to the stated value in `spirit_level.toml`, or change the text to what the lint checks.
