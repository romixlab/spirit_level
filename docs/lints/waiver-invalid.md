# waiver-invalid

**Rule:** A waiver comment names known lints, gives a reason and has not expired.

| | |
|---|---|
| Category | [waivers](index.md#waivers) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | `deny` | a waiver without a reason, for an unknown lint, or expired hides a finding without saying why |
| [small-workspace](../tiers/small-workspace.md) | `deny` | a waiver without a reason, for an unknown lint, or expired hides a finding without saying why |
| [medium](../tiers/medium.md) | `deny` | a waiver without a reason, for an unknown lint, or expired hides a finding without saying why |
| [big](../tiers/big.md) | `deny` | a waiver without a reason, for an unknown lint, or expired hides a finding without saying why |
| [ai](../tiers/ai.md) | off |  |
| [embedded](../tiers/embedded.md) | off |  |

## Why

A waiver is how a project says "this finding is acceptable here, and this is why". A waiver without a reason hides the why; one that names a lint that does not exist (a typo, a lint that was renamed) allows nothing and hides that too; one whose expiry has passed has done its job. All three are reported so the waiver comments stay true.

## The form

```rust
// spirit-level: allow(static-interior-mutability) K: the signal handler's flag, set from one thread
static STOP: AtomicBool = AtomicBool::new(false);

let v = parse(x).unwrap(); // spirit-level: allow(panic-in-library) the grammar guarantees a number here
```

In TOML files the comment starts with `#`. An expiry goes right after the list: `allow(crate-lines) expires=2027-03-31 split tracked in FEATURES SPLIT-2`, or a release, `expires=v0.5.0`.

## How to fix

Give the reason, correct the lint id (`spirit-level lints` lists them), or, for an expired waiver, fix what it allowed and remove it (or renew it with a new reason and a new expiry).

## Configure

```toml
[lints.waiver-invalid]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(waiver-invalid) <reason>` ([waivers](../waivers.md)).
