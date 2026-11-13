## Why

A cargo feature without a comment can only be understood by reading the code it gates. Bevy fails CI on a feature with no doc line; the comment above the feature is its documentation.

## Example

```toml
[features]
# Serialize every public type with serde
serde = ["dep:serde"]
```

## How to fix

Put a `#` comment line right above each feature. Features in the parameter `skip` (by default `default`) need none.
