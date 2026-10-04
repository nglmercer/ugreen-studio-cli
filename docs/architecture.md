# Architecture

[Documentation index](README.md) · [Protocol](protocol.md) · [Verification](verification.md)

The project separates local presentation and validation from the Bluetooth transport. The core uses the Rust standard library. The default-enabled `tui` feature adds terminal libraries without replacing the scriptable CLI.

## Source map

| Source | Responsibility |
| --- | --- |
| `src/main.rs` | CLI parsing, command dispatch, bounded profile-file reads, output and exit status |
| `src/tui/mod.rs` | Feature-gated terminal lifecycle, event loop and dispatch |
| `src/tui/state.rs` | Local proposals, per-address target/model selection and stale/busy state |
| `src/tui/view.rs` | Panels, dialogs (including the paired-device list with link/model rows), small-terminal fallback and safe rendered text |
| `src/tui/worker.rs` | Single-owner device session, bounded work queue, cancellation barriers and write lockout |
| `src/settings.rs` | Studio Pro keys, byte mappings, `StudioProState` device-info interpretation, profile parsing/export |
| `src/models/` | Model identity and per-model capability bits; a model is user-confirmed, never detected |
| `src/device/registry.rs` | Per-address device registry (selected target, confirmed model, channel/timeout/language), v1+v2 parsing, atomic bounded writes |
| `src/bluetooth/` | Host-side paired/link enumeration (`HostHeadset`), address identity and the RFCOMM control transport; Linux D-Bus + fallback, Windows APIs |
| `src/multipoint.rs` | Headset multipoint scaffolding: tri-state state, capability gates, `UnsupportedFeature` refusals |
| `src/protocol/` | Request encoder, both CRC algorithms, streaming decoder (response/notification/unknown), session monitor, hex utilities |
| `src/client.rs` | Request/response matching, verified setting application and guarded peer-operation refusals over a generic `Read + Write` stream |
| `src/i18n/` | Interface text catalogs with placeholder-parity tests |
| `tests/cli.rs` | Process-level offline CLI behavior |
| `tests/captures.rs` | Fixture-driven decoder tests over `tests/fixtures/protocol/` |

Use the source tree as the authority if filenames change. Module tests live alongside implementation; CLI integration tests execute the built binary.

## Data flow

A CLI setting request follows this path:

1. Parse options and validate `Setting`; for profiles, validate every line before connecting
2. Require an explicit address and `studio-pro` protocol selection
3. Open an RFCOMM connection on the chosen channel, wrapped in the recording tee
4. Read a valid device-info response as preflight
5. Encode the selected instruction and payload using request MODBUS CRC
6. Use `write_all`; wait for a CRC-valid response with matching instruction and a nonzero success byte
7. Query device info again; compare the requested value with the relevant field
8. Report verified success only if readback matches

Every CLI connection passes through `protocol::Monitored`, a transparent tee that records each written and read byte in order. Most commands ignore the log; `capture` exports it. The tap never alters bytes, so a failed session still produces a complete record of what actually crossed the wire.

The TUI uses the same protocol, settings and client layers. Its local proposed value is not authoritative device state. Explicit connection/refresh/write actions are dispatched away from the drawing/input loop. A fatal worker failure exits with an error instead of leaving a misleadingly usable session. A confirmation dialog gates writes, unavailable readback fields cannot be changed from the TUI, and repeated keys do not queue duplicate operations; an uncertain result requires an explicit refresh before another write. Cancellation boundaries can stop subsequent work but cannot reverse a write already sent.

No persistence service runs in the background. The application stores no credentials. The only persistent state is `~/.config/ugreen-cli/state` (overridable with `UGREEN_STATE_FILE`), a bounded per-address registry of the user's own explicit choices: selected target, confirmed model, channel, timeout and language. It never stores a detected model. The TUI session log retains at most 100 status events in memory; it is not persistent. Profiles are plain local files managed by the user. Connection lifetime is scoped to the CLI operation or the TUI session.

## Client and transport boundary

`Client<T>` depends on `Read + Write`, making packet behavior testable with mocked streams. It filters decoded responses by instruction and checks the success flag. `flush()` only drains local buffering; it is not proof the device received or accepted a setting.

The native transport gives connect/read/write operations deadlines. `read_exact` and `write_all` share a deadline across their partial calls. A client transaction can involve several operations, and a status/set/profile operation can involve several transactions. The configured timeout is therefore not a strict total wall-clock limit for an entire user command. OS scheduling can also exceed a deadline slightly.

The wire protocol has no request sequence number here. Correlation is by instruction, and this implementation performs requests sequentially. A CRC-valid acknowledgement alone does not establish the desired final state; readback is essential. Incompatible firmware or delayed same-instruction responses cannot be ruled out by the framing layer alone.

Multipoint peer operations are the same story one level up: `Client::multipoint_peers` and the peer disconnect/reconnect/switch methods return `UnsupportedFeature` before assembling any frame. No model capability verifies a peer-management protocol yet, so a refusal is final and immediate, not a timeout, and no `peers` subcommand exists to reach them.

## Platform implementation

### Linux

Uses native nonblocking `AF_BLUETOOTH`/RFCOMM stream sockets with `poll`-based readiness, `OwnedFd` cleanup and `MSG_NOSIGNAL` on writes. Addresses are converted to the byte order expected by BlueZ's C layout. The code rejects CPU ABIs for which its hard-coded native constants have not been selected.

Paired-device listing is a distinct, explicit action with two bounded paths. The primary path reads the system D-Bus (`GetManagedObjects`) without scanning, pairing, or powering the adapter. When the bus is unavailable it falls back to `bluetoothctl --timeout 5 devices Paired`, whose child process is bounded by a six-second collection deadline and 1 MiB output limits per stream. Unexpected output and missing controllers are reported instead of guessed. No scan, pair, power, or adapter-setting command is run on either path. Direct connection does not require either listing path.

### Windows

Uses `AF_BTH`, `SOCK_STREAM` and `BTHPROTO_RFCOMM` with nonblocking Winsock I/O, `select` readiness and scoped resource cleanup. Device enumeration calls `BluetoothFindFirstDevice`/`BluetoothFindNextDevice` with inquiry disabled and authenticated-device filtering.

The FFI definitions deliberately preserve SDK layout details, particularly the packed 30-byte `SOCKADDR_BTH`. Target-specific layout tests are valuable but must actually run on the target to be runtime evidence. Source checking alone is not that evidence.

## Parser and input boundaries

- Response payloads have a one-byte length; complete response frames are at most 263 bytes
- The streaming decoder retains at most 263 pending bytes, accepts fragmented/coalesced reads, checks CRC and resynchronizes after rejected input
- The decoder emits three frame shapes: CRC-valid responses, fixed six-byte notifications, and well-framed `AA BB CC` input with no verified RX meaning (counted as unknown, never read as status); everything else is counted and discarded
- A corrupted length can defer recovery until more bytes arrive or a timeout occurs; it is never treated as success
- Device info shorter than eight bytes is rejected; later optional fields are accessed defensively
- Profile text is capped at 16 KiB; invalid or duplicate settings are rejected before writes
- Device names shown in the CLI have control characters escaped; the TUI additionally replaces invisible/bidirectional control marks and bounds displayed text. Names are labels, never identities or commands

`decode` is deliberately stricter than live stream handling: it fails if any input was discarded, pending, rejected, or decoded as an unknown frame, even when one valid frame was recovered. Notifications count as decoded frames (their event mapping stays unknown) rather than noise. The live client may ignore unrelated frames while waiting for its response.

## Deliberate boundaries

The core has one model mapping, not a universal UGREEN protocol abstraction. Adding another model requires separate researched mappings and tests, not an alias. There is no codec inference, firmware updater, factory reset, arbitrary raw write, audible locator, actual-volume adjustment or ANC-button remapping. Multipoint peer management (list, disconnect, reconnect, switch) stays refused until a Studio Pro capture from the official app verifies those frames; the verified dual-device toggle (`dual`, instruction `0x06`) is a separate two-radio switch and proves nothing about a peer protocol.

Profiles have sequential, best-effort application with verification per setting. Automatic retries and rollback are deliberately absent because acknowledgement loss leaves device state uncertain. See [Usage](usage.md) for recovery after partial application.
