## Why

Zensical links between pages by the Markdown source's relative path, and rewrites it when it builds the site:

```markdown
See [the configuration reference](../configuration.md#which-pages).
```

A link to the built page (`configuration.html`) or to an absolute path from the site root (`/configuration/`) works only inside the one book that built it: it breaks once the docs move to another base path, or are read straight from the source tree (a README on GitHub, an AGENTS.md an agent opens by path). `zensical build --strict` catches a broken link, but only inside a book: a page with no book, or a page a book leaves out of its navigation and no other page links to, gets no check from it.

The lint reads every page the `[docs]` table selects, book or not. A link that leaves the project (a URL, `mailto:`, `//host`) is not judged.

## How to fix

Link to the `.md` file, by a path relative to the page that links it. Drop the `.html` extension and any leading `/`. Fix the path, or drop the link, when it does not point at a file that exists.
