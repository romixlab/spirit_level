## Why

Every PC, every session and the gate build with the same compiler only when every `rust-toolchain.toml` names one exact toolchain. A channel such as `stable` or `nightly`, or a release without its patch number (`1.100`), resolves to whatever the PC last installed, so a new Clippy lint or a changed inference shows up on one machine only.

The lint checks every `rust-toolchain.toml` and legacy `rust-toolchain` file the repo tracks, nested ones too (`firmware/`, `examples_mcu/`), except test projects (`skip`). Each pins one of the expected pair:

| Parameter | Default | Pinned where |
|---|---|---|
| `stable` | the release current when this spl was released | every toolchain file without a nightly need |
| `nightly` | the dated nightly current then | a toolchain file that needs a nightly |

Each spl release ships the current pair, so moving every repo to a new compiler is one spl release and `spl check --fix` in each repo.

## How to fix

```toml
[toolchain]
channel = "1.100.0"
components = ["clippy", "rustfmt"]
```

`spl check --fix` writes the expected toolchain: the nightly in place of any nightly, the stable release in place of anything else, and `channel` into a `[toolchain]` that has none.

A repo that must stay on another toolchain sets the pair, with the reason:

```toml
[lints.toolchain-pinned]
stable = "1.99.0"
reason = "the vendor SDK breaks on 1.100; back to the default with its 4.2 release"
expires = "2027-03-31"
```
