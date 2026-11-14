## Why

A bug is fixed only when a test that fails without the fix is in the suite: the suite then remembers the mistake, and the fix stays fixed. Writing the failing test first also shows that the fix repairs the bug that was seen.

A commit counts as a fix when it has a `Fixes:` trailer ([fixes-tag](P0017.md)) or its subject opens with a fix word (`Fix`, `Fixes`, `Fixed`). It needs a test when it changes code (`**/*.rs` outside test paths). It has one when it changes a file under a test path (`tests/`, `benches/`, `fuzz/`) or adds a line with a test marker (`#[test]`, `#[tokio::test]`, `proptest!`). The judgement from the subject is an estimate, so the finding says which rule made the commit a fix.

## How to fix

Add the test to the same commit: write it first, see it fail, then fix. Where no test can be written (the bug needs hardware, a race no test reaches), say so in a trailer: `Spirit-Level: allow(fix-without-test) needs the board on the bench`.
