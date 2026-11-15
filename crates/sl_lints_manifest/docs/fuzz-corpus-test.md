## Why

A fuzzer earns nothing if the inputs it found are thrown away. SQLite and TigerBeetle keep their interesting fuzz
cases as files and replay them on every ordinary test run, so a bug a fuzzer took hours to find never comes back
silently. A `fuzz/fuzz_targets/*.rs` with no corpus in git, or no test replaying it, has already lost that
regression suite: `cargo fuzz run` alone needs a fuzzing engine nobody runs by hand before every commit.

## How to fix

- Commit a handful of minimised seed inputs under `fuzz/corpus/<target>/` (the file names cargo-fuzz itself
  would write).
- Add an ordinary `#[test]` whose name mentions the target, reading every file in that corpus directory and
  calling the target's entry function directly, no fuzzing engine involved:

```rust
#[test]
fn replays_parse_bytes_corpus() {
    for entry in std::fs::read_dir("fuzz/corpus/parse_bytes").unwrap() {
        let data = std::fs::read(entry.unwrap().path()).unwrap();
        parse_bytes::run(&data);
    }
}
```

The lint only checks that a corpus directory is tracked and that some `#[test]` names the target; it cannot see
whether the test actually replays the corpus.
