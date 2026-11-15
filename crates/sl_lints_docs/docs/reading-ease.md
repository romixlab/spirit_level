## Why

Long sentences of long words are read slowly and understood less. The Flesch reading ease score measures both: 206.835 - 1.015 × words per sentence - 84.6 × syllables per word. Plain English scores 60 to 70, technical writing 30 to 50, and under 30 most readers need several passes.

The score is an estimate: syllables are counted by a spelling heuristic, and names and code in the prose lower it. `spl pages` shows each page's score. Pages with fewer sentences than `min-sentences` are not judged, and only pages in English.

## How to fix

Split long sentences: one idea each. Use the short word where it says the same, "use" for "utilise", "start" for "initialise". Move lists of names and options into a list or a table, where they are not prose.
