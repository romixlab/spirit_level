## Why

A guide explains how something works: a layout, a flow, a sequence of states. Text describes such a mechanism one sentence at a time, and the reader has to build the picture in their head. A figure shows it whole. Without one, a long guide reads as a wall of text however it is broken up, and the reader skims it.

A good figure shows the mechanism the text around it describes: the blocks of a file and where each field sits, the path of a value through the stages, a level on a scale next to the bits that store it. It earns its place when the text points at it and the reader understands the text faster for it. A logo, a stock picture or an icon is decoration and helps nobody.

A figure is one of these:

- a paragraph whose first line holds only images, with or without an attribute list (`{ width="300" }`); the lines under it are its caption, read as prose;
- an image in a table cell;
- an HTML `<figure>` (once, whatever it holds), `<img>`, `<svg>`, or `<object>` showing an image;
- a diagram block: a fenced block in `mermaid`, `dot`, `graphviz`, `plantuml`, `puml`, `d2`, `svgbob`, `bob`, `wavedrom` or `pikchr`.

A badge is not a figure: a row of build and version badges shows no mechanism. An image is a badge when its URL is a badge service's (`img.shields.io`, `badgen.net`, `badge.fury.io`) or a badge endpoint's (a path ending in `badge.svg`, or holding `/badge/`), or when it sits inside a link to a page. An image linked to an image file, its own or a larger one, stays a figure. `badges` adds URL parts for another service; `figures` names URL parts that keep an image a figure whatever else its URL holds, such as a folder of diagrams under a `/badge/` path. An `<a>` without `href` is an anchor, not a link. An image in a heading, Markdown or HTML, markup inside an HTML comment, a table, a code block and a terminal capture are not figures either.

A page with up to `free-words` words of prose needs no figure. Above that it needs one figure for each `words-per-figure` words begun. By default only guides are judged: a reference page is looked up, not read through. A record (a changelog, a feature tracker, an agent file) is never judged, also under `pages = "all"`.

## The defaults

The defaults come from two books: the decibels page of exact_units, written with figures in mind, and the guide of ww_tape, written without. The page numbers are prose words; "needed" uses the defaults (500 free, 600 per figure).

| Page | Prose words | Figures | Needed | Result |
|---|---:|---:|---:|---|
| exact_units `levels/decibels.md` | 1384 | 2 | 2 | pass |
| ww_tape `concepts.md` (7dbddf7) | 2550 | 2 | 4 | fail |
| ww_tape `format.md` (7dbddf7) | 7062 | 1 | 11 | fail |
| ww_tape `index.md` (7dbddf7) | 3378 | 0 | 5 | fail |
| ww_tape `numbers.md` (7dbddf7) | 1223 | 0 | 2 | fail |
| ww_tape `logring.md` (7dbddf7) | 519 | 1 | 1 | pass |
| ww_tape, 10 pages (7dbddf7) | 26855 | 10 | 41 | 9 of 10 fail |
| ww_tape, 21 guides (13a31be6, split into shorter pages) | 26978 | 8 | 38 | 18 of 21 fail |

Splitting ww_tape into shorter pages did not change the picture: 13 of its 21 guides have no figure.

Every row is measured with the defaults and the repository's own `spirit_level.toml`. ww_tape at 7dbddf7 has no `[docs]` table, so all its pages are guides; at 13a31be6 `docs/format.md` is a reference page and is not judged, which leaves 21 guides.

## How to fix

Find what the text explains step by step and draw it: the layout of a block, the order of calls, the states and the moves between them. Put the figure next to the text that describes it and refer to it. A diagram block (```` ```mermaid ````) keeps the figure in the source and in review.

A page of lookups (options, error codes, a list of lints) is a reference page: list it under `[docs] reference` and it is not judged.
