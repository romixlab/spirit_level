# Brand assets

The mark is a level's vial: the bubble at rest between the two lines. The vial is the spirit, the bubble is a hole in it, so the page behind shows through in either scheme.

Source of truth: `logo/logo.typ` (geometry, colours, wordmark) and `logo/render.typ` (one variant per call). `just logo` exports everything below; never edit an export by hand. Needs typst 0.13+ and Montserrat (SIL OFL) installed.

## Files

| File | Use |
|---|---|
| `logo/mark-light.svg`, `mark-dark.svg` | the mark alone, for light and dark pages; book header |
| `logo/mark-mono.svg` | one colour, `currentColor` |
| `logo/spirit_level-light.svg`, `-dark.svg`, `-mono.svg` | horizontal lockup; README header |
| `logo/spirit_level-stacked-light.svg`, `-dark.svg`, `-mono.svg` | stacked lockup |
| `logo/icon.svg`, `png/icon-180.png`, `png/icon-512.png` | the mark on the dark tile: avatar, apple-touch, app icon |
| `logo/icon-small.svg`, `png/icon-small-16/32/48.png`, `favicon.ico` | 16 to 48 px, one colour, no tile; book favicon |
| `png/spirit_level-light-1280.png`, `-dark-1280.png` | the horizontal lockup where SVG is not taken |
| `png/social.png` | 1280 × 640 card: GitHub Settings → Social preview |

The tile icons and the small icon have no light and dark pair: they read on both, checked on white, `#14171f` (book, dark) and `#0d1117` (GitHub, dark).

## Colours

| Role | Light | Dark | Contrast |
|---|---|---|---|
| Spirit (vial, underscore) | `#65a30d` | `#a3e635` | 3.1:1 on white; 11.9:1 on `#14171f` |
| Ink (lines, lettering) | `#373c42` | `#dddddd` | 11.1:1 on white; 13.2:1 on `#14171f` |
| Accent text in the book | `#4d7c0f` | `#a3e635` | 5.0:1 on white; 11.9:1 on `#14171f` |
| Tile, social card | `#0f0f10` | `#0f0f10` | spirit 12.7:1, ink 14.1:1 |
| Tagline (social card) | `#a6a8aa` | `#a6a8aa` | 8.0:1 on the tile |

The small icon uses the light spirit alone: 3.1:1 on white, 5.8:1 on `#14171f`.

## Clear space and size

- Clear space on every side: half the vial's height (a quarter of the mark's height).
- Mark: the large drawing from 48 px wide; below that, `icon-small` (drawn on a 32 grid, sharp at 16 and 32 px).
- Horizontal lockup: 120 px wide at least. Stacked: 80 px.

## Do

- Use the light variant on light backgrounds and the dark one on dark; `mono` for one-colour print or embossing.
- Keep the bubble a hole: a background, photo or tile behind the mark shows through it.
- Scale the files as they are.

## Do not

- Recolour the vial, add a gradient, outline or shadow.
- Move the bubble off centre: a level mark that is not level.
- Put the light variant on a dark page or the reverse.
- Set the wordmark in another font or redraw the underscore as a plain `_`.
- Stretch, rotate or crop the mark.
