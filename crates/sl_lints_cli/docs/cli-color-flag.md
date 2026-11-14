## Why

Automatic colour guesses wrong sometimes: a pager that understands escapes, a CI log that renders them, a terminal a person wants plain. A `--color auto|always|never` flag settles it the same way in every tool. It is global, so it works after the subcommand too (`tool sync --color never`).

## Example

```rust
#[derive(clap::Parser)]
struct Cli {
    #[command(flatten)]
    color: vc_ui::ColorFlag, // --color <WHEN>, global, default auto
    #[command(subcommand)]
    cmd: Cmd,
}
```

## How to fix

Flatten `vc_ui::ColorFlag` into the root parser and call `cli.color.apply()` after parsing, or declare `--color <WHEN>` with the values auto, always and never and `global = true`. The parameters `flag` and `values` name the flag and its values.
