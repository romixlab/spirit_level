// The spirit_level logo: the mark (a level's vial, its bubble at rest between the two lines) and the wordmark.
// The vial is the spirit, the bubble is a hole in it: whatever is behind the mark shows through, on any page.
// render.typ picks a variant and a size; `just logo` exports them all.

#let themes = (
  // ink: the two lines and the lettering; spirit: the vial and the underscore
  light: (ink: rgb("#373c42"), spirit: rgb("#65a30d")),
  dark: (ink: rgb("#dddddd"), spirit: rgb("#a3e635")),
  // one colour; `just logo` turns it into currentColor
  mono: (ink: rgb("#000000"), spirit: rgb("#000000")),
)
// The icon's tile and the social card are dark, whatever the page around them.
#let tile-fill = rgb("#0f0f10")
#let muted = rgb("#a6a8aa")

// A stadium centred on (cx, cy), w wide and h high (a circle when w == h), as curve parts.
#let kappa = 0.5523
#let stadium(cx, cy, w, h) = {
  let r = h / 2
  let (l, rt, t, b) = (cx - w / 2 + r, cx + w / 2 - r, cy - r, cy + r)
  let k = kappa * r
  (
    curve.move((l, t)),
    curve.line((rt, t)),
    curve.cubic((rt + k, t), (rt + r, cy - k), (rt + r, cy)),
    curve.cubic((rt + r, cy + k), (rt + k, b), (rt, b)),
    curve.line((l, b)),
    curve.cubic((l - k, b), (l - r, cy + k), (l - r, cy)),
    curve.cubic((l - r, cy - k), (l - k, t), (l, t)),
    curve.close(),
  )
}

// One geometry per size class. The large one is for 48 px and up; the small one sits on a 32 grid, so every edge
// lands on a pixel at 16 and 32 px.
#let large = (
  width: 224pt, vial-h: 88pt,
  bubble-w: 72pt, bubble-h: 40pt, film: 10pt, // film: the spirit above the risen bubble
  line-w: 14pt, line-len: 30pt, line-gap: 12pt, line-in: 10pt, // line-in: from the bubble's end to the line
)
#let small = (
  width: 32pt, vial-h: 12pt,
  bubble-w: 12pt, bubble-h: 6pt, film: 2pt,
  line-w: 2pt, line-len: 8pt, line-gap: 2pt, line-in: 2pt,
)
#let mark-size(g) = (width: g.width, height: g.vial-h + 2 * (g.line-gap + g.line-len))

// The mark alone, tight to its shapes, transparent around them.
#let mark(theme: "light", geometry: large, ink: auto) = {
  let g = geometry
  let t = themes.at(theme)
  let ink = if ink == auto { t.ink } else { ink }
  let (w, h) = (g.width, mark-size(g).height)
  block(width: w, height: h, {
    let (cx, cy) = (w / 2, h / 2)
    let bubble-y = cy - g.vial-h / 2 + g.film + g.bubble-h / 2
    place(top + left, curve(fill: t.spirit, fill-rule: "even-odd", stroke: none,
      ..stadium(cx, cy, g.width, g.vial-h), ..stadium(cx, bubble-y, g.bubble-w, g.bubble-h)))
    for side in (-1, 1) {
      let x = cx + side * (g.bubble-w / 2 + g.line-in + g.line-w / 2) - g.line-w / 2
      for y in (0pt, h - g.line-len) {
        place(top + left, dx: x, dy: y, rect(width: g.line-w, height: g.line-len, fill: ink, stroke: none))
      }
    }
  })
}

// The wordmark: Montserrat SemiBold, its round letters echo the bubble; the underscore is a small vial.
#let font = (font: "Montserrat", weight: "semibold")
#let word-size = 160pt
#let wordmark(theme: "light", ink: auto) = {
  let t = themes.at(theme)
  let ink = if ink == auto { t.ink } else { ink }
  set text(..font, size: word-size, fill: ink, top-edge: "ascender", bottom-edge: "descender")
  // The underscore keeps the font's own advance and depth; its bar becomes a stadium of the same thickness.
  let bar-h = 0.1 * word-size
  let bar = box(width: 0.56 * word-size, height: bar-h, baseline: 0.15 * word-size, radius: bar-h / 2, fill: t.spirit)
  block([spirit#h(0.05em)#bar#h(0.05em)level])
}

// The mark beside the wordmark, its vial centred on the lowercase letters.
#let lockup-gap = 72pt
#let mark-scale = 100% // the mark's height against the wordmark's line height
#let horizontal(theme: "light", ink: auto) = context {
  let w = wordmark(theme: theme, ink: ink)
  let ws = measure(w)
  let ms = mark-size(large)
  let f = ws.height * mark-scale / ms.height
  // x-height of Montserrat is 0.519 em, its ascender 0.968 em: the vial sits on the middle of the x-height
  let mid = 0.968 * word-size - 0.26 * word-size
  let mark-top = mid - ms.height * f / 2
  let top-pad = calc.max(0pt, -mark-top)
  block(width: ms.width * f + lockup-gap + ws.width, height: calc.max(ws.height, mark-top + ms.height * f) + top-pad, {
    place(top + left, dy: mark-top + top-pad, scale(f * 100%, origin: top + left, reflow: true, mark(theme: theme, ink: ink)))
    place(top + left, dx: ms.width * f + lockup-gap, dy: top-pad, w)
  })
}

// The mark above the wordmark.
#let stack-gap = 40pt
#let stacked(theme: "light", ink: auto) = context {
  let w = wordmark(theme: theme, ink: ink)
  let ws = measure(w)
  let ms = mark-size(large)
  let f = 1.8
  block(width: calc.max(ws.width, ms.width * f), height: ms.height * f + stack-gap + ws.height, {
    place(top + center, scale(f * 100%, reflow: true, mark(theme: theme, ink: ink)))
    place(bottom + center, w)
  })
}

// The app icon: the dark mark on the dark tile, for avatars, apple-touch and large sizes on any background.
#let icon-size = 512pt
#let icon() = block(width: icon-size, height: icon-size, radius: 22%, fill: tile-fill, clip: true,
  align(center + horizon, scale(icon-size * 0.74 / large.width * 100%, reflow: true, mark(theme: "dark"))))

// The small icon for 16 to 32 px: the small geometry, one colour (the light spirit, 3:1 on white and 5:1 on dark),
// no tile, so the vial spans the whole square and reads on light and dark tab bars alike.
#let small-size = 32pt
#let icon-small() = block(width: small-size, height: small-size,
  place(top + left, dx: (small-size - small.width) / 2, dy: (small-size - mark-size(small).height) / 2,
    mark(geometry: small, theme: "light", ink: themes.light.spirit)))
