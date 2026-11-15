## Why

Zensical reads `zensical.toml` key by key, applying its own defaults for the keys it knows and never looking at the rest: a key it does not recognize is neither used nor reported. `sitename` instead of `site_name`, `theme.customdir` instead of `theme.custom_dir`, a plugin option moved up a level by accident: the build still says "No issues found", and the setting the author meant to apply never takes effect, even with `strict = true`.

The lint walks the table Zensical reads (`[project]` when the file nests that way, the document root otherwise) and flags a key that is not one of `known`'s dotted paths (`site_name`, `theme.custom_dir`, ...). A path ending `.*` accepts any key below it: `extra`, `plugins`, `markdown_extensions` and a theme's admonition icon map are free-form, so their contents are not checked.

## How to fix

Correct the key, or drop it. A key Zensical does not read yet, or one from a theme or plugin this list does not cover: add its dotted path to `known` (`site_name`, `theme.favicon`, `plugins.*` for a whole plugin's options).
