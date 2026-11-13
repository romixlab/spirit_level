## Why

Agent and MCP configuration (`.claude/`, `.mcp.json`, `.cursor/`, ...) often holds API keys for the tools it starts. Committed, the key is public to everyone who can read the repo, and stays in history.

For each tracked file matching `files`, a line is reported when either holds:

- a word starts with a known token prefix (`sk-`, `ghp_`, `xoxb-`, `AKIA`, ...) and has at least 12 more characters;
- a key whose name contains one of `keys` (`token`, `secret`, `password`, ...) has a literal value; placeholders (`$VAR`, `${VAR}`, `<your key>`, `xxxx`) pass.

```json
{ "env": { "GITHUB_TOKEN": "${GITHUB_TOKEN}" } }      <- passes
{ "env": { "GITHUB_TOKEN": "ghp_1234567890abcdef" } } <- reported
```

## How to fix

Revoke the key. Remove it from the file and from history, and read it from the environment.
