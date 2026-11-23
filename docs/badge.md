# The README badge

`spl badge` writes a badge for the README: the tier the project extends, and a level vial whose bubble shows how strict its configuration is.

| Tier | Badge |
|---|---|
| `extends = "small"` | spirit level · small, bubble at the left, amber |
| `extends = "medium"` | spirit level · medium, bubble in the middle, green |
| `extends = "big"` | spirit level · big, bubble at the right, teal |
| `extends = ["medium", "ai", "units-strict"]` | spirit level · medium +2 |

```bash
spl badge                         # writes .github/spl-badge.svg
spl badge --out docs/badge.svg    # another place
spl badge --out -                 # prints it
```

The README links it to the book's page of the tier:

```markdown
[![spirit level](.github/spl-badge.svg)](https://spl.vhrd.tech/tiers/medium/)
```

## What it shows

- The right side names the size tier the configuration extends (small, small-workspace, medium, big). Every other name it extends, a set, a tier beside the size tiers such as `ai`, or a shared file, adds one to `+N`. A configuration that extends no size tier shows the first name it extends, or `custom`.
- The bubble shows the share of spirit_level's registered lints the configuration turns on, at the level of the project's own configuration: `warn` and `deny` count, `allow` does not. Overrides count, scoped exceptions and crate tiers do not.
- The scale runs from what the small tier turns on, at the left end, to what the big tier turns on, at the right end. A configuration below small or above big stays at the end.
- The right side's colour follows the bubble: amber at the left, green in the middle, teal at the right.
- Clippy's own lints are not counted: the Clippy adapter lints (`clippy-correctness`, `clippy-pedantic` and the others) count as one lint each, whatever Clippy set the project turns on. The share measures spirit_level's checks, which differ in size from tier to tier, and not the hundreds of Clippy lints a project may switch on one by one.
- Hovering the badge shows the counts: `spirit level: medium (113 of 324 lints on)`.

## A file that stays current

The badge is a self-contained SVG. It loads no font, no script and nothing from the network: text widths come from a built-in Verdana width table, as shields.io measures them. It reads on light and dark READMEs alike, and its edge follows the reader's colour scheme.

The same configuration always gives the same bytes. [readme-badges](lints/M0034.md) reports a README without the badge and a committed badge whose text differs from what `spl badge` writes: another tier or another `+N`, after a change to `spirit_level.toml`. A bubble or colour that moved is not a finding: they follow spirit_level's own lint counts, which change with its releases, so a committed badge stays valid until the configuration changes. `spl check --fix` writes the whole file, bubble included, and inserts the link; `spl badge` refreshes the bubble at any time.

The lint's parameters set the file (`badge`, empty for no badge) and the book it links to (`book`). `spl badge` writes to `badge` by default.
