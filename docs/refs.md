# References in prose

A book about code names two kinds of things in its prose: the terms it defines and the items of its code. With references, each becomes a link with a card that opens when the reader points at it, taps it or tabs to it.

- A glossary term, written `[[piece]]`, links to the term's entry on the glossary page. Its card shows the one-sentence definition.
- A code item, written ``[`TapeWriter::resume_on`]``, links to the item's lines in the repository. Its card shows the signature, the first paragraph of the docs and the file and line.

Both are checked when the book builds. A term the glossary lacks, or a code item that was renamed or removed, fails the build in the same change that breaks it.

## The parts

Three parts work together, and none of their output belongs in git.

| Part | What it does |
|---|---|
| `spl_book_ext` | The Markdown extensions Zensical loads. They turn the references into links and cards. |
| `spl book refs` | Run before each build. It checks the glossary and writes the code index and the cards' stylesheet and script. |
| `glossary-terms` | A lint. It reports a section that uses a term but never marks it. |

The extension is a Python package, since Zensical loads Python-Markdown extensions. It lives in this repository under `python/spl_book_ext`.

## Turning it on

Add the extensions to the book's `zensical.toml`, with the cards' stylesheet and script. The repository's address, `repo_url`, is where code links open.

```toml
[project]
repo_url = "https://github.com/vhrdtech/ww_tape"
extra_css = ["stylesheets/spl-refs.css"]
extra_javascript = ["javascripts/spl-refs.js"]

[project.markdown_extensions]
"spl_book_ext.glossary" = {}
"spl_book_ext.code_refs" = { crates = ["ww_tape", "ww_tape_fs"] }
```

Then build the book through a recipe that runs `spl book refs` first and adds the package to Zensical's environment:

```just
book:
    spl book refs
    uv run --with zensical==0.0.74 --with "spl-book-ext @ git+https://github.com/romixlab/spirit_level#subdirectory=python/spl_book_ext" zensical build --clean --strict
```

`spl book refs` writes the stylesheet and the script into the docs folder, so list both in `.gitignore`. The code index goes to `target/spl-book/code_index.json`, which a Rust repository already ignores.

### Settings

Each extension takes its settings in `zensical.toml`. Paths are relative to the project root, where the book is built from.

| Extension | Setting | Default | Meaning |
|---|---|---|---|
| `spl_book_ext.glossary` | `file` | `docs/glossary.toml` | The glossary file. |
| `spl_book_ext.glossary` | `page` | `/glossary/` | Where the glossary page is served. Links go to its anchors. |
| `spl_book_ext.code_refs` | `index` | `target/spl-book/code_index.json` | The code index that `spl book refs` writes. |
| `spl_book_ext.code_refs` | `crates` | every library of the workspace | The library crates indexed, by the names code uses. |

`spl book refs` takes `--css` and `--script` to write the cards' files elsewhere. In CI it takes the commit from `GITHUB_SHA`; elsewhere it asks git for `HEAD`.

## The glossary

The glossary is a TOML file with one `[[term]]` table per term. A term has a name, the other forms prose uses for it, and a definition of one sentence.

```toml
[[term]]
name = "piece"
forms = ["pieces"]
short = "One file of a tape, written block by block and closed by its finish block."
chapter = "../pieces/"

[[term]]
name = "CRC"
long = "cyclic redundancy check"
short = "A checksum that each block carries, so a reader detects a damaged block."
```

`chapter` is the page that explains the term; the glossary page links to it. `long` is an abbreviation's long form. With the `abbr` extension on, the long form shows on hover wherever the abbreviation is written, on every page.

A word names one term at most. A form listed under two terms fails `spl book refs` and the build, since a reference could then be read two ways.

### Marking terms

In prose, `[[piece]]` marks a term by any of its forms, and `[[pieces]]` finds the same entry. To show other words, put the term after a bar: `[[the last file|piece]]`.

Mark a term at its first use in each section, not at every use. A reader arriving from search or a link finds the card where the section first needs it, and the page is not underlined throughout.

The glossary page holds one line, `<!-- glossary -->`, which becomes every entry in order of name. Each entry has the anchor `#term-NAME` that the references link to.

## Code links

A code link uses rustdoc's intra-doc link form, which Rust readers know: ``[`TapeWriter::resume_on`]``. A plain code span stays plain, so file names and values are untouched.

The name is matched against the end of each item's path. ``[`writer::resume`]`` finds `ww_tape_fs::writer::resume`, and a name that starts with a crate's name matches only in that crate. A trailing `()` or `!` is shown and ignored when matching.

A name that matches no item fails the build. A name that matches several items fails it too, and the error lists their paths, so more of the path can be written.

### The index

`spl book refs` walks each library's modules from its root, as the compiler does. It lists every public item: functions, methods, structs and their public fields, enums and their variants, traits, type aliases, constants, statics and exported macros.

Each entry has the item's path through the modules it is defined in, its file and lines, and its source. A short item shows whole, up to 14 lines; a longer one shows its signature. The link opens the item's lines at the commit the book is built from, so the lines always match the text.

## The lint

`glossary-terms` reads the glossary and every page the book builds. It reports a term that a section uses but never marks, at its first use there.

The lint matches words without regard to sense, so its findings are estimates. Terms that are also everyday words go into its `ignore` parameter; the lint's page has the details.
