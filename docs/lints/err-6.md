# err-6

**Rule:** RP2350 A2 pads with a pull-down enable their input only for the read.

| | |
|---|---|
| Category | [err](index.md#err) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | off |  |
| [big](../tiers/big.md) | off |  |
| [ai](../tiers/ai.md) | off |  |
| [embedded](../tiers/embedded.md) | `warn` | bedrock warning: RP2350 A2 pad with pull-down and input buffer left on (RP2350-E9: the pin stays high): enable the input only for the read |

## Why

**RP2350-E9**, Appendix D.5.1 ("Increased leakage current on Bank 0 GPIO when pad input is enabled"), affects RP2350 **A2** only (fixed in A3):

> When the pad is set as an input (input enable is enabled and output enable is disabled) and the voltage on the
> pad is within the undefined logic region, the leakage current exceeds the standard specified IIN leakage
> level... the pad can source current (typically around 120 μA)... This leakage will hold the pad at around
> 2.2 V... The pad pull-down (if enabled) is significantly weaker than the leakage current in this state and
> therefore isn't strong enough to pull the pad voltage low.

Workaround:

> If pad pull-down behaviour is required, clear the pad input enable in GPIO0.IE (for GPIOs 0 through 47) to
> ensure that the pad pull-down resistor pulls the pad signal low. To read the state of a pad pulled-down GPIO
> from software, enable the input buffer by setting GPIO0.IE immediately before reading, and then re-disable
> immediately afterwards.

## Example

Report_b200a's user button (SW101, GP22, no external pull-down on the board — the firmware relies on the internal one) hits this exactly: with the input buffer left on between reads, the pin can stay held near 2.2 V (a logic high) after release instead of returning to the pulled-down low. The board's rev A/A4 build (`TPL-48`'s pin handling, matching `b200_board/src/pin_diag.rs`) enables the input buffer only for the instant of each read and disables it again immediately after, exactly as the workaround prescribes; Rev B of that board adds an external pull-down instead, which sidesteps the erratum rather than working around it.

## How to fix

On any RP2350 A2 pad using an internal pull-down as an input, enable `GPIOn.IE` only for the duration of the read and disable it again right after; don't leave input-enable on between reads. An external pull-down of 8.2 kΩ or less also works per the sheet, and removes the need for the read-time toggle entirely.

## How it is checked

Check: src mid · chip: RP2350 A2 — needs recognizing a pad configured `Input` + pull-down that stays input-enabled across more than the read itself.

## Source

Ported from embedded_bedrock's firmware check catalogue, `ERR-6` (`book/checks.md`, detail in `book/checks/err.md`). Found on: report_template; embedded_bedrock TPL-48.

## Configure

```toml
[lints.err-6]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(err-6) <reason>` ([waivers](../waivers.md)).
