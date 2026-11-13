## Why

A build made from a working tree with uncommitted changes (a quick local fix, a debug print left in) looks identical to a clean release build once it's running, unless the firmware itself carries and reports a dirty flag — otherwise "what's actually on this board" becomes a guess from memory.

## Example

This repo's own `bedrock_build_info::BedrockBuildInfo` carries a `dirty: bool` field alongside the git SHA (BLD-1), embedded at build time (BLD-2) and readable back off the device; the Notion firmware checklist item this check is imported from asks for exactly this (a dirty build must say so, not just report a SHA that looks like a clean commit).

## How to fix

Always embed build info with a dirty flag derived from `git status --porcelain` at build time (the pattern the CLI `--version` convention in AGENTS.md already uses: SHA plus `-dirty`), and have host tooling treat a dirty build as worth flagging, not silently trusted the way a clean-SHA build is.

## How it is checked

Check: cfg easy — a lint checks that a firmware depending on `bedrock_build`/`bedrock_build_info` actually turns on the feature that embeds this (CFG-7 catches `--build-info` left off as much as a missing dirty bit).

## Source

Ported from embedded_bedrock's firmware check catalogue, `CFG-7` (`book/checks.md`, detail in `book/checks/cfg.md`). Found on: notion; embedded_bedrock BLD.
