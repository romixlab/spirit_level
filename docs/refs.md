# References in prose

A book about code names two kinds of things in its prose: the terms it defines and the items of its code. With references, each becomes a link with a card that opens beside it.

| Written | Links to | Its card shows |
|---|---|---|
| `[[piece]]` | the term's entry on the glossary page | the one-sentence definition |
| ``[`TapeWriter::resume_on`]`` | the item's lines in the repository | kind, path, signature and docs |
| `CAN`, an abbreviation of the book's list | nothing | its long form, as a tooltip |

The book kit [vhrd_zensical](https://github.com/vhrdtech/vhrd_zensical) renders them: Markdown extensions for Zensical, the `vz` command that writes the code index and the cards' files before each build, the console and Rust lexers, and the footer's build stamp. A book pins one version of the kit. Its [README](https://github.com/vhrdtech/vhrd_zensical#readme) says how to turn each part on.

## What spl checks

spl reads the kit's settings in `zensical.toml` (or `mkdocs.yml`) and the files they name, so its lints judge the same files the build uses.

| Lint | Reads | Reports |
|---|---|---|
| [glossary-terms](lints/D0026.md) | `vhrd_zensical.glossary`: its `file`, `docs/glossary.toml` by default | a section that uses a term but never marks it; a glossary that does not read |
| [abbreviation-first-use](lints/D0012.md) | `vhrd_zensical.abbreviations`: its `file`, `docs/abbreviations.toml` by default | an abbreviation in neither the lint's `known` list nor the book's file; one spelled out in prose |
| [book-build-stamp](lints/D0017.md) | `vz stamp`, `vz prep` or `vz build` in a just recipe or a workflow | a footer without a build stamp, or a stamp without its age |

```toml
[project.markdown_extensions]
abbr = {}
"vhrd_zensical.glossary" = {}
"vhrd_zensical.abbreviations" = {}
"vhrd_zensical.code_refs" = { crates = ["ww_tape"], on_crates_io = ["ww-tape"] }
```
