## Why

Libraries are the integration surface between the parts of a workspace; the CLI is for people, agents, hooks and recipes. A component that runs `Command::new("our-cli")` depends on whatever version of the binary is on PATH, parses text meant for people, and hides an edge the crate graph cannot see. tpm's records MCP server ran a `tpm records mcp-tools` child under a lock for every call.

The lint reports `Command::new("<name>")` where `<name>` is a binary target of the same workspace.

## How to fix

Call the library the binary is built on.
