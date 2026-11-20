## Why

A clock-configuration change that alters the RTC's clock source, in a firmware that also keeps state in the backup domain (`--bkp-counters`, or RTC-backed timekeeping) across resets.

Changing the RTC clock source is exactly the kind of backup-domain change embassy itself handles by resetting the domain — which, per `pwr-3`'s logic, is the right move when nothing needs to survive, and the wrong move here: it wipes the counters or time state the firmware was relying on surviving the reset that just happened.

## Example

Portwright_template's `main.rs` needed this worked out explicitly: a clock-config edit that touched the RTC source reset backup-domain state nobody meant to lose.

## How to fix

Keep the RTC clock source stable once the firmware starts relying on backup-domain persistence; if it must change, treat it as a deliberate one-time reset of that state, not a side effect to discover later.

## How it is checked

`src mid`: a clock-config change affecting the RTC source, in a firmware model that also shows backup-register or RTC-backed persistent use, is the pattern to flag.

## Source

Ported from embedded_bedrock's firmware check catalogue, `PWR-4` (`book/checks.md`, detail in `book/checks/pwr.md`). Found on: portwright_template main.rs.
