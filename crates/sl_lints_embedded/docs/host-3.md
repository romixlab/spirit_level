## Why

A host built against a newer wire protocol can send an enum variant this firmware doesn't know about. Matching it with a catch-all arm (`_ => some_default`) makes the firmware silently *do something else* than what the host asked for, instead of telling the host its request wasn't understood.

## Example

This is a general wire-compatibility trap anywhere a request carries an enum the firmware dispatches on — usb_io_fw's diag API (API-5) is exactly the shape of surface (many small enum-tagged requests, built and evolved independently of any one host) where a newer host's added variant would otherwise land on whatever arm a `_ =>` catch-all happens to pick.

## How to fix

Refuse an enum value the firmware doesn't recognize with a typed "unknown/unsupported" error instead of mapping it to any existing variant; WireWeaver's evolution rules exist so a protocol can grow a variant without old firmware needing to guess at it.

## How it is checked

Src mid — needs the dispatch model (which match arms exist, which is the catch-all, whether the catch-all returns an error or a default) rather than a single pattern.

## Source

Ported from embedded_bedrock's firmware check catalogue, `HOST-3` (`book/checks.md`, detail in `book/checks/host.md`). Found on: usb_io_fw API-5.
