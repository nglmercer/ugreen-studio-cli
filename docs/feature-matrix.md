# Feature matrix — October 14, 2026

Baseline `main` / `origin/main`: `b15bef554b29543b2f3e905149daaa9aa0e151f0`.
Fetched before editing; both references matched. The starting workspace contained
uncommitted feature-registry, TUI, settings and eleven-language catalog changes.
Those changes were retained and their broken imports, forms and tests repaired.

## Evidence and implementation are separate

Evidence states: `verified`, `experimental_read`, `static_candidate`, `unsupported`.
Implementation states: `not_started`, `implemented`, `tested_mock`,
`tested_platform`, `tested_hardware`. A mock test never promotes evidence.

`verified` below describes the historical HP206 firmware **0.2.5** record in
[verification](verification.md#live-retail-verification-2026-10-04), not a new
hardware session. No new physical capture, native Windows runtime test, audio
playback, reset or host link mutation was performed for this implementation.
Unknown model/firmware lookup grants zero capabilities. CLI setting/profile writes
and native TUI setting preflight now require an actual `0.2.5` firmware reply.
The historical `Model::capabilities()` is model metadata; use
`models::capabilities_for()` for firmware-specific authorization. The evidence-aware `Client::set_for_model()` queries firmware and preflights settings. The public
low-level `Client::request`, `send` and `set` APIs retain compatibility and require
caller-managed identity/evidence; they are not safe automatic model detection.

## Inventory

| Feature / operation | Platform | Implementation | Evidence | Provenance / firmware | CLI / TUI | Coverage / limitation |
| --- | --- | --- | --- | --- | --- | --- |
| Device info, battery, firmware | Linux | tested_hardware | verified | Historical retail HP206 / 0.2.5 | status, profile export, capture / header | CRC, fragmentation, notifications; no new hardware run |
| ANC, eight EQ presets | Linux | tested_hardware | verified | Historical retail record and protocol fixtures / 0.2.5 | set, profile apply / settings | Preflight, ack, matching readback |
| Game, spatial, dual-device, wind | Linux | tested_hardware | verified | Historical retail / 0.2.5 | set, profile apply / settings | Spatial uses readback without ack; dual independent of peers |
| Prompts, two volume-hold actions | Linux | tested_hardware | verified | Historical retail / 0.2.5 | set, profile apply / settings | Hold actions do not change playback volume |
| Same protocol on Windows RFCOMM | Windows | tested_mock | verified | Headset protocol evidence above / 0.2.5 | Same commands / TUI | Source cross-check only; native Windows not validated |
| Cached paired/host devices | Linux | tested_platform | verified | Historical live BlueZ enumeration; host evidence, no headset firmware | discover, devices / p | Exact addresses, sorting, registry model isolation |
| Cached devices | Windows | tested_mock | unsupported | Supported Win32 implementation, native validation absent | discover, devices / p | Source checks; current runtime evidence unavailable |
| Typed RFCOMM lifecycle and independent OS link | Linux, Windows | tested_mock | unsupported | Offline state tests; no new retail validation | devices JSON / header | Connected, connecting, disconnected, error; host observation never overwritten by RFCOMM |
| Explicit host connect/disconnect | Linux | tested_mock | unsupported | BlueZ Device1 methods; no live mutation validation | host connect/disconnect / p then h/H | Existing bounded D-Bus path; exact identity and adapter ambiguity checks |
| Host connect/disconnect | Windows, other | implemented | unsupported | No implemented legitimate host-control API | Same commands return actionable error | Never equates socket close with host disconnect |
| Host periodic refresh | Linux, Windows | tested_mock | unsupported | Five-second bounded worker fallback | TUI automatically when idle | Coalesced by one-request worker, request IDs and target matching; signal subscription pending |
| BlueZ PropertiesChanged listener | Linux | not_started | unsupported | No listener backend yet | None | Fallback refresh exists; no signal-monitoring claim |
| OS audio profile/codec/activity | Linux | tested_mock | unsupported | PipeWire Node transport properties; not a headset packet | audio status, JSON / A dedicated view | Address matching, explicit transport required, conflicting nodes preserved, timestamps; live BT transport validation pending |
| OS negotiated codec | Windows, other | implemented | unsupported | No supported observable provider implemented | audio status returns unavailable | No version/installed-codec inference; ETW not implemented |
| BlueZ MediaTransport fallback | Linux | tested_mock | unsupported | BlueZ interface definitions and synthetic D-Bus fixtures; no live Bluetooth validation | audio status / Audio view | Exact Device1 path association, standard IDs and company/codec vendor IDs; bare 0xff stays unknown |
| Audio sink selection inventory | Linux | tested_mock | unsupported | PipeWire Audio/Sink node names | audio devices, audio-devices | Explicit output name; no default routing fallback; playback preview pending |
| Audio output inventory / playback | Windows | tested_mock | unsupported | Existing waveOut backend; native validation absent | audio devices, audio-devices / feature form | No audio playback claim |
| Multipoint list 0x0D | Both | tested_mock | static_candidate | APK hypotheses and synthetic peers; firmware unknown | peers refuses / feature workspace | Zero/list completion, fragmentation, CRC, notifications, timeout; never substitutes host list |
| Remote peer disconnect/reconnect/switch | Both | implemented | unsupported | No established independent classic commands | Disabled | Zero-byte refusal; host action scope remains explicit |
| Vendor codec 0x0F, acoustics_code | Both | tested_mock | static_candidate | APK/synthetic layout; firmware unknown | codec refuses, candidate / workspace | Independent of OS codec; no physical capture |
| ID, color, Box MAC queries | Both | tested_mock | static_candidate | APK/synthetic layout; firmware unknown | id, color, box-mac refuse / workspace | Before-I/O gates and strict parsers |
| LDAC preference, peer LDAC, high-quality decoding | Both | tested_mock | static_candidate | APK/synthetic layout; firmware unknown | control refuses, candidate / forms | Independent bits, ack/readback scaffolding; never promises negotiation |
| Volume, channel mode | Both | tested_mock | static_candidate | APK/synthetic layout; firmware unknown | control refuses, candidate / forms | Range/layout mocks only |
| Button mappings, AI actions/control | Both | tested_mock | static_candidate | APK/synthetic layout; firmware unknown | control refuses, candidate / forms | No arbitrary raw normal CLI endpoint |
| Audio sharing off/single/peer 0x16 | Both | tested_mock | static_candidate | APK/synthetic layout; firmware unknown | control refuses, candidate / forms | Wind interaction remains hypothesis, never enabled from fixture |
| Finder start/stop 0x07 | Both | tested_mock | static_candidate | APK/synthetic layout; firmware unknown | find refuses, candidate / forms | Stop on failed start/playback, cancellation, bounded tones; no physical audibility test |
| Factory reset | Both | tested_mock | static_candidate | APK/synthetic packet; firmware unknown | control refuses, candidate / destructive review | Ack/no-retry/close/cleanup mocks; exact destructive confirmation phrase still pending |
| Post-reset host cleanup | Linux, Windows | tested_mock | unsupported | Existing BlueZ/Win32 backend, no actual unpair test | Unreachable until reset verified | Ordering, exact address, registry preservation, partial error handling |
| Offline decode/analyze/candidate | All | tested_mock | unsupported | Offline utilities have no live device capability | CLI / workspace instructions | Fixtures stay available without hardware; no live authority |
| Read-only JSON schema v1 | All | tested_mock | unsupported | Software contract only | status, devices, audio status, capabilities, doctor | Prose/diagnostics stay off stdout; capabilities do not grant authorization |
| Doctor | Linux, Windows | tested_mock | unsupported | Host enumeration and optional address-matched audio | doctor, JSON | Read-only; adapter power/controller/read-permission reporting and optional debugfs possible-collision probe; write permission never probed |
| Declarative registry, typed feature forms | All | tested_mock | unsupported | Software UI only | help, aliases / F | Eleven catalog updates, bounded input, request matching, review and lockout |
| Other headset models | All | implemented | unsupported | No model-specific captures supplied | Model parser refuses | No HP206 packet inheritance |
| Release / CI / packaging | Linux, Windows | implemented | unsupported | Existing Linux/Windows CI; no remote results implied | Cargo feature variants | Local build/check results recorded separately; no new release bundle |

No operation has graduated to `experimental_read`: no new target-specific capture
supports such a promotion. `unsupported` on an OS/software row describes absent
physical/platform evidence or missing backend as stated in that row; it does not
mean an offline implementation emits headset packets.

## Milestone delivery and remaining work

M0 audit and safety baseline are implemented. M1 includes independent states,
current-target sorting, session-safe device browsing, explicit Linux host actions,
and bounded worker refresh; signal monitoring, complete per-device history,
reconnect/backoff and detailed permissions/adapter UX remain. M2 includes the
bounded PipeWire and BlueZ MediaTransport providers and honest unavailable Windows backend;
real platform transport validation remains. M3/M4 retain offline
parsers, orchestration and gates; official-app and retail captures remain required.
M5 includes registry commands, JSON, initial doctor and repaired typed feature UI;
six-page navigation and dedicated Audio/Diagnostics views are implemented; signal monitoring,
remaining diagnostic probes and release bundles remain.
M6 adds exact firmware lookup; low-level SDK authorization still depends on the
caller. This matrix does **not** declare the full work order complete.

## Reproducible checks

The starting dirty workspace failed `cargo fmt --all -- --check` and
`cargo test --all-features` (unresolved feature module, view helpers and catalog
fields). Current results, including offline PTY and cross-target checks, are in
[verification](verification.md). Existing original fixture provenance is in
[reverse engineering](reverse-engineering.md); fixtures were not renamed into a
verified firmware directory or relabeled as physical captures.

## Exact working-tree file inventory

This inventory includes pre-existing edits retained from the starting workspace;
it is not an attribution of all changes to this implementation. The deleted
`src/tui/features.rs` was already deleted at entry; its compatibility parser is
preserved in the shared `src/features.rs` registry.

- `Cargo.lock`
- `Cargo.toml`
- `README.es.md`
- `README.md`
- `docs/README.md`
- `docs/architecture.md`
- `docs/feature-matrix.md`
- `docs/reverse-engineering.md`
- `docs/troubleshooting.md`
- `docs/usage.md`
- `docs/verification.md`
- `src/audio/bluez.rs`
- `src/audio/mod.rs`
- `src/bluetooth/dbus.rs`
- `src/bluetooth/device.rs`
- `src/bluetooth/linux.rs`
- `src/bluetooth/mod.rs`
- `src/cli/context.rs`
- `src/cli/handlers/app.rs`
- `src/cli/handlers/device.rs`
- `src/cli/handlers/host.rs`
- `src/cli/handlers/mod.rs`
- `src/cli/handlers/settings.rs`
- `src/cli/help.rs`
- `src/cli/parser.rs`
- `src/cli/registry.rs`
- `src/client.rs`
- `src/diagnostics.rs`
- `src/features.rs`
- `src/i18n/de.rs`
- `src/i18n/en.rs`
- `src/i18n/es.rs`
- `src/i18n/fr.rs`
- `src/i18n/it.rs`
- `src/i18n/ja.rs`
- `src/i18n/ko.rs`
- `src/i18n/mod.rs`
- `src/i18n/nl.rs`
- `src/i18n/pt.rs`
- `src/i18n/ru.rs`
- `src/i18n/zh.rs`
- `src/lib.rs`
- `src/models/mod.rs`
- `src/models/studio_pro.rs`
- `src/process.rs`
- `src/settings.rs`
- `src/tui/features.rs`
- `src/tui/mod.rs`
- `src/tui/state.rs`
- `src/tui/tests.rs`
- `src/tui/view.rs`
- `src/tui/worker.rs`
- `tests/cli.rs`
- `third-party/RUST-DEPENDENCIES.md`
- `third-party/cargo/0f96a83840e146e43c0e.txt`
- `third-party/cargo/7e12e5df4bae12cb2158.txt`
