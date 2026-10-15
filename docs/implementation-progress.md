# Implementation progress

This checklist tracks the requested roadmap. Unchecked items are outstanding;
mock coverage does not constitute hardware or native Windows validation.

## Phase 0 — Baseline and safety

- [x] Review the feature matrix and existing firmware authorization boundary.
- [x] Preserve candidate gates and firmware preflight; grant no new capabilities.
- [x] Complete CLI/TUI and Linux/Windows source regression audit ([scope and limits](audit-2026-10-14.md)); native Windows runtime remains outstanding.

## Phase 1 — TUI

- [x] Separate everyday registry browsing (`F`) from Experimental Features (`v`).
- [x] Keep registry search and categories scoped to the selected workspace.
- [x] Extract browser policy into `src/tui/workspace.rs`.
- [x] Extract feature list/detail rendering into `src/tui/view/features.rs`.
- [x] Add regression coverage for registry partitioning and candidate form guards.
- [x] Implement persistent Home / Audio / Devices / Settings / Diagnostics / Research navigation.
- [x] Extract typed forms, dialogs, routing, header and main screens from state/view modules.
- [x] Validate navigation and layout with offline PTY, all-locale buffer tests and narrow/wide visual previews.

## Multipoint follow-up

- [x] Fix `r` in the multipoint page to refresh the verified device snapshot.
- [x] Retain connection checks, duplicate-request suppression and cancellation.
- [x] Test dual-state updates while peer identities remain explicitly unknown.
- [ ] Peer discovery: validate candidate `0x0D` against official-app HP206 captures.
- [ ] Peer disconnect/reconnect/switch: establish packet formats from hardware captures;
  no proven instruction/payload mapping is present in this repository.

The dual-device toggle enables two links; it does not identify the other connected
phone or authorize remote peer controls. The local paired-device list cannot supply
that missing information. Refresh uses only the established status/firmware path.

## Phase 2 — Bluetooth reliability

- [x] Add bounded BlueZ signal observation alongside the existing refresh fallback.
- [ ] Validate BlueZ signals with a real adapter/headset (offline implementation tests pass).
- [x] Audit reconnection, target changes and shutdown source/offline behavior on Linux and Windows ([audit](audit-2026-10-14.md)); Windows runtime is a separate open requirement.

## Phases 3–5 — Hardware-dependent functionality

- [ ] Capture read-only codec and multipoint exchanges on an identified HP206 firmware.
- [ ] Validate acknowledgments/readback independently for each reversible control.
- [ ] Validate advanced mappings, AI, sharing and finder before exposing live controls.
- [x] Retain reset protection; candidate reset stays blocked without destructive hardware evidence.

These tasks require official-app captures. New core status/firmware reads and
OS audio observations are recorded in the [audit](audit-2026-10-14.md); they do not
validate advanced packets or promote any candidate opcode.

## Phase 6 — CLI, configuration and localization

- [x] Audit CLI/JSON consistency against the registry and shared application policy ([audit](audit-2026-10-14.md)).
- [x] Complete profile/configuration and catalog consistency review across all eleven languages ([audit](audit-2026-10-14.md)); native-speaker review is not claimed.

## Phase 7 — Testing and release

- [x] Add opt-in hardware-in-loop core-read validation with firmware-scoped evidence records.
- [x] Execute read-only core hardware validation and publish redacted evidence.
- [ ] Independently verify reversible controls.
- [ ] Run native Windows Bluetooth/audio validation.
- [x] Add deterministic ZIP packaging, manifests, checksums and native release build workflow.
- [ ] Complete release verification on native platforms and verify remote workflow artifacts.

## Validation for the current worktree

- Stable Rust default-feature suite: 325 passed; three opt-in tests ignored
  (live system bus, live subscriptions and SVG artifact export).
- CLI-only suite: 202 passed; live system-bus test ignored.
- Rust 1.88.0 locked all-target source check and offline test suite: passed.
- Strict Clippy for default and CLI-only variants: passed.
- Windows MSVC all-target source checks for both variants: passed; not runtime tests.
- Nine offline PTY lifecycle/navigation scenarios pass against the release TUI.
- All six routes render in all eleven locales at narrow/wide buffer sizes.
- Synthetic SVG previews visually inspected at 56 and 110 columns.
- Eight Python checks cover hardware-runner gates/records and deterministic packaging.
- Linux TUI and CLI-only locked release builds with Rust 1.88.0: passed.
- Two ZIPs produced from identical inputs compare byte-for-byte equal.
- Workflow YAML parsing, documentation links, formatting and diff whitespace: passed.

The ZIPs used for local packaging checks are explicitly marked as dirty-worktree
development artifacts. No release was published or remote workflow result verified.
Read-only core RFCOMM and OS audio observations were performed; no candidate
packet or native Windows run was performed. Live BlueZ subscriptions were accepted,
but physical state-change delivery remains unverified.

## Remaining completion evidence

Native Bluetooth/audio runs on Windows, real BlueZ signal observation, identified
HP206 codec/peer captures and independent per-control hardware verification remain
required. Source/offline CLI/JSON and profile/configuration reviews are documented
in the audit. Outstanding physical checks must not be replaced by mocks or source
compilation.
