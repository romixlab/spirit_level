## Why

Zensical builds every page in the docs folder, whether or not the navigation lists it. A page that is neither in the navigation nor linked from another page is published but unreachable: no reader finds it, and nobody notices when it goes out of date.

A page counts as reached when the navigation lists it, or another page of the book links to it. The home page (`index.md`) is always reached. A page that `not_in_nav` names is left out on purpose and not judged. Pages that `exclude_docs` or `draft_docs` leave out of the build are not judged. The book's configuration is `zensical.toml`, or `mkdocs.yml` for a book Zensical builds from a legacy MkDocs project.

## How to fix

Add the page to `nav`, or link to it from a page that readers reach. Delete it if nobody needs it.
