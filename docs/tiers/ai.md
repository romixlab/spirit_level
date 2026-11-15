# ai

Code and docs written with AI agents: agent files, terse output, AI-isms, stop-gap smells, agent-config hygiene.

Use it with `extends = "ai"` in `spirit_level.toml`.

## Lints enabled (with what it extends)

| Code | Lint | Level | Reason | Set in |
|---|---|---|---|---|
| [A0001](../lints/A0001.md) | agent-config-permissions | `deny` | committed settings apply to every agent; a blanket grant lets any of them run anything | ai |
| [A0002](../lints/A0002.md) | agent-config-secrets | `deny` | a committed key is public to every reader and stays in history | ai |
| [A0003](../lints/A0003.md) | agent-session-files | `deny` | transcripts and per-person agent files hold private context and secrets | ai |
| [A0004](../lints/A0004.md) | leftover-files | `deny` | merge, editor and scratch leftovers and session notes do not belong in history | ai |
| [A0005](../lints/A0005.md) | commented-out-code | `deny` | git keeps the old code; commented-out code is a decision not taken | ai |
| [A0006](../lints/A0006.md) | dead-features | `warn` | a feature that enables nothing and gates nothing is left over | ai |
| [A0007](../lints/A0007.md) | lint-table-allow-reason | `deny` | a lint allowed for a whole crate in Cargo.toml says why | ai |
| [A0008](../lints/A0008.md) | near-duplicate-fns | `warn` | a rewrite that keeps the old function leaves two copies to fix | ai |
| [A0009](../lints/A0009.md) | near-duplicate-modules | `deny` | a rewrite that keeps the old module is not finished | ai |
| [A0010](../lints/A0010.md) | stop-gap-comments | `deny` | a TODO or a 'for now' marks a path the final design throws away; tracked work names its id | ai |
| [A0011](../lints/A0011.md) | fixme-before-land | `deny` | an agent may leave itself a FIXME on its branch; the landing refuses it | ai |
| [A0012](../lints/A0012.md) | large-blobs | `warn` | a large tracked file stays in every clone forever; agents commit dumps and binaries with git add -A | ai |
| [C0002](../lints/C0002.md) | allow-without-reason | `deny` | an allowed lint says why, so it can be turned back on | ai |
| [C0008](../lints/C0008.md) | todo-macros | `deny` | todo!() and unimplemented!() are stop-gaps that compile | ai |
| [C0018](../lints/C0018.md) | unit-suffix | `warn` | agents write units last in names from the start; a warning, since it reads names only and fires widely in older code | ai |
| [C0019](../lints/C0019.md) | unbounded-read | `warn` | input from outside sets the memory a read to the end takes; candidates only, a place to look | ai |
| [M0018](../lints/M0018.md) | repo-files | `deny` | AGENTS.md holds the rules every agent reads; CLAUDE.md points to it | ai |
| [P0001](../lints/P0001.md) | agent-file-lines | `deny` | agents read the file whole every session; a long one is skimmed, not followed | ai |
| [P0002](../lints/P0002.md) | agent-file-links | `deny` | an agent that follows a dead link guesses or stops | ai |
| [P0003](../lints/P0003.md) | agent-rules-match-lints | `deny` | a stated limit the lints check differently teaches agents a rule the repo does not hold | ai |
| [P0004](../lints/P0004.md) | claude-md-duplicate | `deny` | two copies of the rules drift apart | ai |
| [P0005](../lints/P0005.md) | commit-body-lines | `warn` | a body holds why, not a retelling of the diff | ai |
| [P0006](../lints/P0006.md) | commit-prose | `warn` | commit messages follow the prose rules of the docs | ai |
| [P0007](../lints/P0007.md) | commit-subject-length | `deny` | the subject is what one-line logs and reviews show | ai |
| [P0008](../lints/P0008.md) | changelog-describes-diff | `warn` | an entry says what changed for the user; the diff already shows the files | ai |
| [P0009](../lints/P0009.md) | changelog-entry-length | `deny` | a CHANGELOG is a list: one line per change | ai |
| [P0010](../lints/P0010.md) | emoji | `warn` | emoji do not grep, diff or render the same everywhere | ai |
| [P0011](../lints/P0011.md) | filler-phrases | `deny` | filler adds words and no meaning | ai |
| [P0012](../lints/P0012.md) | narration | `deny` | docs and comments state facts, not the conversation that produced them | ai |
| [P0013](../lints/P0013.md) | paragraph-sentences | `warn` | result first, then what the reader must do; more facts are a list | ai |
| [P0014](../lints/P0014.md) | restating-comments | `warn` | a comment says what the code cannot | ai |
| [P0015](../lints/P0015.md) | sentence-words | `warn` | short sentences are read once and understood | ai |
| [P0016](../lints/P0016.md) | thinking-comments | `deny` | a comment holds the conclusion, not the reasoning that led to it | ai |
| [P0017](../lints/P0017.md) | fixes-tag | `deny` | an agent copies the culprit's hash and subject from git; a wrong one sends the next reader astray | ai |
| [P0018](../lints/P0018.md) | fix-without-test | `warn` | an agent writes the failing test first; a warning, since a fix is judged from its subject | ai |
| [P0019](../lints/P0019.md) | commit-tested-line | `warn` | an agent says how it tested, or that it did not; a warning while branches begun before the rule land | ai |
| [P0020](../lints/P0020.md) | commit-mixed-move | `warn` | a move edited in the same commit cuts every line's history; a warning, since it judges lines by their start | ai |
| [P0021](../lints/P0021.md) | url-tracking-params | `warn` | a link pasted from a browser or a chat app carries an id that tells the reader nothing and goes stale | ai |

## Clippy set (with what it extends)

The project turns these on in its own `[workspace.lints.clippy]` (`allow` rows are accepted opt-outs, pasted with their reason); [tier-clippy-set](../lints/M0025.md) checks it.

| Clippy lint | Level | Reason | Set in |
|---|---|---|---|
| `allow_attributes` | `warn` | #[expect] fails once the lint stops firing, so an exception cannot outlive its cause | ai |
| `cognitive_complexity` | `warn` | agents grow functions branch by branch; a tangled one is split | ai |
| `print_stderr` | `warn` | agents leave debugging prints behind | ai |
| `print_stdout` | `warn` | agents leave debugging prints behind | ai |
| `self_named_module_files` | `warn` | one module layout, mod.rs, across the workspace | ai |
| `str_to_string` | `warn` | to_owned says what happens: a copy of a borrowed str | ai |
| `too_long_first_doc_paragraph` | `warn` | agents write long doc summaries; the first paragraph is one short sentence | ai |
| `use_self` | `warn` | Self follows a rename | ai |

`clippy.toml` (an integer is a limit the project may lower):

| Setting | Value | Reason | Set in |
|---|---|---|---|
| `cognitive-complexity-threshold` | `20` | set on samples: 17 read fine, 21 and 25 were too long; spl's function-complexity (cyclomatic, 15) counts paths | ai |

## The file

```toml
# Beside the size tiers: code and docs written with AI agents. Combine it with one: extends = ["medium", "ai"].
[tier]
description = "Code and docs written with AI agents: agent files, terse output, AI-isms, stop-gap smells, agent-config hygiene"

# Agent files

[lints.repo-files]
level = "deny"
reason = "AGENTS.md holds the rules every agent reads; CLAUDE.md points to it"

[lints.agent-file-lines]
level = "deny"
reason = "agents read the file whole every session; a long one is skimmed, not followed"

[lints.claude-md-duplicate]
level = "deny"
reason = "two copies of the rules drift apart"

[lints.agent-file-links]
level = "deny"
reason = "an agent that follows a dead link guesses or stops"

[lints.agent-rules-match-lints]
level = "deny"
reason = "a stated limit the lints check differently teaches agents a rule the repo does not hold"

# Terse output

[lints.changelog-entry-length]
level = "deny"
reason = "a CHANGELOG is a list: one line per change"

[lints.changelog-describes-diff]
level = "warn"
reason = "an entry says what changed for the user; the diff already shows the files"

[lints.commit-subject-length]
level = "deny"
reason = "the subject is what one-line logs and reviews show"

[lints.commit-body-lines]
level = "warn"
reason = "a body holds why, not a retelling of the diff"

[lints.commit-prose]
level = "warn"
reason = "commit messages follow the prose rules of the docs"

[lints.fixes-tag]
level = "deny"
reason = "an agent copies the culprit's hash and subject from git; a wrong one sends the next reader astray"

[lints.fix-without-test]
level = "warn"
reason = "an agent writes the failing test first; a warning, since a fix is judged from its subject"

[lints.commit-tested-line]
level = "warn"
reason = "an agent says how it tested, or that it did not; a warning while branches begun before the rule land"

[lints.commit-mixed-move]
level = "warn"
reason = "a move edited in the same commit cuts every line's history; a warning, since it judges lines by their start"

[lints.sentence-words]
level = "warn"
reason = "short sentences are read once and understood"

[lints.paragraph-sentences]
level = "warn"
reason = "result first, then what the reader must do; more facts are a list"

# AI-isms

[lints.filler-phrases]
level = "deny"
reason = "filler adds words and no meaning"

[lints.narration]
level = "deny"
reason = "docs and comments state facts, not the conversation that produced them"

[lints.thinking-comments]
level = "deny"
reason = "a comment holds the conclusion, not the reasoning that led to it"

[lints.emoji]
level = "warn"
reason = "emoji do not grep, diff or render the same everywhere"

[lints.restating-comments]
level = "warn"
reason = "a comment says what the code cannot"

[lints.url-tracking-params]
level = "warn"
reason = "a link pasted from a browser or a chat app carries an id that tells the reader nothing and goes stale"

# Stop-gap smells (the top rule)

[lints.stop-gap-comments]
level = "deny"
reason = "a TODO or a 'for now' marks a path the final design throws away; tracked work names its id"

[lints.fixme-before-land]
level = "deny"
reason = "an agent may leave itself a FIXME on its branch; the landing refuses it"

[lints.unit-suffix]
level = "warn"
reason = "agents write units last in names from the start; a warning, since it reads names only and fires widely in older code"

[lints.unbounded-read]
level = "warn"
reason = "input from outside sets the memory a read to the end takes; candidates only, a place to look"

[lints.todo-macros]
level = "deny"
reason = "todo!() and unimplemented!() are stop-gaps that compile"

[lints.commented-out-code]
level = "deny"
reason = "git keeps the old code; commented-out code is a decision not taken"

[lints.allow-without-reason]
level = "deny"
reason = "an allowed lint says why, so it can be turned back on"

[lints.lint-table-allow-reason]
level = "deny"
reason = "a lint allowed for a whole crate in Cargo.toml says why"

[lints.near-duplicate-fns]
level = "warn"
reason = "a rewrite that keeps the old function leaves two copies to fix"

[lints.near-duplicate-modules]
level = "deny"
reason = "a rewrite that keeps the old module is not finished"

[lints.dead-features]
level = "warn"
reason = "a feature that enables nothing and gates nothing is left over"

# Agent-config hygiene

[lints.leftover-files]
level = "deny"
reason = "merge, editor and scratch leftovers and session notes do not belong in history"

[lints.large-blobs]
level = "warn"
reason = "a large tracked file stays in every clone forever; agents commit dumps and binaries with git add -A"

[lints.agent-config-secrets]
level = "deny"
reason = "a committed key is public to every reader and stays in history"

[lints.agent-config-permissions]
level = "deny"
reason = "committed settings apply to every agent; a blanket grant lets any of them run anything"

[lints.agent-session-files]
level = "deny"
reason = "transcripts and per-person agent files hold private context and secrets"

# The Clippy lints that catch what agents write: allow without expect, stray prints, verbose conversions,
# repeated type names, long doc summaries, tangled functions.

[clippy.lints]
allow_attributes = { level = "warn", reason = "#[expect] fails once the lint stops firing, so an exception cannot outlive its cause" }
print_stdout = { level = "warn", reason = "agents leave debugging prints behind" }
print_stderr = { level = "warn", reason = "agents leave debugging prints behind" }
str_to_string = { level = "warn", reason = "to_owned says what happens: a copy of a borrowed str" }
use_self = { level = "warn", reason = "Self follows a rename" }
self_named_module_files = { level = "warn", reason = "one module layout, mod.rs, across the workspace" }
too_long_first_doc_paragraph = { level = "warn", reason = "agents write long doc summaries; the first paragraph is one short sentence" }
cognitive_complexity = { level = "warn", reason = "agents grow functions branch by branch; a tangled one is split" }

[clippy.config]
cognitive-complexity-threshold = { value = 20, reason = "set on samples: 17 read fine, 21 and 25 were too long; spl's function-complexity (cyclomatic, 15) counts paths" }
```
