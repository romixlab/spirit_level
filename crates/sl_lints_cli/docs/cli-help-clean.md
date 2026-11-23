## Why

clap turns doc comments into help text. A struct flattened into the parser (`#[command(flatten)]`) brings its own rustdoc along: code fences, intra-doc links and doctests then open the tool's `--help`. The text was written for rustdoc, not for a terminal, and it hides the tool's real description.

## Example

```console
$ tool --help
The `--color` flag, flattened into a CLI's root parser ...
``` use clap::Parser; use vc_ui::{ColorChoice, ColorFlag};
```

## How to fix

Set `about` and `long_about` explicitly where the root parser derives `Parser` (`#[command(about = "...", long_about = None)]`), so a flattened struct's documentation is not taken. Keep rustdoc out of argument help: give `help = "..."` where the doc comment is for the API. The parameter `markers` lists the text that marks rustdoc source.
