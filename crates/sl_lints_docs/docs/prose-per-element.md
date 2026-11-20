## Why

`paragraph-run-words` finds one long stretch of paragraphs. This lint judges the whole page: how much prose a reader gets through for each thing that is not a sentence. A guide that explains well alternates: a paragraph, then an example, a figure, a table of values, a list of cases. A page that is mostly sentences, however it is cut into sections, is skimmed.

An element is a list (its items in a row count once), a figure, a table, a code block, an admonition, a content tab or another visual break. Headings are not elements. The measure is the page's prose words divided by its elements, at least one. Pages with fewer than `min-words` words of prose are not judged, and by default only guides are. A record (a changelog, a feature tracker, an agent file) is never judged, also under `pages = "all"`.

## The defaults

The default comes from the same books as `page-figures` and `paragraph-run-words`: the decibels page of exact_units, and the guide of ww_tape.

| Pages | Prose words per element | Over 120 |
|---|---|---|
| exact_units `levels/decibels.md` | 58 | no |
| exact_units, 75 pages | 9 to 324 | 15 of 75 |
| ww_tape, 10 pages (7dbddf7) | 104 to 483 | 9 of 10 |
| ww_tape, 21 guides (13a31be6, split into shorter pages) | 126 to 669 | 21 of 21 |
| logring, 3 guides (7a2fdb0) | 107 to 314 | 2 of 3 |

Every row is measured with the defaults and the repository's own `spirit_level.toml`. ww_tape at 7dbddf7 has no `[docs]` table, so all its pages are guides; at 13a31be6 `docs/format.md` is a reference page and is not judged, which leaves 21 guides. logring's `docs/format.md` is a reference page there.

## How to fix

Find what the prose describes and show it: a figure of the mechanism, a table where values are compared, a list where facts are parallel, an example in code where the reader will write code. Turning a paragraph into a list only helps when its sentences really are parallel items.
