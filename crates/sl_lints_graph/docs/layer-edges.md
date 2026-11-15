## Why

Layers keep a workspace from growing back into one tangle: a crate may depend only on the layers its own layer allows, so the bottom stays small and stable and the top stays thin. The check reads the real cargo graph, edge by edge, so a declared structure cannot drift from the built one. Dev-dependencies are exempt (tests may reach up); build dependencies are not.

When a crate's map entry lists `deps`, every edge must also be one of them: a new edge between two crates is then a visible change of the map.

An edge to the workspace-hack package is exempt: hakari makes every member depend on it, so no layer or `deps` list has to name it. The package is the one `hakari-package` names in `.config/hakari.toml` (or `.guppy/hakari.toml`); a crate merely called `workspace-hack` is checked like any other. The hack's own dependencies are still checked.

## Example

```toml
[map.layers.model]
allow = ["utils", "model"]

[map.crates]
foo_model = { layer = "model" }
foo_ui = { layer = "ui" }
```

`foo_model` depending on `foo_ui` is reported: model may not see ui.

## How to fix

Move the crate to the layer it belongs in, or move the code that needs the upper crate into a crate above. Do not reorder the layers to make an edge fit (rerun: "move a crate, do not reorder the layers").
