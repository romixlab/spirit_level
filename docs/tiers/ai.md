# ai

Code and docs written with AI agents: agent files, terse output, AI-isms, stop-gap smells, agent-config hygiene.

Use it with `extends = "ai"` in `spirit_level.toml`.

## Lints enabled (with what it extends)

| Lint | Level | Reason | Set in |
|---|---|---|---|
| [agent-config-permissions](../lints/agent-config-permissions.md) | `deny` | committed settings apply to every agent; a blanket grant lets any of them run anything | ai |
| [agent-config-secrets](../lints/agent-config-secrets.md) | `deny` | a committed key is public to every reader and stays in history | ai |
| [agent-file-lines](../lints/agent-file-lines.md) | `deny` | agents read the file whole every session; a long one is skimmed, not followed | ai |
| [agent-file-links](../lints/agent-file-links.md) | `deny` | an agent that follows a dead link guesses or stops | ai |
| [agent-rules-match-lints](../lints/agent-rules-match-lints.md) | `deny` | a stated limit the lints check differently teaches agents a rule the repo does not hold | ai |
| [agent-session-files](../lints/agent-session-files.md) | `deny` | transcripts and per-person agent files hold private context and secrets | ai |
| [allow-without-reason](../lints/allow-without-reason.md) | `deny` | an allowed lint says why, so it can be turned back on | ai |
| [changelog-describes-diff](../lints/changelog-describes-diff.md) | `warn` | an entry says what changed for the user; the diff already shows the files | ai |
| [changelog-entry-length](../lints/changelog-entry-length.md) | `deny` | a CHANGELOG is a list: one line per change | ai |
| [claude-md-duplicate](../lints/claude-md-duplicate.md) | `deny` | two copies of the rules drift apart | ai |
| [commented-out-code](../lints/commented-out-code.md) | `deny` | git keeps the old code; commented-out code is a decision not taken | ai |
| [commit-body-lines](../lints/commit-body-lines.md) | `warn` | a body holds why, not a retelling of the diff | ai |
| [commit-prose](../lints/commit-prose.md) | `warn` | commit messages follow the prose rules of the docs | ai |
| [commit-subject-length](../lints/commit-subject-length.md) | `deny` | the subject is what one-line logs and reviews show | ai |
| [dead-features](../lints/dead-features.md) | `warn` | a feature that enables nothing and gates nothing is left over | ai |
| [emoji](../lints/emoji.md) | `warn` | emoji do not grep, diff or render the same everywhere | ai |
| [filler-phrases](../lints/filler-phrases.md) | `deny` | filler adds words and no meaning | ai |
| [leftover-files](../lints/leftover-files.md) | `deny` | merge, editor and scratch leftovers and session notes do not belong in history | ai |
| [lint-table-allow-reason](../lints/lint-table-allow-reason.md) | `deny` | a lint allowed for a whole crate in Cargo.toml says why | ai |
| [narration](../lints/narration.md) | `deny` | docs and comments state facts, not the conversation that produced them | ai |
| [near-duplicate-fns](../lints/near-duplicate-fns.md) | `warn` | a rewrite that keeps the old function leaves two copies to fix | ai |
| [near-duplicate-modules](../lints/near-duplicate-modules.md) | `deny` | a rewrite that keeps the old module is not finished | ai |
| [paragraph-sentences](../lints/paragraph-sentences.md) | `warn` | result first, then what the reader must do; more facts are a list | ai |
| [repo-files](../lints/repo-files.md) | `deny` | AGENTS.md holds the rules every agent reads; CLAUDE.md points to it | ai |
| [restating-comments](../lints/restating-comments.md) | `warn` | a comment says what the code cannot | ai |
| [sentence-words](../lints/sentence-words.md) | `warn` | short sentences are read once and understood | ai |
| [stop-gap-comments](../lints/stop-gap-comments.md) | `deny` | a TODO or a 'for now' marks a path the final design throws away; tracked work names its id | ai |
| [thinking-comments](../lints/thinking-comments.md) | `deny` | a comment holds the conclusion, not the reasoning that led to it | ai |
| [todo-macros](../lints/todo-macros.md) | `deny` | todo!() and unimplemented!() are stop-gaps that compile | ai |

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
