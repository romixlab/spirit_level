## Why

An admonition is a box for a note, a tip or a warning:

```markdown
!!! warning "Disconnect the battery"

    The board draws current from the battery even when it is switched off.
```

Zensical styles twelve types: note, abstract, info, tip, success, question, warning, failure, danger, bug, example and quote. Any other type renders as a plain note, so a `!!! caution` loses the colour that marks it as a warning. The body must be indented four spaces. With two spaces, Python-Markdown ends the box at the title and renders the body as ordinary text below it.

## How to fix

Use one of the types in `types`. Indent every line of the body by four spaces. A book that adds its own types with custom CSS lists them in `types`.
