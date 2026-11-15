## Why

A flat `crates/` folder reads well up to a few dozen crates: prefixes group them, and the listing fits a screen. Past that the listing stops telling what the parts are, and finding a crate means knowing its name. A large workspace puts its crates in layer folders (`crates/<layer>/<crate>`), and a layer of about a hundred crates or more splits once more by discipline (`crates/engines/rf/<crate>`), never deeper ([crate-folder-layer](G0011.md)).

The lint counts the crate folders directly inside each folder that holds crates. A folder named after a layer of the [crate map](../map.md), or inside one, is held to `max-crates`; any other folder, such as a flat `crates/`, to `max-flat`.

## How to fix

A flat folder past `max-flat`: give every crate its layer folder, as the map's layers say, and update the `members` globs. A layer folder past `max-crates`: give its crates a `discipline` in the map and move each into `crates/<layer>/<discipline>/<crate>`.
