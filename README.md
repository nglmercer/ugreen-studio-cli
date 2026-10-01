# ugreen-cli

[Español](README.es.md) · [Documentation](docs/README.md)

An unofficial Rust CLI and terminal interface for configuring **UGREEN Studio Pro** headphones locally on Linux and Windows. Not affiliated with or endorsed by UGREEN.

> **Experimental hardware support.** The implementation follows the Studio Pro **HP206** reference protocol and an upstream captured response. HP206 is a protocol candidate for the retail Studio Pro, not an identity verified on your headphones. This build has not been tested with physical headphones. **HiTune Max5c is unsupported:** some command IDs have different meanings.

## Quick start

Install [Rust](https://rust-lang.org/tools/install/) and the platform prerequisites in [Building](docs/building.md), then run from the project directory:

```sh
cargo build --release --locked
cargo run --release --locked -- tui
```

The executable is `target/release/ugreen` on Linux or `target\release\ugreen.exe` on Windows. Examples below use `ugreen`; substitute that path if the executable is not on your `PATH`.

The delivery ZIP also includes Linux x86-64 executables: `bin/linux-x86_64/ugreen` (TUI + CLI) and `bin/linux-x86_64/ugreen-cli-only`. Both require glibc 2.39 or newer and `libgcc_s.so.1`; build locally on older distributions. No Windows executable is included; use the Windows build instructions.

The TUI starts disconnected. Enter an address with `a`, or explicitly load the OS's paired-device cache with `p`; press `m` then `y` to confirm the Studio Pro protocol, then press `c` to connect and read status. Pair the headphones in your OS settings first. Opening the interface, choosing a device, or editing a proposed value does not write settings.

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

## Terminal interface or CLI

The default `tui` Cargo feature uses Ratatui 0.30.2 and Crossterm 0.29.0. `ugreen tui` opens the interface explicitly. Running with no arguments opens it only when both stdin and stdout are terminals; otherwise it prints help. Explicit CLI commands stay scriptable.

- `a`: edit address; `p`: load paired-device cache; `m`, then `y`: confirm model protocol
- `c`: connect; `r`: refresh status; `d`: disconnect
- Up/Down: select a setting; Left/Right: change its proposed value
- Enter: review a change; `y`: confirm it; Esc: cancel a dialog
- `l`: session log; `?`: help; `q`: quit

A proposed value is not a device value. A change is verified only after a valid acknowledgement and matching readback. Battery comes from device-info bytes; unavailable values remain unknown. Codec status/control is unavailable. See [Usage](docs/usage.md) for the full workflow and failure behavior.

To omit the TUI dependencies and keep the standard-library-only CLI:

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
- `discover` reads cached paired devices. There is no active scan, pairing, automatic target choice, or automatic connection on startup
- A valid device-info preflight precedes setting writes; matching readback is required to report success
- Writes are not automatically retried. A timeout may mean a setting changed but its acknowledgement was lost; query status before trying again
- No firmware updates, factory reset, raw-transmit command, find-headphones sound, actual volume changes, or ANC-button remapping
- No telemetry, network service, account, or stored credentials; local Bluetooth control only

A phone app may already occupy the RFCOMM control channel. Close its control session before retrying; do not reset or unpair the headphones merely because a request fails. Read [Troubleshooting](docs/troubleshooting.md) before repeated writes.

## Offline capture inspection

```sh
ugreen decode 'DD EE FF 04 01 1E 14 FF FF A0 00 01 00 00 08 0B 00 07 00 00 00 00 02 00 00 09 00 00 04 05 00 00 00 0C 0D 0E D0 E3'
```

This validates the upstream fixture without Bluetooth access. Request CRC is MODBUS; response CRC is CCITT-FALSE. Malformed, noisy, incomplete, or checksum-invalid capture streams exit nonzero. The [protocol notes](docs/protocol.md) explain the evidence and decoder limits.

## Documentation and verification

Start with the [documentation index](docs/README.md): [build instructions](docs/building.md), [usage](docs/usage.md), [architecture](docs/architecture.md), [protocol/provenance](docs/protocol.md), [verification](docs/verification.md), and [troubleshooting](docs/troubleshooting.md).

Linux and Windows have native RFCOMM backends. The [CI workflow](.github/workflows/ci.yml) provides platform checks; having a workflow does not mean it has run remotely. Compilation and mock tests do not establish physical headphone compatibility or Windows runtime success. macOS and other platforms have no Bluetooth backend in this project.

[Serena](docs/serena-pilot.md) is optional development tooling. Its earlier pilot is historical, not a current installation or a requirement to build or run this application.

## License and attribution

[MIT](LICENSE). Protocol port and captured fixture adapted from [sanild/ugreen-studio-controller-macos](https://github.com/sanild/ugreen-studio-controller-macos). Framing/CRC research was cross-checked against [tzy3454u/ugreen-headphone-control](https://github.com/tzy3454u/ugreen-headphone-control). Original MIT notices are retained in [third-party](third-party/); keep them when redistributing. Terminal dependencies have their own licenses; see the [Rust dependency inventory](third-party/RUST-DEPENDENCIES.md). Product names and trademarks belong to their respective owners.
