## Why

A reader who meets an abbreviation they do not know stops and looks for its meaning. Readers arrive on any page from a search or a link, so the meaning has to be on that page. Spelling the abbreviation out in prose ("personal computer (PC)") slows every reader who knows it, and repeats on page after page.

So a project sorts its abbreviations into two lists in `spirit_level.toml`:

| List | Holds | In the book |
|---|---|---|
| `known` | abbreviations every reader knows: `PC`, `USB`, `CPU` | written plain, no tooltip |
| `long-forms` | every other abbreviation, with its long form: `CAN = "Controller Area Network"` | written plain; the long form shows on hover |

The lint flags an abbreviation in neither list, at its first use on a page. It also flags prose that spells out an abbreviation of either list. Prose never carries the long form: the book shows it.

The lint reads a word of two to six capitals and digits, with at least two capitals and at most one digit, as an abbreviation: `PWM`, `I2C`, `MCUs`. A word with more digits is a part number (`STM32`). It counts as spelled out when two or more words stand next to it in parentheses, either way round, and one of them starts with its first letter. "Stands for", "short for" or "means" after it spell it out too. A file name such as `AGENTS.md`, inline code and words written in capitals for stress (`MUST`, `NOT`) are not abbreviations.

The finding is a candidate: some words in capitals are names, not abbreviations.

```markdown
The CAN bus links the boards.                       <!-- flagged while CAN is in neither list -->
The Controller Area Network (CAN) links the boards. <!-- flagged: spelled out -->
A personal computer (PC) reads the log.             <!-- flagged: PC is known -->
The CAN bus links the boards; a PC reads the log.   <!-- right, with CAN in long-forms -->
```

## Long forms on hover

The book shows the long forms with spl's Markdown extension `spl_book_ext.abbreviations` and Python-Markdown's `abbr` extension. Both go into `zensical.toml`:

```toml
[project.markdown_extensions]
abbr = {}
"spl_book_ext.abbreviations" = {}
```

`spl book refs` writes the table, resolved from `spirit_level.toml` and its tiers, to `target/spl-book/abbreviations.json` before each build. The extension reads it and marks every tabled abbreviation in a page as `<abbr title="long form">`. The theme's `content.tooltips` feature shows the title as a tooltip. Inline code and code blocks keep the abbreviation as written. A page outside the book, such as a README on a forge, shows the abbreviation without its long form.

## How to fix

- An abbreviation every reader of the project knows: add it to `known`.
- Any other: add it to `long-forms` with its long form, and write the abbreviation alone.
- A spelled-out one: delete the long form from the prose.

```toml
[lints.abbreviation-first-use]
level = "deny"
reason = "common abbreviations plain, the rest with their long form on hover"
known = ["PC", "USB", "CPU"]

[lints.abbreviation-first-use.long-forms]
CAN = "Controller Area Network"
SD = "Secure Digital"
```
