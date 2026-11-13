## Why

Committed agent settings apply to every person and every agent session in the repo. A blanket grant (`Bash(*)`, `Bash(rm:*)`, `bypassPermissions`) lets any agent run anything without asking, including one steered by text in an issue or a web page.

For each tracked settings file matching `files` (JSON), the rule reports:

- entries of `permissions.allow` that are in `broad`;
- `permissions.defaultMode` set to `bypassPermissions`;
- `dangerouslySkipPermissions` or `skipDangerousModePermissionPrompt` set to true.

## How to fix

Allow the commands the project needs by name (`Bash(cargo test:*)`, `Bash(just lint)`). Keep wider grants in personal, untracked settings.
