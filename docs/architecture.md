# Architecture

[Documentation index](README.md) · [Protocol](protocol.md) · [Verification](verification.md)

The project separates local presentation and validation from the Bluetooth transport. The core uses the Rust standard library. The default-enabled `tui` feature adds terminal libraries without replacing the scriptable CLI.

## Source map

| Source | Responsibility |
| --- | --- |
| `src/main.rs` | CLI parsing, command dispatch, bounded profile-file reads, output and exit status |
| `src/tui/mod.rs` | Feature-gated terminal lifecycle, event loop and dispatch |
| `src/tui/state.rs` | Local proposals, confirmation, target selection and stale/busy state |
| `src/tui/view.rs` | Panels, dialogs, small-terminal fallback and safe rendered text |
| `src/tui/worker.rs` | Single-owner device session, bounded work queue, cancellation barriers and write lockout |
| `src/settings.rs` | Studio Pro keys, byte mappings, device-info interpretation, profile parsing/export |
| `src/protocol.rs` | Request encoder, both CRC algorithms, bounded streaming response decoder, hex utilities |
| `src/client.rs` | Request/response matching and verified setting application over a generic `Read + Write` stream |
| `src/transport.rs` | Address/channel validation, deadline helpers, shared connection API and platform selection |
| `src/transport/linux.rs` | Native nonblocking RFCOMM sockets; explicit BlueZ paired-cache listing |
| `src/transport/windows.rs` | Native Winsock/RFCOMM and Microsoft cached paired-device enumeration |
| `src/transport/unsupported.rs` | Clear errors where no Bluetooth backend is provided |
| `tests/cli.rs` | Process-level offline CLI behavior |

Use the source tree as the authority if filenames change. Module tests live alongside implementation; CLI integration tests execute the built binary.

## Data flow

A CLI setting request follows this path:

1. Parse options and validate `Setting`; for profiles, validate every line before connecting
2. Require an explicit address and `studio-pro` protocol selection
3. Open an RFCOMM connection on the chosen channel
4. Read a valid device-info response as preflight
5. Encode the selected instruction and payload using request MODBUS CRC
6. Use `write_all`; wait for a CRC-valid response with matching instruction and a nonzero success byte
7. Query device info again; compare the requested value with the relevant field
8. Report verified success only if readback matches

The TUI uses the same protocol, settings and client layers. Its local proposed value is not authoritative device state. Explicit connection/refresh/write actions are dispatched away from the drawing/input loop. A fatal worker failure exits with an error instead of leaving a misleadingly usable session. A confirmation dialog gates writes, unavailable readback fields cannot be changed from the TUI, and repeated keys do not queue duplicate operations; an uncertain result requires an explicit refresh before another write. Cancellation boundaries can stop subsequent work but cannot reverse a write already sent.

No persistence service runs in the background. The application stores no credentials. Its TUI session log retains at most 100 status events in memory; it is not persistent. Profiles are plain local files managed by the user. Connection lifetime is scoped to the CLI operation or the TUI session.

## Client and transport boundary

`Client<T>` depends on `Read + Write`, making packet behavior testable with mocked streams. It filters decoded responses by instruction and checks the success flag. `flush()` only drains local buffering; it is not proof the device received or accepted a setting.

The native transport gives connect/read/write operations deadlines. `read_exact` and `write_all` share a deadline across their partial calls. A client transaction can involve several operations, and a status/set/profile operation can involve several transactions. The configured timeout is therefore not a strict total wall-clock limit for an entire user command. OS scheduling can also exceed a deadline slightly.

The wire protocol has no request sequence number here. Correlation is by instruction, and this implementation performs requests sequentially. A CRC-valid acknowledgement alone does not establish the desired final state; readback is essential. Incompatible firmware or delayed same-instruction responses cannot be ruled out by the framing layer alone.

## Platform implementation

### Linux

Uses native nonblocking `AF_BLUETOOTH`/RFCOMM stream sockets with `poll`-based readiness, `OwnedFd` cleanup and `MSG_NOSIGNAL` on writes. Addresses are converted to the byte order expected by BlueZ's C layout. The code rejects CPU ABIs for which its hard-coded native constants have not been selected.

Paired-device listing is a distinct, explicit action using `bluetoothctl --timeout 5 devices Paired`. The child process is bounded by a six-second collection deadline and 1 MiB output limits per stream. Unexpected output and missing controllers are errors. No scan, pair, power, or adapter-setting command is run.

### Windows

Uses `AF_BTH`, `SOCK_STREAM` and `BTHPROTO_RFCOMM` with nonblocking Winsock I/O, `select` readiness and scoped resource cleanup. Device enumeration calls `BluetoothFindFirstDevice`/`BluetoothFindNextDevice` with inquiry disabled and authenticated-device filtering.

The FFI definitions deliberately preserve SDK layout details, particularly the packed 30-byte `SOCKADDR_BTH`. Target-specific layout tests are valuable but must actually run on the target to be runtime evidence. Source checking alone is not that evidence.

## Parser and input boundaries

- Response payloads have a one-byte length; complete response frames are at most 263 bytes
- The streaming decoder retains at most 263 pending bytes, accepts fragmented/coalesced reads, checks CRC and resynchronizes after rejected input
- A corrupted length can defer recovery until more bytes arrive or a timeout occurs; it is never treated as success
- Device info shorter than eight bytes is rejected; later optional fields are accessed defensively
- Profile text is capped at 16 KiB; invalid or duplicate settings are rejected before writes
- Device names shown in the CLI have control characters escaped; the TUI additionally replaces invisible/bidirectional control marks and bounds displayed text. Names are labels, never identities or commands

`decode` is deliberately stricter than live stream handling: it fails if any input was discarded, pending, or rejected, even when one valid frame was recovered. The live client may ignore unrelated frames while waiting for its response.

## Deliberate boundaries

The core has one model mapping, not a universal UGREEN protocol abstraction. Adding another model requires separate researched mappings and tests, not an alias. There is no codec inference, firmware updater, factory reset, arbitrary raw write, audible locator, actual-volume adjustment or ANC-button remapping.

Profiles have sequential, best-effort application with verification per setting. Automatic retries and rollback are deliberately absent because acknowledgement loss leaves device state uncertain. See [Usage](usage.md) for recovery after partial application.
