## Why

Two independent pieces of code — a firmware updater and a settings store, most often — each taking their own `FLASH` peripheral handle instead of sharing one owner.

Flash programming on most of these parts is a shared, stateful peripheral (one set of control registers, one busy flag); two independent owners racing an erase and a write against each other corrupts whichever one loses, and because both "succeed" from their own point of view, the corruption surfaces later, far from the code that caused it.

## Example

Embedded_bedrock TPL-24: the updater and the settings store both wanting `FLASH` directly is exactly the shape this check exists for.

## How to fix

Give flash access to one owner (a flash-manager task, or the updater and settings sharing a single handle behind a mutex) and route every consumer through it.

## How it is checked

`src mid`: the firmware model should show exactly one place that owns the `FLASH` peripheral singleton; any second independent acquisition is the finding.

## Source

Ported from embedded_bedrock's firmware check catalogue, `FLS-2` (`book/checks.md`, detail in `book/checks/fls.md`). Found on: embedded_bedrock TPL-24.
