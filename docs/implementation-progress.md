# Implementation progress

This checklist tracks the requested roadmap. Unchecked items are outstanding;
mock coverage does not constitute hardware or native Windows validation.

## Phase 0 — Baseline and safety

- [x] Review the feature matrix and existing firmware authorization boundary.
- [x] Preserve candidate gates and firmware preflight; grant no new capabilities.
- [ ] Complete baseline regression audit across CLI, TUI and both native backends.

## Phase 1 — TUI

- [x] Separate everyday registry browsing (`F`) from Experimental Features (`v`).
- [x] Keep registry search and categories scoped to the selected workspace.
- [x] Extract browser policy into `src/tui/workspace.rs`.
- [x] Extract feature list/detail rendering into `src/tui/view/features.rs`.
- [x] Add regression coverage for registry partitioning and candidate form guards.
- [ ] Implement persistent Home / Audio / Devices / Settings / Diagnostics / Research navigation.
- [ ] Finish extracting dialogs, routing and other screens from state/view modules.
- [ ] Validate the complete redesign with PTY and visual checks.

## Phase 2 — Bluetooth reliability

- [ ] Add bounded BlueZ signal observation alongside the existing refresh fallback.
- [ ] Audit reconnection, target changes and shutdown on Linux and Windows.

## Phases 3–5 — Hardware-dependent functionality

- [ ] Capture read-only codec and multipoint exchanges on an identified HP206 firmware.
- [ ] Validate acknowledgments/readback independently for each reversible control.
- [ ] Validate advanced mappings, AI, sharing and finder before exposing live controls.
- [ ] Retain reset protection until destructive behavior has explicit hardware evidence.

These tasks require hardware access and captures; this change produces no new
hardware evidence and does not promote any candidate opcode.

## Phase 6 — CLI, configuration and localization

- [ ] Audit CLI/JSON consistency against the registry and application service boundary.
- [ ] Complete profile/configuration and translation review across all supported languages.

## Phase 7 — Testing and release

- [ ] Add opt-in hardware-in-loop validation with firmware-scoped evidence records.
- [ ] Run native Windows Bluetooth/audio validation.
- [ ] Complete packaging and reproducible release verification.

## Validation for this increment

- `cargo test --all-targets`: 292 passed, one system-bus test ignored.
- `cargo test --no-default-features --all-targets`: 187 passed, one system-bus test ignored.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo fmt --check` and `git diff --check`: passed.
- Offline PTY lifecycle/navigation smoke checks: eight scenarios, including Research.

No new Bluetooth hardware operation or native Windows run was performed.
