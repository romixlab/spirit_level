# Lints

Every lint, grouped by category, with its stable code and its level in each shipped tier (`off`: the tier does not enable it). A project changes any of them in its `spirit_level.toml` ([configuration](../configuration.md)).

## agents

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [P0001](P0001.md) | [agent-file-lines](P0001.md) | an agent file stays under its line limit | warn | warn | warn | warn | deny | off | off | off | off | off |
| [P0002](P0002.md) | [agent-file-links](P0002.md) | links and @imports in agent files resolve | off | off | off | off | deny | off | off | off | off | off |
| [P0003](P0003.md) | [agent-rules-match-lints](P0003.md) | limits an agent file states are the limits the lints check | off | off | off | warn | deny | off | off | off | off | off |
| [P0004](P0004.md) | [claude-md-duplicate](P0004.md) | CLAUDE.md points to AGENTS.md instead of copying it | off | off | off | off | deny | off | off | off | off | off |

## api

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [C0001](C0001.md) | [pub-surface](C0001.md) | a library's public surface stays small for its size | off | off | off | warn | off | off | off | off | off | off |

## audit

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [X0001](X0001.md) | [audit-unmaintained](X0001.md) | no dependency is marked unmaintained or replaced in the RustSec database | off | off | off | off | off | off | off | off | off | off |
| [X0002](X0002.md) | [audit-unsound](X0002.md) | no dependency is marked unsound in the RustSec database | off | off | off | off | off | off | off | off | off | off |
| [X0003](X0003.md) | [audit-vulnerabilities](X0003.md) | no dependency has a published security vulnerability | off | off | off | off | off | off | off | off | off | off |
| [X0004](X0004.md) | [audit-yanked](X0004.md) | Cargo.lock holds no yanked version | off | off | off | off | off | off | off | off | off | off |

## book

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [D0008](D0008.md) | [nav-entries](D0008.md) | the book's navigation names pages that exist | deny | deny | deny | deny | off | off | off | off | off | off |
| [D0009](D0009.md) | [orphan-pages](D0009.md) | every page of the book is in the navigation or linked from another page | warn | warn | warn | warn | off | off | off | off | off | off |
| [D0010](D0010.md) | [page-title](D0010.md) | every page of the book has one title | warn | warn | warn | warn | off | off | off | off | off | off |
| [D0011](D0011.md) | [admonition-syntax](D0011.md) | admonitions use a type Zensical styles and a body indented four spaces | deny | deny | deny | deny | off | off | off | off | off | off |
| [D0017](D0017.md) | [book-build-stamp](D0017.md) | the book's footer shows the crate's version, commit and build age | warn | warn | warn | warn | off | off | off | off | off | off |
| [D0018](D0018.md) | [readme-index-clash](D0018.md) | no docs folder holds both README.md and index.md | deny | deny | deny | deny | off | off | off | off | off | off |
| [D0019](D0019.md) | [theme-feature-conflicts](D0019.md) | theme features that exclude each other are not both on | deny | deny | deny | deny | off | off | off | off | off | off |
| [D0020](D0020.md) | [zensical-keys](D0020.md) | zensical.toml names only keys Zensical reads | deny | deny | deny | deny | off | off | off | off | off | off |

## boot

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [F0001](F0001.md) | [boot-1](F0001.md) | a firmware entered by a jump detects it and resets once | off | off | off | off | off | deny | off | off | off | off |
| [F0002](F0002.md) | [boot-10](F0002.md) | backup-domain helpers restore what they found | off | off | off | off | off | warn | off | off | off | off |
| [F0003](F0003.md) | [boot-2](F0003.md) | the H7 supply configuration is read back, never written blind | off | off | off | off | off | deny | off | off | off | off |

## brd

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [F0099](F0099.md) | [brd-1](F0099.md) | a board description reads, follows the schema, and the chip data and PCB it names read | off | off | off | off | off | deny | off | off | off | off |
| [F0100](F0100.md) | [brd-2](F0100.md) | every pin of a board description has one job in each configuration | off | off | off | off | off | deny | off | off | off | off |
| [F0101](F0101.md) | [brd-3](F0101.md) | of MCU pins joined on the board, at most one is used | off | off | off | off | off | deny | off | off | off | off |
| [F0102](F0102.md) | [brd-4](F0102.md) | the pins, package, peripherals and signals a board description names exist on its chip | off | off | off | off | off | deny | off | off | off | off |
| [F0103](F0103.md) | [brd-5](F0103.md) | every peripheral signal sits on a pin that can carry it | off | off | off | off | off | deny | off | off | off | off |
| [F0104](F0104.md) | [brd-6](F0104.md) | the MCU pins are connected on the PCB as the board description says | off | off | off | off | off | deny | off | off | off | off |
| [F0105](F0105.md) | [brd-7](F0105.md) | every connected MCU pin of the PCB is in the board description, and no free pin is unconnected | off | off | off | off | off | warn | off | off | off | off |

## build

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [X0005](X0005.md) | [cargo-doc](X0005.md) | cargo doc builds the workspace's documentation without a warning | warn | warn | deny | deny | off | off | off | off | off | off |
| [X0006](X0006.md) | [cargo-msrv](X0006.md) | every crate builds with the toolchain of its declared rust-version | off | off | warn | deny | off | off | off | off | off | off |
| [X0007](X0007.md) | [cargo-no-std](X0007.md) | every crate the map marks no_std builds for targets without std | off | off | off | deny | off | off | off | off | off | off |
| [X0008](X0008.md) | [cargo-wasm](X0008.md) | every crate the map marks wasm builds for wasm32-unknown-unknown | off | off | off | deny | off | off | off | off | off | off |

## cfg

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [F0004](F0004.md) | [cfg-1](F0004.md) | every probe-rs runner and recipe names its probe with --probe VID:PID | off | off | off | off | off | deny | off | off | off | off |
| [F0005](F0005.md) | [cfg-2](F0005.md) | F0/G0/L0 firmware is flashed by a runner that halts the core first | off | off | off | off | off | warn | off | off | off | off |
| [F0006](F0006.md) | [cfg-4](F0006.md) | every crate is in the tree from one source | off | off | off | off | off | deny | off | off | off | off |
| [F0007](F0007.md) | [cfg-5](F0007.md) | the HAL's chip feature names the part on the board | off | off | off | off | off | deny | off | off | off | off |
| [F0008](F0008.md) | [cfg-6](F0008.md) | thumbv6m firmware that needs compare-and-swap uses portable-atomic with critical-section | off | off | off | off | off | warn | off | off | off | off |
| [F0009](F0009.md) | [cfg-7](F0009.md) | the build info carries the git SHA and a dirty flag | off | off | off | off | off | warn | off | off | off | off |

## cli

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [C0020](C0020.md) | [cli-help-examples](C0020.md) | --help of a CLI and of each of its leaf commands ends in an Examples block | warn | warn | warn | warn | off | off | off | off | off | off |
| [C0021](C0021.md) | [cli-help-text](C0021.md) | every command and argument of a CLI has a help line | warn | warn | warn | warn | off | off | off | off | off | off |
| [C0022](C0022.md) | [cli-help-clean](C0022.md) | a CLI's help holds no rustdoc leftovers | warn | warn | deny | deny | off | off | off | off | off | off |
| [C0023](C0023.md) | [cli-completion](C0023.md) | a CLI prints its shell completion registration for bash, zsh and fish | warn | warn | warn | warn | off | off | off | off | off | off |
| [C0024](C0024.md) | [cli-plain-when-piped](C0024.md) | a CLI writes no colour escapes into a pipe | warn | warn | deny | deny | off | off | off | off | off | off |
| [C0025](C0025.md) | [cli-color-flag](C0025.md) | a CLI takes a global --color flag with auto, always and never | warn | warn | warn | warn | off | off | off | off | off | off |
| [C0026](C0026.md) | [cli-usage-error](C0026.md) | a CLI answers an unknown flag with exit code 2 and a message on stderr only | warn | warn | deny | deny | off | off | off | off | off | off |
| [C0027](C0027.md) | [cli-version-line](C0027.md) | --version prints the binary's name, version, git SHA and commit time | warn | warn | warn | warn | off | off | off | off | off | off |
| [C0028](C0028.md) | [cli-json-output](C0028.md) | every leaf command of a CLI has a machine-readable output flag | off | off | off | off | off | off | off | off | off | off |
| [C0029](C0029.md) | [cli-standard-flags](C0029.md) | a CLI's short flags keep their standard meaning | warn | warn | warn | warn | off | off | off | off | off | off |
| [C0037](C0037.md) | [cli-secret-flags](C0037.md) | no option takes a secret as its value | warn | warn | warn | warn | off | off | off | off | off | off |
| [C0038](C0038.md) | [cli-no-abbreviations](C0038.md) | a CLI runs no subcommand from a unique prefix | warn | warn | deny | deny | off | off | off | off | off | off |
| [C0039](C0039.md) | [command-registry](C0039.md) | a CLI with many commands registers each command group in its own module | warn | warn | warn | warn | off | off | off | off | off | off |

## clippy

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [X0009](X0009.md) | [clippy-build](X0009.md) | the workspace builds under clippy without compiler errors or warnings | off | off | off | off | off | off | off | off | off | off |
| [X0010](X0010.md) | [clippy-cargo](X0010.md) | no finding of the cargo-group clippy lints the project turned on | off | off | off | off | off | off | off | off | off | off |
| [X0011](X0011.md) | [clippy-complexity](X0011.md) | no finding of clippy's complexity group: simple things done simply | off | off | off | off | off | off | off | off | off | off |
| [X0012](X0012.md) | [clippy-correctness](X0012.md) | no finding of clippy's correctness group: code that is outright wrong | off | off | off | off | off | off | off | off | off | off |
| [X0013](X0013.md) | [clippy-nursery](X0013.md) | no finding of the nursery clippy lints the project turned on | off | off | off | off | off | off | off | off | off | off |
| [X0014](X0014.md) | [clippy-pedantic](X0014.md) | no finding of the pedantic clippy lints the project turned on | off | off | off | off | off | off | off | off | off | off |
| [X0015](X0015.md) | [clippy-perf](X0015.md) | no finding of clippy's perf group: no needlessly slow code | off | off | off | off | off | off | off | off | off | off |
| [X0016](X0016.md) | [clippy-restriction](X0016.md) | no finding of the restriction clippy lints the project turned on | off | off | off | off | off | off | off | off | off | off |
| [X0017](X0017.md) | [clippy-style](X0017.md) | no finding of clippy's style group: code written the idiomatic way | off | off | off | off | off | off | off | off | off | off |
| [X0018](X0018.md) | [clippy-suspicious](X0018.md) | no finding of clippy's suspicious group: code that is most likely wrong | off | off | off | off | off | off | off | off | off | off |

## clk

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [F0010](F0010.md) | [clk-2](F0010.md) | the embassy time driver names a timer that nothing else uses | off | off | off | off | off | deny | off | off | off | off |
| [F0011](F0011.md) | [clk-3](F0011.md) | the clock security system watches HSE and LSE, with a handler | off | off | off | off | off | warn | off | off | off | off |
| [F0012](F0012.md) | [clk-5](F0012.md) | USB has an exact 48 MHz clock | off | off | off | off | off | deny | off | off | off | off |

## cls

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [F0106](F0106.md) | [cls-1](F0106.md) | every file that holds an executor task has a task class in [embedded.classes] | off | off | off | off | off | warn | off | off | off | off |
| [F0107](F0107.md) | [cls-2](F0107.md) | a service module denies the Clippy lints that find panics at its top | off | off | off | off | off | deny | off | off | off | off |

## cnt

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [F0013](F0013.md) | [cnt-1](F0013.md) | every error branch counts what happened with cnt! | off | off | off | off | off | warn | off | off | off | off |
| [F0014](F0014.md) | [cnt-2](F0014.md) | counter buffers an interrupt updates are never borrowed mutably or read unaligned | off | off | off | off | off | warn | off | off | off | off |
| [F0015](F0015.md) | [cnt-3](F0015.md) | a proc-macro crate and its runtime crate are pinned to the same exact version | off | off | off | off | off | warn | off | off | off | off |

## code

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [C0002](C0002.md) | [allow-without-reason](C0002.md) | every allow or expect of a lint says why | warn | warn | warn | deny | deny | off | off | off | off | off |
| [C0003](C0003.md) | [cli-subprocess](C0003.md) | workspace crates call each other as libraries, never by running the workspace's own binaries | off | off | deny | deny | off | off | off | off | off | off |
| [C0004](C0004.md) | [float-determinism](C0004.md) | deterministic crates compute float transcendentals through libm | off | off | off | deny | off | off | off | off | off | off |
| [C0005](C0005.md) | [panic-in-library](C0005.md) | library code returns errors for what can fail and says why where it panics | off | off | warn | warn | off | deny | off | off | off | off |
| [C0006](C0006.md) | [print-in-library](C0006.md) | library code does not print | warn | warn | warn | warn | off | off | off | off | off | off |
| [C0007](C0007.md) | [random-state-hash](C0007.md) | deterministic crates use no randomly seeded hash maps | off | off | off | deny | off | off | off | off | off | off |
| [C0008](C0008.md) | [todo-macros](C0008.md) | no todo!() or unimplemented!() outside tests | warn | warn | warn | deny | deny | off | off | off | off | off |
| [C0014](C0014.md) | [build-info-env](C0014.md) | library code reads build info at run time, not through env! | off | off | warn | warn | off | off | off | off | off | off |
| [C0015](C0015.md) | [test-support-gated](C0015.md) | test fakes and testing modules compile only for tests or behind the test-support feature | off | off | warn | warn | off | off | off | off | off | off |
| [C0017](C0017.md) | [sized-ints-in-contracts](C0017.md) | serde types of a contract crate use sized integers, not usize or isize | off | off | warn | deny | off | off | off | off | off | off |
| [C0018](C0018.md) | [unit-suffix](C0018.md) | a numeric field or constant that holds a time or a size names its unit | off | off | off | off | warn | off | off | off | off | off |
| [C0019](C0019.md) | [unbounded-read](C0019.md) | a read to the end of a stream is bounded with take() | off | off | off | warn | warn | off | off | off | off | off |
| [C0030](C0030.md) | [compound-assert](C0030.md) | an assertion checks one condition, so a failure names the one that broke | off | off | off | warn | off | warn | off | off | off | off |
| [C0031](C0031.md) | [assert-message](C0031.md) | every assertion says in words what it checks | off | off | off | warn | off | warn | off | off | off | off |
| [C0032](C0032.md) | [float-partial-cmp-unwrap](C0032.md) | floats are ordered with total_cmp, not partial_cmp that panics on NaN | warn | warn | warn | warn | off | off | off | off | off | off |
| [C0040](C0040.md) | [build-script-clock](C0040.md) | build.rs reads no clock and resolves git through the real git dir | warn | warn | warn | warn | off | off | off | off | off | off |

## commits

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [P0005](P0005.md) | [commit-body-lines](P0005.md) | a commit body stays short | off | off | off | off | warn | off | off | off | off | off |
| [P0006](P0006.md) | [commit-prose](P0006.md) | commit messages use no filler, narration or emoji and do not describe the diff | off | off | off | off | warn | off | off | off | off | off |
| [P0007](P0007.md) | [commit-subject-length](P0007.md) | a commit subject stays under its length limit | off | off | warn | warn | deny | off | off | off | off | off |
| [P0017](P0017.md) | [fixes-tag](P0017.md) | a Fixes: trailer names its culprit by a hash that resolves and the culprit's subject | off | off | warn | warn | deny | off | off | off | off | off |
| [P0018](P0018.md) | [fix-without-test](P0018.md) | a commit that fixes code adds a test | off | off | off | warn | warn | off | off | off | off | off |
| [P0019](P0019.md) | [commit-tested-line](P0019.md) | a commit says how it was tested in a Tested: trailer | off | off | off | off | warn | off | off | off | off | off |
| [P0020](P0020.md) | [commit-mixed-move](P0020.md) | a commit that moves a file does not also edit it beyond its paths | off | off | off | off | warn | off | off | off | off | off |

## dead-code

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [C0009](C0009.md) | [dead-pub](C0009.md) | public items of workspace libraries are used outside their crate | off | off | off | warn | off | off | off | off | off | off |

## deny

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [X0019](X0019.md) | [deny-advisories](X0019.md) | no dependency has a security advisory, is unmaintained, unsound or yanked | warn | warn | deny | deny | off | off | off | off | off | off |
| [X0020](X0020.md) | [deny-bans](X0020.md) | the dependency graph holds no banned crate and no duplicate deny.toml refuses | off | off | off | deny | off | off | off | off | off | off |
| [X0021](X0021.md) | [deny-licenses](X0021.md) | every crate's licence is one deny.toml allows | off | off | warn | deny | off | off | off | off | off | off |
| [X0022](X0022.md) | [deny-sources](X0022.md) | every crate comes from a registry or git source deny.toml allows | off | off | off | deny | off | off | off | off | off | off |

## dependencies

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [M0001](M0001.md) | [default-features-off](M0001.md) | third-party dependencies are declared with default-features = false | warn | warn | warn | deny | off | off | off | off | off | off |
| [M0002](M0002.md) | [duplicate-versions](M0002.md) | no crate is in the dependency graph in two versions | off | warn | warn | deny | off | off | off | off | off | off |
| [M0003](M0003.md) | [git-dependency-pins](M0003.md) | git dependencies are pinned the way the project's policy says: a branch by default | warn | warn | deny | deny | off | off | off | off | off | off |
| [M0004](M0004.md) | [no-sys-crates](M0004.md) | the dependency graph has no -sys crates and no C build steps | deny | deny | deny | deny | off | off | off | off | off | off |
| [M0005](M0005.md) | [path-outside-workspace](M0005.md) | path dependencies stay inside the workspace | deny | deny | deny | deny | off | off | off | off | off | off |
| [M0027](M0027.md) | [patch-outside-workspace](M0027.md) | no committed [patch] or [replace] override takes a path outside the workspace | deny | deny | deny | deny | off | off | off | off | off | off |

## dma

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [F0016](F0016.md) | [dma-1](F0016.md) | DMA buffers sit in memory the DMA master can reach | off | off | off | off | off | deny | off | off | off | off |
| [F0017](F0017.md) | [dma-2](F0017.md) | DMA buffers in cacheable RAM are cleaned and invalidated, or not cacheable | off | off | off | off | off | deny | off | off | off | off |
| [F0018](F0018.md) | [dma-4](F0018.md) | every DMA ring consumer counts its overruns | off | off | off | off | off | warn | off | off | off | off |
| [F0019](F0019.md) | [dma-6](F0019.md) | where an erratum asks for DMA, the code uses DMA instead of polling | off | off | off | off | off | warn | off | off | off | off |

## egui

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [U0010](U0010.md) | [egui-shared-theme](U0010.md) | every egui app installs the shared theme | off | off | off | off | off | off | off | warn | off | off |
| [U0011](U0011.md) | [egui-status-glyph](U0011.md) | a status glyph is painted in its tone, never written into a label | off | off | off | off | off | off | off | deny | off | off |
| [U0012](U0012.md) | [egui-id-in-loop](U0012.md) | a widget made in a loop gets an id that differs per iteration | off | off | off | off | off | off | off | warn | off | off |
| [U0013](U0013.md) | [egui-repaint-every-frame](U0013.md) | UI code asks for a repaint when something changed, not on every frame | off | off | off | off | off | off | off | warn | off | off |
| [U0015](U0015.md) | [egui-panel-api](U0015.md) | egui panel and App code uses the API the version in use ships, never one it replaced | off | off | off | off | off | off | off | warn | off | off |
| [U0016](U0016.md) | [egui-checklist-menu](U0016.md) | a checklist of checkboxes sits in the shared checklist widget, not a menu_button that closes on each click | off | off | off | off | off | off | off | warn | off | off |

## embedded

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [F0020](F0020.md) | [firmware-readable](F0020.md) | every file of a firmware crate that the embedded lints read can be read | off | off | off | off | off | deny | off | off | off | off |

## err

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [F0021](F0021.md) | [err-11](F0021.md) | the silicon revision is read from a source that tells every revision apart | off | off | off | off | off | warn | off | off | off | off |
| [F0022](F0022.md) | [err-2](F0022.md) | every register poke for an erratum quotes the erratum next to it | off | off | off | off | off | deny | off | off | off | off |
| [F0023](F0023.md) | [err-3](F0023.md) | flash prefetch is off where code crosses into bank 2 on G0B0/G0B1/G0C1 | off | off | off | off | off | deny | off | off | off | off |
| [F0024](F0024.md) | [err-4](F0024.md) | FDCAN edge filtering and dedicated Tx buffers stay off | off | off | off | off | off | deny | off | off | off | off |
| [F0025](F0025.md) | [err-5](F0025.md) | FDCAN one-shot mode retransmits after a lost arbitration | off | off | off | off | off | warn | off | off | off | off |
| [F0026](F0026.md) | [err-6](F0026.md) | RP2350 A2 pads with a pull-down enable their input only for the read | off | off | off | off | off | warn | off | off | off | off |
| [F0027](F0027.md) | [err-7](F0027.md) | RP2350 ROM reboots use a delay and only the allowed FRCE_OFF bits | off | off | off | off | off | warn | off | off | off | off |
| [F0028](F0028.md) | [err-8](F0028.md) | G0 ADC calibration waits two ADC clocks after the disable | off | off | off | off | off | warn | off | off | off | off |

## errors

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [C0010](C0010.md) | [error-enum-growth](C0010.md) | a big error enum wraps the errors it comes from instead of listing every failure flat | off | off | warn | warn | off | off | off | off | off | off |

## exe

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [F0029](F0029.md) | [exe-1](F0029.md) | a task with a hard answer deadline runs on its own executor, not beside bulk work on the thread executor | off | off | off | off | off | deny | off | off | off | off |
| [F0030](F0030.md) | [exe-10](F0030.md) | one crate provides the executor and its pender | off | off | off | off | off | deny | off | off | off | off |
| [F0031](F0031.md) | [exe-12](F0031.md) | every async fn awaits something | off | off | off | off | off | warn | off | off | off | off |
| [F0032](F0032.md) | [exe-13](F0032.md) | a ring-buffered reader in a loop batches its reads | off | off | off | off | off | warn | off | off | off | off |
| [F0033](F0033.md) | [exe-2](F0033.md) | tasks on an interrupt executor and interrupt handlers never block | off | off | off | off | off | deny | off | off | off | off |
| [F0034](F0034.md) | [exe-3](F0034.md) | an interrupt executor's priority is set before it is started | off | off | off | off | off | deny | off | off | off | off |
| [F0035](F0035.md) | [exe-4](F0035.md) | an interrupt executor runs below the interrupts that wake its tasks | off | off | off | off | off | warn | off | off | off | off |
| [F0036](F0036.md) | [exe-5](F0036.md) | an interrupt executor uses a vector no driver binds and no used peripheral raises | off | off | off | off | off | deny | off | off | off | off |
| [F0037](F0037.md) | [exe-6](F0037.md) | state shared between an interrupt executor and thread mode sits behind a critical-section mutex | off | off | off | off | off | deny | off | off | off | off |
| [F0038](F0038.md) | [exe-7](F0038.md) | a loop that drains a queue in a shared task is bounded and yields | off | off | off | off | off | warn | off | off | off | off |
| [F0039](F0039.md) | [exe-8](F0039.md) | a request handler answers without waiting on a bus or a slow peripheral | off | off | off | off | off | warn | off | off | off | off |
| [F0040](F0040.md) | [exe-9](F0040.md) | no future that reaches embassy-time is polled with a no-op waker | off | off | off | off | off | deny | off | off | off | off |

## ext

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [F0041](F0041.md) | [ext-12](F0041.md) | an RCC enable is read back before the peripheral's first access | off | off | off | off | off | warn | off | off | off | off |
| [F0042](F0042.md) | [ext-4](F0042.md) | external chip registers are written from shadow copies, not read-modify-write | off | off | off | off | off | warn | off | off | off | off |
| [F0043](F0043.md) | [ext-7](F0043.md) | register fields are range-checked before they are encoded | off | off | off | off | off | warn | off | off | off | off |

## fls

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [F0044](F0044.md) | [fls-1](F0044.md) | flash a power cut can tear is read through ECC-aware access on parts with flash ECC | off | off | off | off | off | deny | off | off | off | off |
| [F0045](F0045.md) | [fls-2](F0045.md) | the flash peripheral has one owner | off | off | off | off | off | deny | off | off | off | off |
| [F0046](F0046.md) | [fls-3](F0046.md) | RP flash is erased and written from core 0 with nothing else reading flash | off | off | off | off | off | deny | off | off | off | off |
| [F0047](F0047.md) | [fls-4](F0047.md) | the image in flash is checked by a periodic CRC, and ECC is on where the part has it | off | off | off | off | off | warn | off | off | off | off |

## flt

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [F0048](F0048.md) | [flt-1](F0048.md) | the HardFault handler records the crash and resets instead of spinning | off | off | off | off | off | deny | off | off | off | off |
| [F0049](F0049.md) | [flt-2](F0049.md) | HardFault and DefaultHandler do nothing that can panic or log | off | off | off | off | off | deny | off | off | off | off |
| [F0050](F0050.md) | [flt-3](F0050.md) | bedrock_crash::exc_return!() is the first statement of HardFault | off | off | off | off | off | warn | off | off | off | off |
| [F0051](F0051.md) | [flt-4](F0051.md) | UsageFault, MemManage and BusFault have handlers and are enabled where the core has them | off | off | off | off | off | warn | off | off | off | off |
| [F0052](F0052.md) | [flt-5](F0052.md) | the reset cause is read and reported at boot | off | off | off | off | off | warn | off | off | off | off |
| [F0053](F0053.md) | [flt-7](F0053.md) | a watchdog runs, fed from a bounded loop of the main work | off | off | off | off | off | deny | off | off | off | off |
| [F0054](F0054.md) | [flt-8](F0054.md) | a boot-loop rescue falls back to the ROM bootloader after repeated unhealthy boots | off | off | off | off | off | warn | off | off | off | off |
| [F0055](F0055.md) | [flt-9](F0055.md) | the healthy mark is set on the transport in use | off | off | off | off | off | warn | off | off | off | off |
| [F0108](F0108.md) | [flt-11](F0108.md) | the panic and HardFault handlers put the board into its safe state before anything else | off | off | off | off | off | warn | off | off | off | off |
| [F0109](F0109.md) | [flt-12](F0109.md) | the boot counter is read before the first output is enabled | off | off | off | off | off | warn | off | off | off | off |

## geiger

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [X0023](X0023.md) | [geiger-unsafe-deps](X0023.md) | every dependency that uses unsafe code is one the project accepted | off | off | off | off | off | off | off | off | off | off |

## graph

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [G0001](G0001.md) | [core-crate-deps](G0001.md) | a _core crate depends only on other _core crates, the allowed layers and listed vocabulary crates | off | off | warn | deny | off | off | off | off | off | off |
| [G0002](G0002.md) | [core-crate-types](G0002.md) | a _core crate holds types: no I/O, no threads, no async runtime | off | off | off | deny | off | off | off | off | off | off |
| [G0003](G0003.md) | [crate-cycles](G0003.md) | workspace crates form no cycle, dev-dependencies included | off | warn | deny | deny | off | off | off | off | off | off |
| [G0004](G0004.md) | [crate-table](G0004.md) | the crate table in the README is the one generated from the manifests and the map | off | off | off | deny | off | off | off | off | off | off |
| [G0005](G0005.md) | [external-form](G0005.md) | crates of other projects come by git as the map lists them, used only where their role allows | off | off | off | deny | off | off | off | off | off | off |
| [G0006](G0006.md) | [forbidden-closure](G0006.md) | a crate never reaches the crates the map forbids it, directly or through others | off | off | off | deny | off | off | off | off | off | off |
| [G0007](G0007.md) | [layer-edges](G0007.md) | every dependency edge between workspace crates is allowed by the map | off | off | off | deny | off | off | off | off | off | off |
| [G0008](G0008.md) | [map-complete](G0008.md) | the crate map and the workspaces list the same crates | off | off | off | deny | off | off | off | off | off | off |
| [G0009](G0009.md) | [third-party-placement](G0009.md) | heavy third-party crates are used only by the crates the map names | off | off | off | deny | off | off | off | off | off | off |
| [G0010](G0010.md) | [unsafe-allow-list](G0010.md) | unsafe code is forbidden except in the crates the map marks unsafe | off | off | off | deny | off | off | off | off | off | off |
| [G0011](G0011.md) | [crate-folder-layer](G0011.md) | a crate's folder is its layer in the map, then its discipline where the map gives one | off | off | off | deny | off | off | off | off | off | off |
| [G0012](G0012.md) | [map-licence](G0012.md) | every crate carries the licence the map sets for its layer or for itself | off | off | off | deny | off | off | off | off | off | off |

## hakari

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [X0024](X0024.md) | [hakari-generate](X0024.md) | the workspace-hack package's dependencies are the ones hakari computes | off | off | off | warn | off | off | off | off | off | off |
| [X0025](X0025.md) | [hakari-manage-deps](X0025.md) | every member hakari manages depends on the workspace-hack package, and no other | off | off | off | warn | off | off | off | off | off | off |

## host

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [F0056](F0056.md) | [host-1](F0056.md) | nothing reachable from host input can panic | off | off | off | off | off | deny | off | off | off | off |
| [F0057](F0057.md) | [host-3](F0057.md) | unknown enum values from the host are refused, not mapped to a default | off | off | off | off | off | warn | off | off | off | off |
| [F0058](F0058.md) | [host-5](F0058.md) | truncated and oversized requests are counted and dropped | off | off | off | off | off | warn | off | off | off | off |

## hygiene

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [A0001](A0001.md) | [agent-config-permissions](A0001.md) | committed agent settings grant no blanket permissions | off | off | off | off | deny | off | off | off | off | off |
| [A0002](A0002.md) | [agent-config-secrets](A0002.md) | committed agent configuration holds no secrets | off | off | off | off | deny | off | off | off | off | off |
| [A0003](A0003.md) | [agent-session-files](A0003.md) | no agent transcripts or per-person agent files are committed | off | off | off | off | deny | off | off | off | off | off |
| [A0004](A0004.md) | [leftover-files](A0004.md) | no merge, patch, editor or scratch leftovers are committed | off | off | off | off | deny | off | off | off | off | off |
| [A0012](A0012.md) | [large-blobs](A0012.md) | no git-tracked file is over the size limit outside the allow list | off | off | off | off | warn | off | off | off | off | off |

## irq

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [F0059](F0059.md) | [irq-1](F0059.md) | interrupt handlers clear exactly the flags they handled | off | off | off | off | off | deny | off | off | off | off |
| [F0060](F0060.md) | [irq-2](F0060.md) | write-1-to-clear flags are cleared with a write, not modify() | off | off | off | off | off | deny | off | off | off | off |
| [F0061](F0061.md) | [irq-4](F0061.md) | no driver disables a shared interrupt vector that other drivers use | off | off | off | off | off | deny | off | off | off | off |
| [F0062](F0062.md) | [irq-5](F0062.md) | every routed interrupt line is enabled and has a handler bound | off | off | off | off | off | warn | off | off | off | off |
| [F0063](F0063.md) | [irq-6](F0063.md) | interrupts are disabled through critical sections, never nested by hand | off | off | off | off | off | warn | off | off | off | off |
| [F0064](F0064.md) | [irq-8](F0064.md) | the default interrupt handler records which IRQ fired | off | off | off | off | off | warn | off | off | off | off |

## links

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [D0021](D0021.md) | [page-links](D0021.md) | a link between pages is a relative path to a Markdown file that exists | deny | deny | deny | deny | off | off | off | off | off | off |

## log

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [F0065](F0065.md) | [log-1](F0065.md) | RTT never blocks | off | off | off | off | off | deny | off | off | off | off |
| [F0066](F0066.md) | [log-2](F0066.md) | DEFMT_RTT_BUFFER_SIZE and DEFMT_LOG are set in .cargo/config.toml | off | off | off | off | off | warn | off | off | off | off |

## machete

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [X0026](X0026.md) | [machete-unused-deps](X0026.md) | no crate declares a dependency its sources never use | warn | warn | warn | deny | off | off | off | off | off | off |

## manifest

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [M0006](M0006.md) | [crate-docs](M0006.md) | every crate root opens with crate-level documentation | warn | warn | warn | deny | off | off | off | off | off | off |
| [M0007](M0007.md) | [edition-latest](M0007.md) | every crate is on the current edition | warn | warn | warn | warn | off | off | off | off | off | off |
| [M0008](M0008.md) | [feature-docs](M0008.md) | every cargo feature has a comment saying what it turns on | off | off | off | warn | off | off | off | off | off | off |
| [M0009](M0009.md) | [package-description](M0009.md) | every crate has a description in its Cargo.toml | warn | warn | warn | deny | off | off | off | off | off | off |
| [M0010](M0010.md) | [package-license](M0010.md) | every crate states its licence | warn | warn | warn | warn | off | off | off | off | off | off |
| [M0011](M0011.md) | [publish-false](M0011.md) | workspace crates are publish = false unless listed as published | off | off | warn | warn | off | off | off | off | off | off |
| [M0012](M0012.md) | [readme-per-crate](M0012.md) | every crate has a README | warn | deny | deny | deny | off | off | off | off | off | off |
| [M0013](M0013.md) | [rust-version-set](M0013.md) | every crate declares its minimum Rust version | off | off | warn | deny | off | off | off | off | off | off |
| [M0025](M0025.md) | [tier-clippy-set](M0025.md) | the project's [lints.clippy] and clippy.toml turn on the Clippy set its tiers expect, every opt-out with a reason | warn | warn | deny | deny | off | off | off | off | off | off |

## mem

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [F0067](F0067.md) | [mem-1](F0067.md) | the FLASH region of memory.x leaves out the regions kept for other uses | off | off | off | off | off | deny | off | off | off | off |
| [F0068](F0068.md) | [mem-10](F0068.md) | no LOAD segment is writable and executable, and .rodata holds nothing writable | off | off | off | off | off | warn | off | off | off | off |
| [F0069](F0069.md) | [mem-12](F0069.md) | a firmware with defmt links with -Tdefmt.x | off | off | off | off | off | deny | off | off | off | off |
| [F0070](F0070.md) | [mem-2](F0070.md) | extra RAM sections end without a tail ALIGN or symbol | off | off | off | off | off | warn | off | off | off | off |
| [F0071](F0071.md) | [mem-3](F0071.md) | the RAM init and zeroing ranges stay clear of .uninit | off | off | off | off | off | deny | off | off | off | off |
| [F0072](F0072.md) | [mem-6](F0072.md) | .cnt_bkp_buffer sits at the backup-register address | off | off | off | off | off | warn | off | off | off | off |
| [F0073](F0073.md) | [mem-7](F0073.md) | embassy-boot partition symbols are offsets from the flash origin | off | off | off | off | off | warn | off | off | off | off |

## pan

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [F0110](F0110.md) | [pan-1](F0110.md) | every panic site of the release ELF is acknowledged with a reason | off | off | off | off | off | off | off | off | off | off |

## perf

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [F0074](F0074.md) | [perf-2](F0074.md) | hot paths on cores without 64-bit maths avoid it, and float formatting | off | off | off | off | off | warn | off | off | off | off |
| [F0075](F0075.md) | [perf-3](F0075.md) | RAM functions are #[inline(never)] | off | off | off | off | off | warn | off | off | off | off |

## pin

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [F0076](F0076.md) | [pin-1](F0076.md) | pin drivers live as long as the pin is used | off | off | off | off | off | deny | off | off | off | off |
| [F0077](F0077.md) | [pin-16](F0077.md) | pins held by a dead-battery pull-down are read only after it is released | off | off | off | off | off | warn | off | off | off | off |
| [F0078](F0078.md) | [pin-2](F0078.md) | the board is put in its safe state first thing after the HAL init | off | off | off | off | off | deny | off | off | off | off |
| [F0079](F0079.md) | [pin-4](F0079.md) | peripheral pins stay on their function while the peripheral is enabled | off | off | off | off | off | deny | off | off | off | off |
| [F0080](F0080.md) | [pin-6](F0080.md) | every pin has one owner | off | off | off | off | off | deny | off | off | off | off |
| [F0081](F0081.md) | [pin-8](F0081.md) | unused pins are analog or pulled, never floating inputs | off | off | off | off | off | warn | off | off | off | off |
| [F0082](F0082.md) | [pin-9](F0082.md) | the pin map comes from the board at run time, not from a cargo feature | off | off | off | off | off | deny | off | off | off | off |

## prose

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [P0008](P0008.md) | [changelog-describes-diff](P0008.md) | a CHANGELOG entry says what changed for the user, not which code was edited | off | off | off | off | warn | off | off | off | off | off |
| [P0009](P0009.md) | [changelog-entry-length](P0009.md) | a CHANGELOG entry is one short line | warn | warn | warn | warn | deny | off | off | off | off | off |
| [P0010](P0010.md) | [emoji](P0010.md) | docs and comments carry no emoji | off | off | off | off | warn | off | off | off | off | off |
| [P0011](P0011.md) | [filler-phrases](P0011.md) | docs and comments use no filler phrases | off | off | off | off | deny | off | off | off | off | off |
| [P0012](P0012.md) | [narration](P0012.md) | docs and comments state facts, not how they were written | off | off | off | off | deny | off | off | off | off | off |
| [P0013](P0013.md) | [paragraph-sentences](P0013.md) | paragraphs in docs stay short | off | off | off | off | warn | off | off | off | off | off |
| [P0014](P0014.md) | [restating-comments](P0014.md) | a comment says what the code cannot | off | off | off | off | warn | off | off | off | off | off |
| [P0015](P0015.md) | [sentence-words](P0015.md) | sentences in docs stay short | off | off | off | off | warn | off | off | off | off | off |
| [P0016](P0016.md) | [thinking-comments](P0016.md) | comments state conclusions, not a stream of thought | off | off | off | off | deny | off | off | off | off | off |
| [P0021](P0021.md) | [url-tracking-params](P0021.md) | a link in docs or a comment carries no tracking query parameter | off | off | off | off | warn | off | off | off | off | off |

## pwr

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [F0083](F0083.md) | [pwr-1](F0083.md) | a brown-out reset level or voltage detector fits the board's supply | off | off | off | off | off | warn | off | off | off | off |
| [F0084](F0084.md) | [pwr-3](F0084.md) | the backup domain is reset at boot when nothing keeps state there | off | off | off | off | off | warn | off | off | off | off |
| [F0085](F0085.md) | [pwr-4](F0085.md) | a firmware that keeps state in the backup domain never changes the RTC clock source | off | off | off | off | off | warn | off | off | off | off |

## reading

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [D0001](D0001.md) | [prose-run-words](D0001.md) | prose between two visual breaks stays short | warn | warn | warn | warn | off | off | off | off | off | off |
| [D0002](D0002.md) | [page-reading-time](D0002.md) | a guide reads in a bounded time | warn | warn | warn | warn | off | off | off | off | off | off |
| [D0003](D0003.md) | [heading-words](D0003.md) | a heading is a short name, not a sentence | warn | warn | warn | warn | off | off | off | off | off | off |
| [D0004](D0004.md) | [page-headings](D0004.md) | a long page is divided into sections | warn | warn | warn | warn | off | off | off | off | off | off |
| [D0005](D0005.md) | [code-wall](D0005.md) | code comes with prose that explains it | warn | warn | warn | warn | off | off | off | off | off | off |
| [D0006](D0006.md) | [image-alt](D0006.md) | every image has alternative text | warn | warn | warn | warn | off | off | off | off | off | off |
| [D0007](D0007.md) | [code-block-language](D0007.md) | every code block names its language | warn | warn | warn | warn | off | off | off | off | off | off |

## repo

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [M0014](M0014.md) | [changelog-unreleased](M0014.md) | CHANGELOG.md has an Unreleased section | warn | warn | warn | warn | off | off | off | off | off | off |
| [M0015](M0015.md) | [justfile-recipes](M0015.md) | the justfile has the standard recipes | warn | warn | warn | warn | off | off | off | off | off | off |
| [M0016](M0016.md) | [justfile-simple](M0016.md) | justfile recipes only call programs: a few lines, no inline scripts | off | off | off | warn | off | off | off | off | off | off |
| [M0017](M0017.md) | [profile-defaults](M0017.md) | the dev profile splits debug info and a deploy profile builds incrementally | warn | warn | warn | warn | off | off | off | off | off | off |
| [M0018](M0018.md) | [repo-files](M0018.md) | the repo carries its rule files | warn | warn | warn | warn | deny | off | off | off | off | off |
| [M0019](M0019.md) | [toolchain-pinned](M0019.md) | the repo pins its toolchain in rust-toolchain.toml | off | off | warn | deny | off | off | off | off | off | off |
| [M0026](M0026.md) | [profile-no-lto](M0026.md) | the profiles of the edit-build-test loop and the deploy profile build without LTO | warn | warn | warn | warn | off | off | off | off | off | off |
| [M0028](M0028.md) | [test-binaries](M0028.md) | a crate builds its integration tests as one binary | off | off | warn | warn | off | off | off | off | off | off |
| [M0031](M0031.md) | [fuzz-corpus-test](M0031.md) | every fuzz target has a corpus tracked in git and a test that replays it | off | off | warn | warn | off | off | off | off | off | off |

## rustfmt

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [X0027](X0027.md) | [rustfmt-unformatted](X0027.md) | every Rust file is formatted the way cargo fmt writes it | off | off | off | off | off | off | off | off | off | off |

## semver

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [X0028](X0028.md) | [semver-checks-breaking](X0028.md) | a stable crate changes its public API only with the version bump the change needs | off | off | off | deny | off | off | off | off | off | off |

## shear

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [X0029](X0029.md) | [shear-misplaced-deps](X0029.md) | a dependency only tests use is a dev-dependency | off | off | off | off | off | off | off | off | off | off |
| [X0030](X0030.md) | [shear-unused-deps](X0030.md) | no crate declares a dependency it never uses, and every workspace dependency is used | off | off | off | off | off | off | off | off | off | off |

## siz

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [F0086](F0086.md) | [siz-1](F0086.md) | flash and RAM regions stay under a fill limit | off | off | off | off | off | warn | off | off | off | off |
| [F0087](F0087.md) | [siz-3](F0087.md) | no per-package opt-level beside lto = "fat" | off | off | off | off | off | warn | off | off | off | off |
| [F0088](F0088.md) | [siz-4](F0088.md) | the release profile sets codegen-units = 1, lto, panic = "abort" and a measured opt-level | off | off | off | off | off | warn | off | off | off | off |
| [F0089](F0089.md) | [siz-5](F0089.md) | core::fmt stays out of the firmware | off | off | off | off | off | warn | off | off | off | off |
| [F0090](F0090.md) | [siz-6](F0090.md) | no large .data initializer | off | off | off | off | off | warn | off | off | off | off |

## size

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [S0001](S0001.md) | [crate-lines](S0001.md) | a crate's src/ stays under its line limit | off | off | warn | warn | off | off | off | off | off | off |
| [S0002](S0002.md) | [function-complexity](S0002.md) | a function's cyclomatic complexity stays under its limit | warn | warn | warn | warn | off | off | off | off | off | off |
| [S0003](S0003.md) | [function-lines](S0003.md) | a function stays under its line limit | warn | warn | warn | warn | off | off | off | off | off | off |
| [S0004](S0004.md) | [module-lines](S0004.md) | a source file stays under its line limit | off | off | warn | warn | off | off | off | off | off | off |
| [S0005](S0005.md) | [test-lines](S0005.md) | a crate's test code stays under its line limit | off | off | off | warn | off | off | off | off | off | off |
| [S0006](S0006.md) | [crate-folder-size](S0006.md) | a folder holds no more crate folders than its limit | off | off | warn | deny | off | off | off | off | off | off |

## state

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [T0001](T0001.md) | [lock-held-across](T0001.md) | no lock guard is held across an .await or an I/O call | warn | warn | warn | deny | off | off | off | off | off | off |
| [T0002](T0002.md) | [lock-unwrap](T0002.md) | a crate keeps its lock().unwrap() calls under a limit | off | off | warn | warn | off | off | off | off | off | off |
| [T0003](T0003.md) | [nested-locks](T0003.md) | no second lock is taken while a guard is alive | off | warn | warn | warn | off | off | off | off | off | off |
| [T0004](T0004.md) | [static-interior-mutability](T0004.md) | a static with interior mutability has one owner, or a reason why a global lock is right | warn | warn | warn | deny | off | off | off | off | off | off |
| [T0005](T0005.md) | [static-mut](T0005.md) | no static mut | deny | deny | deny | deny | off | off | off | off | off | off |
| [T0006](T0006.md) | [thread-local-state](T0006.md) | thread_local! state has one owner, or a reason | off | warn | warn | warn | off | off | off | off | off | off |

## stk

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [F0091](F0091.md) | [stk-1](F0091.md) | flip-link is the linker | off | off | off | off | off | deny | off | off | off | off |
| [F0092](F0092.md) | [stk-2](F0092.md) | build.rs prints only directives Cargo knows | off | off | off | off | off | deny | off | off | off | off |
| [F0093](F0093.md) | [stk-3](F0093.md) | the stack sits below .data and .bss and is not empty | off | off | off | off | off | deny | off | off | off | off |
| [F0094](F0094.md) | [stk-4](F0094.md) | cortex-m-rt paint-stack stays off | off | off | off | off | off | deny | off | off | off | off |
| [F0095](F0095.md) | [stk-5](F0095.md) | big state is built in place, not by value and moved into a static | off | off | off | off | off | warn | off | off | off | off |
| [F0096](F0096.md) | [stk-6](F0096.md) | every stack frame stays within its budget and the deepest call chain within the stack | off | off | off | off | off | warn | off | off | off | off |

## stop-gap

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [A0005](A0005.md) | [commented-out-code](A0005.md) | no code is left commented out | off | off | off | off | deny | off | off | off | off | off |
| [A0006](A0006.md) | [dead-features](A0006.md) | every cargo feature enables something or is tested by a cfg | off | off | off | off | warn | off | off | off | off | off |
| [A0007](A0007.md) | [lint-table-allow-reason](A0007.md) | a lint allowed in a Cargo.toml [lints] table says why in a comment | off | off | off | off | deny | off | off | off | off | off |
| [A0008](A0008.md) | [near-duplicate-fns](A0008.md) | no two functions are near copies of each other | off | off | off | off | warn | off | off | off | off | off |
| [A0009](A0009.md) | [near-duplicate-modules](A0009.md) | no two modules are near copies of each other | off | off | off | off | deny | off | off | off | off | off |
| [A0010](A0010.md) | [stop-gap-comments](A0010.md) | comments carry no stop-gap markers: TODO, HACK, for now, workaround | off | off | off | deny | deny | off | off | off | off | off |
| [A0011](A0011.md) | [fixme-before-land](A0011.md) | no FIXME is left when a branch lands | off | off | off | deny | deny | off | off | off | off | off |

## structure

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [C0011](C0011.md) | [module-graph-scc](C0011.md) | no module cycle holds most of a crate's code | off | off | warn | warn | off | off | off | off | off | off |
| [C0016](C0016.md) | [orphan-source-file](C0016.md) | every .rs file under src/ is reached by a mod declaration | off | off | warn | deny | off | off | off | off | off | off |

## taplo

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [X0031](X0031.md) | [taplo-fmt](X0031.md) | every TOML file parses and is formatted as taplo writes it | off | warn | warn | deny | off | off | off | off | off | off |

## tools

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [X0034](X0034.md) | [tool-version](X0034.md) | every external tool an enabled lint runs is a version its output is tested against | deny | deny | deny | deny | off | off | off | off | off | off |

## typos

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [X0032](X0032.md) | [typos-misspelling](X0032.md) | no misspelled word in code, comments, docs or file names | warn | warn | warn | deny | off | off | off | off | off | off |

## udeps

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [X0033](X0033.md) | [udeps-unused-deps](X0033.md) | no crate declares a dependency the compiler never loads | off | off | off | off | off | off | off | off | off | off |

## ui

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [U0001](U0001.md) | [ui-raw-spawn](U0001.md) | UI code starts threads and tasks only through the project's spawn wrapper | off | off | off | off | off | off | warn | warn | off | off |
| [U0002](U0002.md) | [ui-blocking-call](U0002.md) | UI code never blocks the UI thread on files, the network, processes or sleeps | off | off | off | off | off | off | warn | warn | off | off |
| [U0003](U0003.md) | [ui-colour-literal](U0003.md) | UI code takes its colours from the theme, never writes them as values | off | off | off | off | off | off | warn | warn | off | off |
| [U0004](U0004.md) | [ui-size-literal](U0004.md) | UI code takes spacing, sizes, radii and type sizes from the theme, never writes them as numbers | off | off | off | off | off | off | warn | warn | off | off |
| [U0005](U0005.md) | [ui-disabled-without-reason](U0005.md) | a disabled control says why and what would enable it | off | off | off | off | off | off | warn | warn | off | off |
| [U0006](U0006.md) | [ui-destructive-unconfirmed](U0006.md) | a control that throws work away asks first or can be undone | off | off | off | off | off | off | warn | warn | off | off |
| [U0007](U0007.md) | [ui-more-count](U0007.md) | a cut list ends in a fold that opens to every item, never in "N more" | off | off | off | off | off | off | warn | warn | off | off |
| [U0008](U0008.md) | [ui-label-punctuation](U0008.md) | UI text writes the ellipsis as one character and button labels end without a full stop | off | off | off | off | off | off | deny | deny | off | off |
| [U0009](U0009.md) | [ui-debug-text](U0009.md) | text shown to a person never holds Rust's debug formatting | off | off | off | off | off | off | warn | warn | off | off |
| [U0014](U0014.md) | [ui-unwrap-external-data](U0014.md) | UI code returns an error for a bad file, parse or connection, never unwraps or expects it | off | off | off | off | off | off | warn | warn | off | off |

## units

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [C0033](C0033.md) | [exact-units-quantity](C0033.md) | a number that holds a physical or dimensionless quantity carries its unit in its type | warn | warn | warn | warn | off | off | off | off | off | off |
| [C0034](C0034.md) | [exact-units-plain-number](C0034.md) | an index, a count, a sequence number or a duration carries what it counts in its type | off | off | off | off | off | off | off | off | warn | warn |
| [C0035](C0035.md) | [exact-units-bounds](C0035.md) | a number whose range the code knows says it in its type | warn | warn | warn | warn | off | off | off | off | off | off |
| [C0036](C0036.md) | [exact-units-in-bodies](C0036.md) | inside functions too, a quantity stays typed: no unit-named locals or literal arguments, casts, float maths, bare comparisons, raw arithmetic, scale constants or conversions by hand | off | off | off | off | off | off | off | off | off | warn |

## usb

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [F0097](F0097.md) | [usb-1](F0097.md) | high-speed bulk endpoints have 512-byte packets and bIntervals in range | off | off | off | off | off | deny | off | off | off | off |
| [F0098](F0098.md) | [usb-3](F0098.md) | bulk endpoints on a hot path are double-buffered | off | off | off | off | off | warn | off | off | off | off |

## waivers

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [C0012](C0012.md) | [waiver-invalid](C0012.md) | a waiver comment names known lints, gives a reason and has not expired | deny | deny | deny | deny | off | off | off | off | off | off |
| [C0013](C0013.md) | [waiver-unused](C0013.md) | every waiver comment allows a finding | warn | warn | warn | deny | off | off | off | off | off | off |

## words

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [D0012](D0012.md) | [abbreviation-first-use](D0012.md) | an abbreviation is spelled out at its first use on a page | warn | warn | warn | warn | off | off | off | off | off | off |
| [D0013](D0013.md) | [term-before-definition](D0013.md) | a page defines a term before it uses it | warn | warn | warn | warn | off | off | off | off | off | off |
| [D0014](D0014.md) | [term-synonyms](D0014.md) | one concept has one name | warn | warn | warn | warn | off | off | off | off | off | off |
| [D0015](D0015.md) | [passive-voice](D0015.md) | a guide says who does what: few passive sentences | warn | warn | warn | warn | off | off | off | off | off | off |
| [D0016](D0016.md) | [reading-ease](D0016.md) | a guide reads easily: short sentences of short words | warn | warn | warn | warn | off | off | off | off | off | off |

## workspace

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded | ui | egui | units-strict | units-paranoid |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [M0020](M0020.md) | [crate-dir-name](M0020.md) | a member's directory is named after its crate | off | warn | warn | warn | off | off | off | off | off | off |
| [M0021](M0021.md) | [macro-crate-suffix](M0021.md) | proc-macro crates and only they end in _macros or _derive | off | warn | warn | warn | off | off | off | off | off | off |
| [M0022](M0022.md) | [workspace-dependencies](M0022.md) | members take every dependency from [workspace.dependencies] | off | warn | deny | deny | off | off | off | off | off | off |
| [M0023](M0023.md) | [workspace-inheritance](M0023.md) | members inherit the shared package keys from [workspace.package] | off | deny | deny | deny | off | off | off | off | off | off |
| [M0024](M0024.md) | [workspace-lints](M0024.md) | the root declares [workspace.lints] and every member inherits them | off | deny | deny | deny | off | off | off | off | off | off |
| [M0029](M0029.md) | [macro-compile-fail](M0029.md) | every proc-macro crate has compile-fail tests | off | off | warn | warn | off | off | off | off | off | off |
| [M0030](M0030.md) | [contract-snapshot](M0030.md) | a contract crate exports its version and keeps every version's schema snapshot in git, unchanged once in the base | off | off | off | deny | off | off | off | off | off | off |
