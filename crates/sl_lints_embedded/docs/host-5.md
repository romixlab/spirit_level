## Why

A request shorter than its own command needs, or a response the firmware is asked to produce that is larger than the protocol's packet limit, has to be refused — but if it's refused silently, a host sending malformed traffic (an older host against a newer firmware, or a genuinely buggy one) leaves no trace to diagnose with.

## Example

Typec_dongle's CMSIS-DAP path (FW-19) counts every one of these distinctly: `dap_requests_truncated` for a request cut short, `dap_requests_too_long` for one longer than the 512-byte packet limit, and `dap_responses_too_long` for a response that would exceed it — each dropped without a panic, but each counted so the pattern is visible without a debugger attached.

## How to fix

Give every "this request/response doesn't fit the protocol" rejection its own `cnt!` counter (CNT-1), distinct from a generic error count, so a host sending malformed traffic shows up in `device.counters` instead of silently failing.

## How it is checked

Src mid — a lint can check that a request-validation early-return path reaches a `cnt!` call, though telling truncated from oversized from malformed needs the handler's own branching.

## Source

Ported from embedded_bedrock's firmware check catalogue, `HOST-5` (`book/checks.md`, detail in `book/checks/host.md`). Found on: portwright FW-19.
