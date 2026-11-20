# Docs as the reader meets them

The prose lints check the wording of a document: filler, narration, long sentences. The docs lints check its shape and its words: how long a reader goes without a heading, a figure or a table, how long the page takes to read, whether a long page has sections, whether code comes with an explanation, whether abbreviations and terms are explained before the reader needs them. They also check the practices of a [Zensical](https://zensical.org/docs/) book: a navigation that names real pages, no page that nobody can reach, one title per page, admonitions that render.

The numbers are useful on their own, before any budget. `spl pages` prints them for every page, and the lints in the `reading`, `words` and `book` categories warn when a page goes over a budget.

## The pages report

`spl pages` prints one row per page:

```text
page                     kind       words  read  run  prun  para  sent  flesch  pass%  abbr  terms  syn  img  w/img  w/el   h  links  cta  code
README.md                guide        412  1.8m  140   160    48    14      52     12     1      0    0    1    380    63   6    9/3    0     4
docs/configuration.md    guide       1630  7.1m  210   420    61    16      48     21     0      2    1    0      -    74  14   12/0    0    11
docs/lints/D0001.md      reference    310  1.3m   88    88    40    11      55     18     0      0    0    0      -   103   6    8/0    0     1
```

| Column | Meaning |
|---|---|
| `kind` | `guide` or `reference` (see [Kinds of page](#kinds-of-page)) |
| `words` | words of prose and headings |
| `read` | reading time in minutes, images included |
| `run` | the longest run of prose words between two visual breaks |
| `prun` | the longest run of paragraph words with no list, figure, table or code block between them; headings do not end it |
| `para` | the longest paragraph, in words |
| `sent` | the median sentence, in words |
| `flesch` | Flesch reading ease, English pages only; `-` otherwise |
| `pass%` | percent of the sentences that read as passive, English pages only |
| `abbr` | abbreviations not spelled out at their first use |
| `terms` | terms used before the page defines them |
| `syn` | uses of a name the project's word list prefers otherwise |
| `img` | figures: paragraphs that hold only images, HTML figures and drawings, diagram blocks |
| `w/img` | prose words per figure; `-` without figures |
| `w/el` | prose words per element: a list, figure, table, code block or other break |
| `h` | headings |
| `links` | internal and external links |
| `cta` | calls to action: links and buttons styled to be pressed (built pages) |
| `code` | code blocks |

`spl pages --page PAGE` prints every number of one page, with its runs, its headings and each abbreviation, term and name the last four columns count. `spl pages --format json` prints all of them for every page as one document. Two JSON files from two runs diff line by line, so a change to a page shows what it moved.

The report uses the parameters of the lints, so the report and the lints count the same way: the reading speed and image times come from `page-reading-time`, the budgets from each lint.

## What is measured where

spirit_level reads the Markdown sources by default. The sources are always in the repository, and a finding points at the line to change. A built site holds what the sources do not: pages from templates, included snippets, the rendered navigation. A site whose pages live mostly in templates is read from its built HTML instead (see [A built site](#a-built-site)). The table says where each fact comes from for Markdown pages.

| Fact | Read from |
|---|---|
| Prose, headings, code blocks, tables, images, links | The Markdown pages |
| Admonitions and content tabs (`!!!`, `???`, `===`) | The Markdown pages, with their indented bodies read as Markdown |
| Title and description | The page's front matter and its first heading |
| Navigation, the docs folder, `not_in_nav`, the theme language | `zensical.toml` at the project root |

A book configured by `mkdocs.yml` alone gets the reading lints but not the book lints: spirit_level reads TOML, not YAML.

### How a page is measured

- **Prose** is the text of paragraphs, list items and block quotes. Inline code inside a sentence counts as words. Headings, code blocks, tables, image text and raw HTML are not prose.
- **Visual breaks** are headings, code blocks, tables, figures, admonitions, content tabs and HTML figures, videos, tables and `<details>` blocks. A **run** is the prose between two breaks.
- **Figures** are paragraphs that hold only images (a row of badges is one figure), HTML `<figure>`, `<img>` and `<svg>` blocks, and diagram blocks: fenced blocks in `mermaid`, `dot`, `graphviz`, `plantuml`, `puml`, `d2`, `svgbob`, `bob`, `wavedrom` or `pikchr`, which a book draws. A diagram block is a figure, not code.
- A **paragraph run** is the paragraphs in a row with no list, figure, table, code block or admonition between them. Headings do not end it, so many short sections of prose are one run.
- **Reading time** is words divided by words per minute, plus a falling time per figure: the first figure costs `image-first-seconds`, each one after it a little less, down to `image-min-seconds` over `image-falloff-count` figures. Every further figure costs the floor.
- **Sentences** are split after `.`, `!` or `?` and a space, except after a known abbreviation, a single letter or a number. The same splitter serves `sentence-words`.
- **Words** in inline code are names, not prose: the word lints do not read them. A **definition** is bold text at a term's first use, or a sentence such as "A tier means ...".

## Which pages

The optional `[docs]` table in `spirit_level.toml` says which Markdown files are pages, and which of them are reference pages:

```toml
[docs]
files = ["**/*.md"]                                  # globs of the pages, relative to the root
skip = ["CHANGELOG.md", "FEATURES.md", "AGENTS.md", "CLAUDE.md", "**/CHANGELOG.md", "**/tests/**"]
reference = ["docs/lints/**", "docs/tiers/**"]       # pages read by lookup, not front to back
language = "en"                                      # default: the book's theme language, else "en"
```

The values shown for `files` and `skip` are the defaults. A walk from the root leaves out `target/`, hidden folders and nested cargo projects that are not workspace members, such as test fixtures.

### A built site

A site built from templates and shortcodes, such as a zola site, holds most of a page outside its Markdown. The `[docs.site]` table makes the reading and word lints and `spl pages` read the built HTML in place of the Markdown sources:

```toml
[docs.site]
generator = "zola"                 # "zola" or "zensical"; default: found from zensical.toml, zola.toml or config.toml
dir = "public"                     # the built site; default: zola's public, Zensical's site_dir
content = "main"                   # the element holding a page's content: tag, .class or tag.class
sources = ["content/**"]           # the Markdown files whose built pages replace them
url = "https://example.org"        # the site's own address: a link to it is internal
files = ["**/*.html"]              # the built pages, relative to dir
skip = ["404.html"]                # built pages left out, relative to dir
skip-classes = ["v-label", "specs"]    # elements whose text is not prose
cta-classes = ["v-btn"]            # links and buttons that are calls to action
capture-classes = ["term-fig"]     # figures that show a terminal capture
```

Every key is optional. Without `dir`, `content`, `sources` and `url`, the generator's own settings decide:

| Key | zola | Zensical |
|---|---|---|
| `dir` | `public` | `project.site_dir` |
| `content` | `main` | `article` |
| `sources` | `content/**` | `project.docs_dir` and below |
| `url` | `base_url` | `project.site_url` |

The default `skip-classes` are Zensical's heading anchors, admonition titles, tab labels, edit buttons and page dates; the default `cta-classes` is Zensical's `md-button`. A list set in the table replaces its default.

spirit_level does not build the site: build it first (`zola build`, `zensical build`). A missing build folder stops the reading and word lints with a note, and `spl pages` with exit code 2. A built page has these differences from a Markdown page:

- Only the content element counts. The header, navigation and footer outside it never do, nor do scripts, styles and elements marked `aria-hidden`. An inline SVG drawing is a figure; its labels are not prose.
- A `<pre>` with a diagram language as a class (`<pre class="mermaid">`) is a figure.
- **Prose** is the text of `<p>`, `<li>` and `<blockquote>`. Text in a figure caption, a `<summary>`, a `<dt>` or an element of a `skip-classes` class is neither prose nor a break.
- **Visual breaks** are headings, `<pre>` blocks, `<figure>`, `<video>`, `<table>`, `<details>`, admonitions, content tabs and calls to action. A figure of a `capture-classes` class is a break but no image to look at.
- A code block's language comes from a `language-*` class or a `data-lang` attribute on the `<pre>`, its `<code>` or an element around them.
- A link to the site's own `url` is internal. `page-links` judges Markdown pages only: the links of a built page are the generator's.
- Each built page's `<html lang>` decides whether it is English, ahead of the `[docs]` language.
- A built page with no content element, such as a redirect, is no page.
- A finding points at the built file's line. The `[docs]` `reference` globs match the built path, such as `public/docs/**`.

The book lints still read the Markdown sources and `zensical.toml`.

### Pages of another repo's book

A library's pages can be a section of a book that another repo builds: that repo copies the folder into its own `docs`. A link from such a page to the rest of the book is right there and leads nowhere here. The `[docs.book]` table says where the pages stand:

```toml
[docs.book]
dir = "docs"                # the folder of pages the book takes
at = "docs/format"          # where the folder stands in the book's repo
added = ["conformance.md"]  # optional: pages the book's repo keeps in that folder itself
repo = "../the_book"        # optional: that repo's checkout, relative to this file
```

[page-links](lints/D0021.md) then resolves a link of a page under `dir` from the page's place in the book:

| The link leads | Checked |
|---|---|
| into `at` | the file exists under `dir` in this repo |
| to a page of `added` | as another page of the book |
| to another page of the book | the file exists in `repo`; a miss is a candidate, since the book may take that page from a third repo |
| to another page, and `repo` is not set or not checked out | not judged |

Pages outside `dir`, such as the README, are judged as before. Read from this repo's source tree, a link to the rest of the book stays broken: the table records that the book is where these pages are read.

### Kinds of page

A guide is read from front to back: an introduction, a tutorial, a concept page. A reference page is looked up: a lint's page, an API list, a table of options. A reader does not read a reference page whole, so its length is no burden, but a long stretch of prose without a heading is still hard to scan.

A page is a reference page when one of the `reference` globs matches its path, and a guide otherwise. Generated pages usually sit in their own folder, so one glob covers them. Each budget lint has a `pages` parameter: `all`, `guide` or `reference`. The defaults judge reading time, heading length and code walls on guides only, and runs of prose and sections on every page.

## Language

Flesch reading ease is a formula for English: its constants were fitted to English text, and its syllable count is an English heuristic. The report shows it only when the page language is English (`language` starts with `en`), and shows `-` for any other language. The same holds for the passive share. `reading-ease` and `passive-voice` judge English guides only, with a low floor by default: a technical page full of exact terms scores low however clear it is.

Every other measure counts words and breaks and works in any language that separates words with spaces. Reading speeds differ by language; a project in another language sets `wpm` on `page-reading-time`.

## The catalogue

Every practice the docs lints were built from, where it comes from, and how it is checked. A practice that needs judgement stays with the reviewer: a lint that guesses would bury its true findings under false ones.

| Practice | Source | Checked by | Confidence |
|---|---|---|---|
| Prose runs between visual breaks stay short | vhrd_site page stats (longest run) | [prose-run-words](lints/D0001.md) | certain |
| A guide reads in a bounded time | vhrd_site page stats (reading time) | [page-reading-time](lints/D0002.md) | estimate: assumes the reading speed |
| Headings are short names, not sentences | vhrd_site page stats (words per heading); prose rules | [heading-words](lints/D0003.md) | certain on length |
| A heading names what its section answers | prose rules | review | |
| A long page is divided into sections | vhrd_site page stats (headings) | [page-headings](lints/D0004.md) | certain |
| Code comes with prose that explains it | vhrd_site page stats (code blocks) | [code-wall](lints/D0005.md) | certain |
| Images have alternative text | vhrd_site page stats (images without alt) | [image-alt](lints/D0006.md) | certain |
| Code blocks name their language | [Zensical: code blocks](https://zensical.org/docs/authoring/code-blocks/) | [code-block-language](lints/D0007.md) | certain |
| The navigation names pages that exist | [Zensical: navigation](https://zensical.org/docs/setup/navigation/) (a path that is not a page becomes an external link) | [nav-entries](lints/D0008.md) | certain |
| Every page is reachable from the navigation or a link | [Zensical: basics](https://zensical.org/docs/setup/basics/) (`not_in_nav` pages are still built) | [orphan-pages](lints/D0009.md) | certain |
| A page has one title | [Zensical: page title](https://zensical.org/docs/authoring/markdown/#page-title) | [page-title](lints/D0010.md) | certain |
| Admonitions use a known type and a body indented four spaces | [Zensical: admonitions](https://zensical.org/docs/authoring/admonitions/) | [admonition-syntax](lints/D0011.md) | certain |
| Paragraphs stay short | vhrd_site page stats (paragraph words); prose rules | [paragraph-sentences](lints/P0013.md), in sentences | certain |
| Sentences stay short | vhrd_site page stats (sentence length); prose rules | [sentence-words](lints/P0015.md) | certain |
| Links and their anchors resolve in a book | [Zensical: validation](https://zensical.org/docs/setup/validation/) | `zensical build --strict` (on by default) | certain |
| Links in agent files resolve | | [agent-file-links](lints/P0002.md) | certain |
| Pages link to `.md` files by relative path, never to `.html` or the site's absolute paths | [Zensical: linking between pages](https://zensical.org/docs/authoring/markdown/#linking-between-pages) | [page-links](lints/D0021.md) | certain |
| No folder holds both `README.md` and `index.md` | [Zensical: Markdown](https://zensical.org/docs/authoring/markdown/) | [readme-index-clash](lints/D0018.md) | certain |
| Theme features that exclude each other are not both on | [Zensical: navigation](https://zensical.org/docs/setup/navigation/) | [theme-feature-conflicts](lints/D0019.md) | certain |
| A page meant as an entry point has a description | [Zensical: front matter](https://zensical.org/docs/authoring/frontmatter/) | review: which pages are entry points | |
| A page has enough figures for its length | vhrd_site page stats (words per image) | review, with `img` in the report | |
| The text is easy to read | vhrd_site page stats (Flesch) | [reading-ease](lints/D0016.md), `flesch` in the report | estimate: syllables by a spelling heuristic |
| Sentences say who acts | prose rules | [passive-voice](lints/D0015.md), `pass%` in the report | estimate: "to be" before a past participle |
| Abbreviations are spelled out at first use | prose rules | [abbreviation-first-use](lints/D0012.md) | candidate |
| Terms are defined at first use | prose rules | [term-before-definition](lints/D0013.md) | candidate: bold or a definition sentence |
| One concept has one name | prose rules | [term-synonyms](lints/D0014.md), with the project's word list | candidate |
| Why and what come before how | prose rules | review | |
