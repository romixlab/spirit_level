# justfile-recipes

**Rule:** The justfile has the standard recipes.

| | |
|---|---|
| Category | [repo](index.md#repo) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | `warn` | every repo answers to the same recipes: default, test, lint, install, deploy |
| [small-workspace](../tiers/small-workspace.md) | `warn` | every repo answers to the same recipes: default, test, lint, install, deploy |
| [medium](../tiers/medium.md) | `warn` | every repo answers to the same recipes: default, test, lint, install, deploy |
| [big](../tiers/big.md) | `warn` | every repo answers to the same recipes: default, test, lint, install, deploy |
| [ai](../tiers/ai.md) | off |  |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `recipes` | list of strings | `["default", "test", "lint", "install", "deploy"]` | recipes that must exist |

## Why

Every repo answers to the same recipes, so a person or an agent never has to learn a repo's way of testing: `just` lists them, `just test` and `just lint` check, `just install` puts the binaries on PATH, `just deploy` brings this PC up to main.

## How to fix

Add the missing recipes. The list is the parameter `recipes`.

## Configure

```toml
[lints.justfile-recipes]
level = "deny"
recipes = ["default", "test", "lint", "install", "deploy"]
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(justfile-recipes) <reason>` ([waivers](../waivers.md)).
