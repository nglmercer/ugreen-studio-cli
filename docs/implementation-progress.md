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
  targeted dual-close now has a static `0x06 [0,index,mac×6]` mapping, but no
  hardware-verified mapping or independent reconnect/switch format is present.

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

- Stable Rust default-feature suite: 337 passed; four opt-in tests ignored
  (live system bus, live subscriptions, physical signal cycle and SVG artifact export).
- CLI-only suite: 209 passed; live system-bus test ignored.
- Rust 1.88.0 locked all-target source check and offline test suite: passed.
- Strict Clippy for default and CLI-only variants: passed.
- Windows MSVC all-target source checks for both variants: passed; not runtime tests.
- Eleven offline PTY lifecycle/navigation scenarios pass against the release TUI.
- All six routes render in all eleven locales at narrow/wide buffer sizes.
- Synthetic SVG previews visually inspected at 56 and 110 columns.
- Eighteen Python checks cover hardware-runner gates/records, deterministic packaging,
  finished-bundle integrity/provenance verification and APK evidence parsing.
- Linux TUI and CLI-only locked release builds with Rust 1.88.0: passed.
- Two ZIPs produced from identical inputs compare byte-for-byte equal.
- Workflow YAML parsing, documentation links, formatting and diff whitespace: passed.

The ZIPs used for local packaging checks are explicitly marked as dirty-worktree
development artifacts. No release was published or remote workflow result verified.
Core and experimental peer/codec reads plus OS audio observations were performed;
no control write or native Windows run was performed in this follow-up. Live BlueZ subscriptions were accepted,
but physical state-change delivery remains unverified.

## Remaining completion evidence

Native Bluetooth/audio runs on Windows, real BlueZ signal observation, identified
HP206 codec/peer captures and independent per-control hardware verification remain
required. Source/offline CLI/JSON and profile/configuration reviews are documented
in the audit. Outstanding physical checks must not be replaced by mocks or source
compilation.

## Follow-up — 2026-10-15

Added a [physical signal-cycle check](verification.md#physical-bluez-signal-check)
requiring target-specific BlueZ traffic and independent disconnected/reconnected
snapshots. The offline suite passes; the physical cycle remains unexecuted.
The connected Linux headset is available, but manual power cycling, official-app
captures and a native Windows test machine are currently unavailable, as confirmed
by the user. GitHub's release-workflow lookup returned HTTP 404 for `release.yml`
on the default branch; remote release artifacts remain unverified. No candidate
capability was enabled and no hardware validation item was marked complete.

Added finished-ZIP verification before release-workflow upload: outer checksum,
exact manifest inventory, internal hashes, executable identity, fixed ZIP metadata,
clean provenance and source revision. Fourteen Python tests pass, including
tampering and mismatched-provenance cases. A fresh locked Linux release build was
packaged as a dirty development bundle; all 190 files verified against its
manifest and the executable matched the supplied build. The clean-release gate
correctly refused that development bundle. Workflow YAML, documentation links and
diff whitespace checks pass. This improves release validation tooling; native
Windows execution and remote artifact verification remain outstanding.

The [APK re-audit](apk-analysis-2026-10-15.md) adds reproducible static source
evidence, identifies targeted dual-close, and corrects MAC-bound LDAC from
`0x0B` to `0x1E` in the offline builder. It also withdraws the unsupported
finder tone-source claim and records the disabled local Studio Pro finder flag.
No new physical evidence or capability promotion follows from APK analysis.

## User-requested experimental mode — 2026-10-15

The user explicitly opted into APK-derived runtime features. Added session-only
`--experimental` to CLI and TUI, with exact firmware `0.2.5` preflight and a
separate runtime allowlist. Verified model capability metadata is unchanged.
Peer/codec/ID/color/box-MAC queries, LDAC, volume/channel, buttons/AI, sharing and
targeted dual-close are accessible. Reset, Finder, speculative scalar high-quality
and independent peer reconnect/switch remain excluded.

The peer page now queries on entry and `r`, displays reported slot/name/MAC and
distinguishes unknown from explicitly empty. New queries, writes and target/session
changes invalidate old peers. `1`/`2` opens a close review; that operation checks
the current slot/MAC, disables dual mode and verifies peer absence. `R` retains an
explicit core refresh for uncertain-write recovery. Cancellation during peer
preflight prevents the close write. All eleven catalogs include the new operation.

Native Linux read-only CLI trials on the selected Studio Pro firmware 0.2.5
returned a complete one-peer list and codec pairs; see the
[redacted observation](evidence/experimental-reads-2026-10-15.json). No control,
host disconnect, reset or pairing change was performed. These observations have
no raw-frame or official-app provenance, and do not close the roadmap's independent
hardware or Windows acceptance requirements. Runtime opt-in does not relabel
static source evidence as verified.

The release TUI was also exercised against the connected headset: connect,
open the peer page and quit. Its rendered output contained the reported name and
MAC, exit was clean and terminal attributes were restored; see the
[redacted TUI record](evidence/experimental-tui-2026-10-15.json). No settings or
peer-close action was requested. Final source checks: 333 default-feature and
208 CLI-only tests passed; strict Clippy passed for both variants; Windows MSVC
all-target source check passed. The release build and ten offline PTY scenarios
pass. Native Windows runtime and physical control verification remain open.

## TUI access and diagnostics follow-up

Screenshots exposed empty everyday Multipoint categories and host-only diagnostic
JSON that contradicted the active session header. Added `Ctrl+E` and an App
Settings toggle for experimental mode. The worker updates both the current native
client and future connections before the UI changes permissions; enabling mode
needs no reconnect and sends no hardware command. Enabled APK operations now
appear in normal categories as well as Research. Empty filters show an explanation
and the toggle shortcut. Peer discovery appears before peer closing.

Diagnostics now renders a concise summary using firmware already read by the
same active session. Optional debugfs socket-inventory permission errors are marked
as nonblocking; a matching socket may belong to this application. CLI `doctor`
remains host-only. All eleven catalogs include the mode and diagnostic labels.
New regressions cover live-session permission changes without reconnect, in-app
access, category contents and contextual firmware. Default tests: 337 passed;
CLI-only: 209 passed. Synthetic Spanish previews cover mode-off/mode-on Multipoint
and active-session diagnostics at 56 and 110 columns. No existing user session
was interrupted to test these UI changes.

## Standard feature access and release candidate — October 15

This update supersedes the earlier experimental-mode UI and permission policy.
Implemented operations are available by default, including peers and targeted
dual-close; exact firmware preflight and uncertain-write recovery remain active.
Historical evidence metadata is unchanged. The legacy CLI flag is a no-op.
Diagnostics hide optional socket inventory failures during an active session,
show available controls, and render audio labels without Rust debug wrappers.

Validation: 335 default-feature Rust tests and 209 CLI-only tests pass; strict
Clippy and Windows MSVC all-target source checks pass for both variants. All 18
Python tests pass. Eleven offline PTY scenarios exercise the optimized release
binary. Synthetic Spanish Multipoint and Diagnostics previews were visually
reviewed at 110 columns, with previews also exported at 56 columns. Formatting,
local documentation links and whitespace checks pass. Linux TUI and CLI-only
binaries build with Rust 1.88.0. No new hardware controls were sent. Native
Windows runtime and per-control physical validation remain open; see
[release readiness](release-readiness.md).
