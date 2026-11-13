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
| [C0002](../lints/C0002.md) | allow-without-reason | `deny` | an allowed lint says why, so it can be turned back on | ai |
| [C0008](../lints/C0008.md) | todo-macros | `deny` | todo!() and unimplemented!() are stop-gaps that compile | ai |
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

# Stop-gap smells (the top rule)

[lints.stop-gap-comments]
level = "deny"
reason = "a TODO or a 'for now' marks a path the final design throws away; tracked work names its id"

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

[lints.agent-config-secrets]
level = "deny"
reason = "a committed key is public to every reader and stays in history"

[lints.agent-config-permissions]
level = "deny"
reason = "committed settings apply to every agent; a blanket grant lets any of them run anything"

[lints.agent-session-files]
level = "deny"
reason = "transcripts and per-person agent files hold private context and secrets"
```
