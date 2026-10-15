# ugreen-cli

[Español](README.es.md) · [Documentation](docs/README.md)

An unofficial Rust CLI and terminal interface for configuring **UGREEN Studio Pro** headphones locally on Linux and Windows. Not affiliated with or endorsed by UGREEN.

> **Hardware support.** The implementation follows the Studio Pro **HP206** reference protocol and an upstream captured response, verified live against retail firmware **0.2.5** on Linux (reads, all nine writes incl. a reversible cycle; see [verification](docs/verification.md)). Model selection is still user-declared, not device-identity detection. **HiTune Max5c is unsupported:** some command IDs have different meanings.

## Quick start

Install [Rust](https://rust-lang.org/tools/install/) and the platform prerequisites in [Building](docs/building.md), then run from the project directory:

```sh
cargo build --release --locked
cargo run --release --locked -- tui
```

The executable is `target/release/ugreen` on Linux or `target\release\ugreen.exe` on Windows. Examples below use `ugreen`; substitute that path if the executable is not on your `PATH`.

The earlier delivery ZIP included Linux x86-64 executables: `bin/linux-x86_64/ugreen` (TUI + CLI) and `bin/linux-x86_64/ugreen-cli-only`. Both require glibc 2.39 or newer and `libgcc_s.so.1`; build locally on older distributions. The new per-platform packaging workflow is described in [building](docs/building.md#release-bundles); remote artifacts remain unverified.

On start the TUI reconnects to the last confirmed device (on by default; `--no-autoconnect` opts out). With no cached target it starts disconnected: enter an address with `a`, or explicitly load the OS's paired-device cache with `p`; press `m` then `y` to confirm the Studio Pro protocol, then press `c` to connect and read status. Pair the headphones in your OS settings first. Opening the interface, choosing a device, or editing a proposed value does not write settings.

For a first offline check:

```sh
ugreen --help
ugreen models
ugreen --dry-run set anc ultra
```

For a deliberate device read, replace the example address with your headphones' address:

```sh
ugreen discover
ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro status
```

## Device features

Peer discovery, codec/ID/color/box-MAC reads, LDAC, volume/channel, button and AI
controls, audio sharing and targeted peer closing are available in the normal
CLI and TUI. No mode switch or extra flag is required. Select Studio Pro and
connect; firmware `0.2.5` is checked before device operations.

```sh
ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro peers
ugreen --json --address AA:BB:CC:DD:EE:FF --model studio-pro peers
ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro codec
ugreen tui
```

Press `P` for headset-connected peers or browse Multipoint in the normal feature
categories. Closing a peer also disables dual mode and requires confirmation in
the TUI. Reset, Finder, independent reconnect/switch and the speculative one-byte
high-quality command remain unavailable. See [device operations](docs/usage.md#supported-device-operations)
and [release readiness](docs/release-readiness.md) for validation scope.

## Terminal interface or CLI

The default `tui` Cargo feature uses Ratatui 0.30.2 and Crossterm 0.29.0. `ugreen tui` opens the interface explicitly. Running with no arguments opens it only when both stdin and stdout are terminals; otherwise it prints help. Explicit CLI commands stay scriptable.

- `v`: page selector; `A`: Audio; `D`: Diagnostics; `F`: feature workspace; `a`: edit address; `p`: load host devices (also during a session); `m`, then `y`: confirm model protocol
- `c`: connect; `r`: refresh status; `d`: disconnect
- Up/Down: select a setting; Left/Right: change its proposed value
- Enter: review a change; Enter again: confirm it; Esc: cancel a dialog
- `l`: session log; `?`: help; `q`: quit

A proposed value is not a device value. A change is verified only after a valid acknowledgement and matching readback. Battery comes from device-info bytes; unavailable values remain unknown. Linux OS codec observations are available through `audio status` or TUI `A` when PipeWire reports an address-matched transport. The vendor codec query and codec controls remain gated. See [Usage](docs/usage.md) for the full workflow and failure behavior.

To omit the TUI dependencies and keep the CLI without terminal dependencies:

```sh
cargo build --release --locked --no-default-features
cargo run --locked --no-default-features -- --help
```

The package declares Rust 1.88 or newer. Default builds download third-party terminal dependencies; a no-default-features build does not compile them. Offline Cargo builds still require any necessary toolchains, registry metadata and dependencies to be cached.

## Supported settings

| Setting | Values |
| --- | --- |
| `anc` | `off`, `ultra`, `general`, `gentle`, `adaptive`, `ambient` |
| `eq` | `classic`, `jazz`, `electronic`, `pop`, `classical`, `rock`, `bass`, `treble` |
| `game`, `spatial`, `dual`, `wind` | `on`, `off` |
| `prompts` | `voice`, `beeps` |
| `volume-up-action`, `volume-down-action` | `none`, `next`, `previous` |

The last two settings map **button hold actions**, not playback loudness. Presets and values are exact, case-sensitive protocol names. No custom EQ curve or codec selector is implemented.

```sh
ugreen commands
ugreen --dry-run set eq bass
ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro set eq bass
ugreen profile example > my-profile.conf
ugreen --dry-run profile apply my-profile.conf
ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro profile apply my-profile.conf
ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro profile export > saved-profile.conf
```

Options precede the command. `--channel` accepts 1–30 (default 1); `--timeout` accepts 1–60 seconds (default 3) per operation. The timeout is not a total budget for a multi-request operation or whole profile.

Profiles are UTF-8 `key=value` text, at most 16 KiB, with mandatory `model=studio-pro`. Blank lines and full-line `#` comments are accepted. Duplicate or unknown keys and invalid values fail before Bluetooth access. Profiles apply sequentially and **can partially succeed**; there is no rollback. See the [profile guide](docs/usage.md#profiles), including Windows text encoding advice.

## Safety and limits

- Hardware requests need an explicit address and Studio Pro protocol selection. Model selection is not device-identity detection
- `discover` reads cached paired devices. There is no active scan, pairing, or automatic target choice; the TUI only ever reconnects to your last confirmed device (on by default, `--no-autoconnect` opts out) and never picks an unknown device
- Peer reads and targeted dual-close are normal operations; independent reconnect/switch remain unavailable. Targeted close also disables dual mode
- Actual firmware `0.2.5` and a valid device-info preflight are required before CLI/TUI setting writes; matching readback is required to report success
- Writes are not automatically retried. A timeout may mean a setting changed but its acknowledgement was lost; query status before trying again
- No firmware updates, factory reset, raw-transmit command or Finder. Mapped volume/button controls use the normal firmware-checked operation path
- No telemetry, network service, account, or stored credentials; local Bluetooth control only

A phone app may already occupy the RFCOMM control channel. Close its control session before retrying; do not reset or unpair the headphones merely because a request fails. Read [Troubleshooting](docs/troubleshooting.md) before repeated writes.

## Offline capture inspection

```sh
ugreen decode 'DD EE FF 04 01 1E 14 FF FF A0 00 01 00 00 08 0B 00 07 00 00 00 00 02 00 00 09 00 00 04 05 00 00 00 0C 0D 0E D0 E3'
```

This validates the upstream fixture without Bluetooth access. Request CRC is MODBUS; response CRC is CCITT-FALSE. Malformed, noisy, incomplete, checksum-invalid, or unknown-framed capture streams exit nonzero. Six-byte notifications decode with `event=unknown`.

To record a live session instead, `capture` tees every TX/RX byte and prints fixture-style hex on stdout (context on stderr), so it can be saved, fed back to `decode`, or reviewed as a future fixture:

```sh
ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro capture > session.hex
ugreen decode "$(cat session.hex)"
```

`#` starts a comment in hex input. The [protocol notes](docs/protocol.md) explain the evidence and decoder limits.


Persistent navigation: `F1` Home, `F2` Audio controls, `F3` Devices, `F4` Settings,
`F5` Diagnostics and `F6` Research. The main browser keeps candidates in Research.
Language changes reach the worker, and configuration persists in the native Linux
or Windows config directory. JSON output also supports `audio devices` and its
`audio-devices` alias. See the [progress checklist](docs/implementation-progress.md).

A fresh read-only Linux session returned firmware 0.2.5 and CRC-valid core replies;
the [redacted evidence](docs/evidence/core-read-2026-10-14.json) records its limited
scope. Mapped operations are enabled by default; source provenance and independent hardware evidence remain documented separately.

## Documentation and verification

Start with the [documentation index](docs/README.md): [build instructions](docs/building.md), [usage](docs/usage.md), [architecture](docs/architecture.md), [protocol/provenance](docs/protocol.md), [verification](docs/verification.md), and [troubleshooting](docs/troubleshooting.md).

Linux and Windows have native RFCOMM backends. The [CI workflow](.github/workflows/ci.yml) provides platform checks; having a workflow does not mean it has run remotely. Compilation and mock tests do not establish physical headphone compatibility or Windows runtime success. macOS and other platforms have no Bluetooth backend in this project.

[Serena](docs/serena-pilot.md) is optional development tooling. Its earlier pilot is historical, not a current installation or a requirement to build or run this application.

## License and attribution

[MIT](LICENSE). Protocol port and captured fixture adapted from [sanild/ugreen-studio-controller-macos](https://github.com/sanild/ugreen-studio-controller-macos). Framing/CRC research was cross-checked against [tzy3454u/ugreen-headphone-control](https://github.com/tzy3454u/ugreen-headphone-control). Original MIT notices are retained in [third-party](third-party/); keep them when redistributing. Terminal dependencies have their own licenses; see the [Rust dependency inventory](third-party/RUST-DEPENDENCIES.md). Product names and trademarks belong to their respective owners.

## Host diagnostics and evidence

```sh
ugreen --json devices
ugreen --json --model studio-pro capabilities
ugreen --address AA:BB:CC:DD:EE:FF --json audio status
ugreen audio devices
ugreen --address AA:BB:CC:DD:EE:FF --json doctor
ugreen --address AA:BB:CC:DD:EE:FF host connect
ugreen --address AA:BB:CC:DD:EE:FF host disconnect
```

Options precede commands. JSON uses `schema_version: 1` and is limited to
read-only operations. Linux host actions affect the OS Bluetooth link; Windows
returns unsupported and directs you to OS settings. In the TUI device list,
`h` requests host connect and `H` host disconnect. Target changes require closing
the control session; host refresh never overwrites RFCOMM state. Idle host
observations refresh every five seconds through the worker.

Audio observations include source and time, remain distinct from the gated
vendor codec, and never infer a codec from advertised support. Missing transport
properties remain unknown. [The feature matrix](docs/feature-matrix.md) records
implemented parts, missing work and physical-validation limits.

Press `P` for Headset Connections: dual-device state and observation age are shown; peer discovery is available by default. Host paired devices are a separate list.
