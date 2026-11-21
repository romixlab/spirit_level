## Why

Any value that can reach `unwrap`, `expect`, slice indexing, `unreachable!` or an arithmetic op that can overflow, and that traces back to host input (a pin number, a length, a rate, a count field), is a denial-of-service the host side doesn't even need to be malicious to hit — a buggy or older host sending a malformed or truncated request is enough.

## Example

Typec_dongle's CMSIS-DAP path (FW-19) had exactly this: `dap_rs` indexes past the end of a truncated request, so a request cut short by the host panicked the firmware. usb_io_fw's `AGENTS.md` states the rule plainly: "No panics on host input: every value from the API is validated (pin numbers, voltage levels, baud rates) and rejected with an error the host can show."

## How to fix

Validate every host-supplied value at the boundary and return a typed error instead of indexing, unwrapping or computing on it directly; for a request whose own framing can be short (FW-19's fix), check the declared length against what actually arrived before parsing it as if it were complete.

## How it is checked

Src hard — finding every path from a host-controlled value to a panicking operation needs the call graph, not just a pattern match; `check: src hard` in the catalogue reflects that.

## Source

Ported from embedded_bedrock's firmware check catalogue, `HOST-1` (`book/checks.md`, detail in `book/checks/host.md`). Found on: usb_io_fw AGENTS.md; portwright FW-19 (dap_rs indexed past a truncated request).
