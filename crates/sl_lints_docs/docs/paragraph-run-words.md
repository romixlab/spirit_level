## Why

A page can pass every length limit and still be a wall of text. Cut a long run of prose into short paragraphs, put a heading over every few of them, and `prose-run-words` and `paragraph-sentences` are satisfied, while the reader still meets nothing but sentences for pages on end. Each paragraph states one fact, and the reader has to hold them all to see how they connect.

This lint measures what the reader actually meets: the words of paragraphs in a row with no list, figure, table, code block or admonition between them. A heading does not end the run, because a heading gives the eye a rest but shows nothing. List items are not counted as paragraphs: a list ends the run.

By default only guides are judged.

## The defaults

The default comes from two books: the decibels page of exact_units, written with figures, tables and examples between its paragraphs, and the guide of ww_tape, written as prose under many headings. The longest run of each page is shown; the budget is 350 words.

| Pages | Longest run, words | Over 350 |
|---|---|---|
| exact_units `levels/decibels.md` | 156 | no |
| exact_units, 75 pages | 9 to 382 | 1 of 75 |
| ww_tape, 10 pages (7dbddf7) | 240 to 1654 | 9 of 10 |
| ww_tape, 22 pages (13a31be6, split into shorter pages) | 109 to 669 | 14 of 22 |
| logring `format.md` (7a2fdb0) | 475 | yes |

The other measures tried separate the books less well. The median words per paragraph is higher in ww_tape (46 to 80) than in decibels (29), so a limit on it would reward chopping. Paragraphs per heading are the same in both (about 3). Prose words per list, figure, table or code block separate them (58 against 104 to 483) but point at no place on the page.

## How to fix

Give the reader something other than sentences where the run is long: a figure of the mechanism the text describes, a table for a comparison, a list for parallel facts, an example in code. More headings or shorter paragraphs do not end the run.
