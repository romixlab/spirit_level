## Why

A list cut for room that ends in "3 more" or "…and 12 others" hides the items the person may need, and gives no way to see them. The practice: the rest goes under a fold, collapsed by default, whose header says what is inside and which opens to every item.

The lint reports a string literal in a [UI crate](../ui.md#which-crates-are-ui-crates) where a number or a format placeholder is followed by one of `words` (`more`, `others`): `"{} more"`, `"and {n} others"`. Text in the arguments of `fold-calls` is a fold's header, which says what it holds ("3 more subagents, not busy"), and is not reported. Findings are estimates: the text may be a count with its own way to see the items. Two forms are fine where vhrd_egui allows them: a compact size of a widget that shows a count whose tooltip lists the items, and a `+N` chip in a cell of fixed width.

## How to fix

End the list in a fold (vhrd_egui: `ve_basics::fold_rest`, or `ScrollStrip` for a row of chips). A bot or a CLI offers the rest as a button or `--all`.
