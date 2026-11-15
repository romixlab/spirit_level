## Why

Zensical reads `zensical.toml` key by key, applying its own defaults for the keys it knows and never looking at the rest: a key it does not recognize is neither used nor reported. `sitename` instead of `site_name`, `theme.customdir` instead of `theme.custom_dir`, a plugin option moved up a level by accident: the build still says "No issues found", and the setting the author meant to apply never takes effect, even with `strict = true`. A book Zensical builds from a legacy MkDocs project reads `mkdocs.yml` instead, unchanged: the same keys, the same silent typo.

The lint walks the table Zensical reads (`[project]` when `zensical.toml` nests that way, `mkdocs.yml`'s document root, or the TOML file's own root when it has no `[project]`) and flags a key that is not one of `known`'s dotted paths (`site_name`, `theme.custom_dir`, ...). A path ending `.*` accepts any key below it: `extra`, `plugins`, `markdown_extensions` and a theme's admonition icon map are free-form, so their contents are not checked. A list of named variants (`theme.palette`, written as a TOML array of tables or a YAML list) is checked the same way, one entry at a time; `nav`'s list is not, since its keys are reader-chosen page titles.

## How to fix

Correct the key, or drop it. A key Zensical does not read yet, or one from a theme or plugin this list does not cover: add its dotted path to `known` (`site_name`, `theme.favicon`, `plugins.*` for a whole plugin's options).
