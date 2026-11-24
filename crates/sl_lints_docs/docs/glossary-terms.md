## Why

A book that turns on the glossary extension (`vhrd_zensical.glossary` of the book kit, see [References in prose](../refs.md)) shows a term's definition when the reader points at `[[term]]`. A reader arrives in any section, from search or a link, and needs that card where the section first uses the term. Marking every use would underline half of every page, so the rule is one mark per section, at the first use.

The lint reads the glossary the book's settings name and every page the book builds. In each section, the prose between two headings, a term written plainly but never marked is reported at its first use. Every form of a term counts (`forms` in the glossary), longer terms are matched first, and inline code is not read.

The words are matched without regard to sense, so the finding is an estimate: "blocks the thread" reads as the term "block". Terms that are also everyday words go into `ignore`:

```toml
[lints.glossary-terms]
level = "deny"
ignore = ["clock"]
```

## How to fix

Write the first use in the section as `[[pieces]]`, or `[[the end of the piece|tail]]` to show other words. Where the word does not mean the term, waive that line or rephrase it.
