# Tapes

A tape is written in [[pieces]]. Each piece ends in its [[content hash]], which a reader checks.

The clock of a tape counts time. The term `piece` in code is not prose.

## Resuming

After a crash the writer reopens [[the last file|piece]], and every later piece goes on from it.
