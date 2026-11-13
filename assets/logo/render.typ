// One variant of the logo on a page of its own size: `typst compile --input kind=mark --input theme=dark ...`.
// kind: mark, horizontal (mark beside the wordmark), stacked (mark above it), icon (the mark on the dark tile),
// icon-small (one colour, for 16 to 32 px), social (a 1280 × 640 card).
// theme: light (for light pages), dark (for dark pages) or mono (one colour); icon, icon-small and social ignore it.
// size: the longer side in pt, which is pixels in a PNG at `--ppi 72`; empty or absent: the drawing's own size.
#import "logo.typ": *

#let kind = sys.inputs.at("kind", default: "horizontal")
#let theme = sys.inputs.at("theme", default: "light")
#let size = sys.inputs.at("size", default: "")

#let art = if kind == "mark" { mark(theme: theme) } else if kind == "horizontal" {
  horizontal(theme: theme)
} else if kind == "stacked" { stacked(theme: theme) } else if kind == "icon" { icon() } else if kind == "icon-small" {
  icon-small()
} else if kind == "social" {
  // The horizontal lockup on the tile colour, the tagline under it.
  block(width: 1280pt, height: 640pt, fill: tile-fill, align(center + horizon, stack(spacing: 56pt,
    scale(78%, reflow: true, horizontal(theme: "dark")),
    text(..font, weight: "medium", size: 40pt, fill: muted)[A linter for Rust projects of every size],
  )))
} else { panic("kind: mark, horizontal, stacked, icon, icon-small or social, not " + kind) }

#set page(width: auto, height: auto, margin: 0pt, fill: none)
#context {
  let natural = measure(art)
  let f = if size == "" { 1 } else { float(size) * 1pt / calc.max(natural.width, natural.height) }
  scale(f * 100%, origin: top + left, reflow: true, art)
}
