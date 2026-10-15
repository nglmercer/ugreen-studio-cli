# Release readiness

Implemented device features are available by default in the CLI and TUI. No
experimental flag, toggle or reconnect is required. Studio Pro model selection,
firmware 0.2.5 preflight, typed arguments, acknowledgement/readback checks,
cancellation and uncertain-write recovery remain part of normal execution.

## Included functionality

- Core status/firmware and the nine established settings; configuration/profiles.
- Headset peer names, MACs and slots, complete/empty/unknown handling and JSON.
- Targeted peer close with dual-mode disable, current slot/MAC validation and
  post-operation dual/peer readback.
- Codec, ID, color and box-MAC queries.
- LDAC preference and MAC-bound LDAC, volume, mono/stereo, buttons/AI and sharing.
- Linux host/audio observations, concise session-aware diagnostics and CLI JSON.
- Eleven-language TUI, persistent navigation, cancellation and terminal restoration.
- Deterministic Linux/Windows packaging workflow, file manifests and ZIP verifier.

The ordinary UI shows available functionality. Research contains offline tools
and unavailable operations. Source provenance remains separate from physical
verification: enabling an implementation is not a new hardware test result.

## Unsupported operations

Finder is disabled in the app's local Studio Pro configuration, and its tone
source is unestablished. Independent peer reconnect/switch/disconnect while
preserving dual mode has no established packet mapping. Factory reset and the
scalar high-quality hypothesis remain unavailable. Targeted close is explicitly
named because it also disables dual mode.

## Completion gates

Local builds, tests and archive verification are recorded in
[verification](verification.md) and the [implementation checklist](implementation-progress.md).
The local worktree contains uncommitted changes, so bundles made from it are
marked as development/release-candidate artifacts; they are not clean published
releases.

The following evidence is still required before claiming the entire roadmap and
native-platform release verification complete:

- Native Windows Bluetooth/audio/headset runtime validation.
- Independent per-control acknowledgement/readback and physical outcome checks.
- Identified HP206 official-app codec/peer captures and real BlueZ state-change
  delivery validation.
- Clean-source release provenance, remote workflow runs and downloaded artifact
  verification for the exact release revision.

No hardware setting, reset, host disconnect or pairing mutation was performed
while preparing this release candidate. Earlier read-only peer/codec observations
are linked in Verification. No public release was published.
