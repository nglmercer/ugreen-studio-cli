# Usage

[Documentation index](README.md) · [Troubleshooting](troubleshooting.md)

All examples use `ugreen` as the executable name. Use `./target/release/ugreen` on Linux or `.\target\release\ugreen.exe` on Windows when running directly from a build. Replace `AA:BB:CC:DD:EE:FF` with your own headphones' address. Do not use a Max5c or unknown model as a test target.

## A cautious first session

1. Run `ugreen models`, `ugreen commands`, and a dry run. These do not access Bluetooth
2. Pair the Studio Pro in OS settings; the application does not pair for you
3. Close another app's headphone control session if it could own RFCOMM channel 1
4. Run `ugreen discover` to read the paired cache, or enter an address you have verified yourself
5. Read status before changing anything; unknown or unexpected values are a reason to investigate
6. If you decide to test a change, apply one setting and read the verified result before using profiles

```sh
ugreen --dry-run set anc ultra
ugreen discover
ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro status
ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro set eq bass
```

The model flag selects a protocol. It is not a hardware model check. The application does not provide a way to verify a retail variant automatically.

## Launch behavior

| Invocation | Behavior |
| --- | --- |
| `ugreen tui` | Open the TUI when the feature is compiled in and an interactive terminal is available |
| `ugreen` | Open TUI only when stdin and stdout are terminals; otherwise show help |
| `ugreen --help` | Always show CLI help |
| `ugreen status` | Run the CLI command; fail without explicit model/address |
| CLI-only build, no arguments | Show help |
| CLI-only build, `ugreen tui` | Report that the TUI feature is unavailable |

The TUI cannot be used with redirected stdin/stdout. Use explicit CLI commands for scripts and pipelines. Starting it auto-loads the paired-device cache (no scan, no connection, no writes); pass an address to skip that, or let the cache dialog pick it. The interface needs at least 56 columns by 20 rows; at smaller sizes it asks you to resize and disables device actions.

## Language

Both the CLI and the TUI speak English and Spanish. The CLI follows `--lang en|es` first, then the `UGREEN_LANG` environment variable, then the OS locale (`LANG`/`LC_*`/`LANGUAGE`); protocol values, profile syntax and `key=value` output labels always stay in English. The TUI starts in the CLI's language and `L` toggles live.

You can supply a target and protocol before launching:

```sh
ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro tui
```

This supplies the initial choices; connect remains a separate action. For offline packet previews, use CLI `--dry-run set` or `--dry-run profile apply`.

## Terminal controls

The `?` key opens the shortcuts screen with every binding, including mouse.
The footer bar mirrors the same keys; clicking a footer label presses it.

| Key / mouse | Action |
| --- | --- |
| `a` | Edit target Bluetooth address while disconnected |
| `p` | Load the OS's paired-device cache and choose an address (auto-loads at startup) |
| `m`, then `y` | Review and confirm the Studio Pro/HP206 protocol choice |
| `c` | Connect to the selected target and read status |
| `r` | Refresh status explicitly |
| `d` | Disconnect |
| Up / Down, wheel | Select a setting; navigate the paired-device list |
| Click | Select a setting or device; second click advances the proposal |
| Left / Right | Cycle a setting's proposed value |
| Enter | Apply the proposed setting change, or accept a dialog choice |
| Right-click / Esc | Cancel a dialog, discard the selected proposal, or request cancellation of busy work |
| `l` | Open/close the session log; Up/Down and Home/End navigate |
| `L` | Switch language English/Español |
| `?` | Open/close the shortcuts screen; Up/Down scroll and Home returns to the top |
| `q` or Ctrl+C | Quit; while a write or refresh is in flight, ask for `q`/Enter again to exit (a lone paired-cache load quits at once) |

Mouse clicks and the wheel need a terminal that reports mouse events; when
capture is unavailable the keyboard keeps full control. Quitting while busy
disables capture cleanup through Ratatui's normal restore path.

Address entry accepts hexadecimal digits and colons, Backspace deletes, Ctrl+U clears, Enter validates, and Esc discards the edit; follow the dialog hints. In the protocol dialog, `n` also cancels. The session log retains the latest 100 status events in memory and is not saved to disk. Esc also closes the log or help.

A listed device is just a cached paired device and may be offline or a different model. Selecting it alone does not open a Bluetooth connection.

The interface distinguishes last-read state from a proposed value. Editing a proposal is local. Fields missing or unknown in device readback cannot be written from the TUI, and proposing the current value does not send a redundant write. A write requires a review and confirmation, followed by device-info preflight, acknowledgement (except spatial audio: retail firmware applies the write but answers with an `85 86 87` notification instead of a `DD EE FF` ack, so only the readback verifies it), and matching readback. If a write fails or its result is uncertain, further writes are blocked until an explicit successful refresh. Do not infer failure to apply from a timeout alone.

Pressing `d` while busy requests cancellation and disconnection after the current stage. Esc requests cancellation but keeps the interface open; quitting while busy asks for a second `q` or Enter and does not wait for the operation. Cancellation cannot undo bytes already sent. A setting may have changed if you disconnect or exit during a write. Query status before another attempt. Device operations use bounded timeouts, but a whole workflow can require several operations. Follow the on-screen busy/cancellation state rather than repeatedly submitting a write.

Battery is read from the vendor device-info response. No platform battery API is used as a fallback. Codec information is unavailable, and there is no codec-selection command. Unknown fields are not replaced with guessed defaults.

## CLI commands

| Command | Bluetooth behavior |
| --- | --- |
| `help`, `--help`, `-h` | Offline help |
| `--version`, `-V` | Offline package version |
| `models` | Offline compatibility warning |
| `commands` | Offline supported keys/values |
| `decode HEX` | Offline response-frame validation |
| `profile example` | Offline example profile |
| `discover` | Read OS paired cache; no scan or connection |
| `status` | Connect; query device info, then firmware |
| `set KEY VALUE` | Connect; preflight, write, acknowledge, read back |
| `profile export` | Connect; query info; emit known settings |
| `profile apply FILE` | Validate full file, then sequential verified writes |

Place global options **before** the command:

- `--address`: exactly six colon-separated hexadecimal octets; either letter case is accepted
- `--model studio-pro`: required for hardware commands; no other protocol is implemented
- `--channel 1`: RFCOMM channel 1–30, default 1; no automatic service/channel discovery
- `--timeout 3`: 1–60 seconds, default 3, for individual operations
- `--lang en`: interface language `en` or `es`; without it, `UGREEN_LANG`, then the OS locale
- `--dry-run`: preview packets for `set` or `profile apply`; no connection or writes

`--dry-run` is not a simulated status query. It is rejected with `tui`, `discover`, `status`, and `profile export`. Use `decode` for captured response bytes and `profile example` for an offline template.

## Interpreting status

`status` reports the user-selected protocol, battery, supported setting values, firmware, and raw device-info bytes when the full command succeeds. Missing fields or unknown enum values display as `unknown`. Battery bytes outside 1–100, including 0 and 255, are treated as unknown by this implementation.

Firmware is a separate query. If it fails, settings may already have been printed and the command still exits nonzero. Check both output and exit status. Values are a snapshot from the last query; there is no continuous status subscription.

ANC `off` and `ambient` use mode bits that may preserve a previous depth in the upper nibble. Readback verification intentionally compares only the mode for those two choices. Other ANC modes must match their full byte. See [Protocol](protocol.md).

## Profiles

Generate an example and inspect its packets:

```sh
ugreen profile example > my-profile.conf
ugreen --dry-run profile apply my-profile.conf
```

Example contents:

```ini
# UGREEN Studio Pro profile v1
model=studio-pro
anc=ultra
eq=classic
game=off
spatial=off
```

Rules:

- UTF-8 text, at most 16,384 bytes, containing at least one setting
- Mandatory `model=studio-pro`, exactly once
- One `key=value` per nonblank line; surrounding whitespace is trimmed
- Full-line `#` comments only; inline comments are not supported
- Exact case-sensitive keys/values; duplicate keys, unsupported keys, and invalid values are errors
- All parsing completes before connecting, including in dry-run mode

Export includes only known supported values, not battery, firmware, codec, unknown fields, or the Bluetooth address:

```sh
ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro profile export > saved-profile.conf
ugreen --dry-run profile apply saved-profile.conf
```

Check the export command's exit status before treating the file as a backup. Shell redirection can create an empty file when the command fails. If a device reports no known settings, an export can contain only the model header; such a file cannot be applied because there are no settings.

Apply only after reviewing the target and preview:

```sh
ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro profile apply saved-profile.conf
```

Profiles are **not transactions**. Settings apply in file order. A failure stops the sequence and reports how many settings were verified. Earlier changes are not rolled back, and the failing setting itself may have changed. Read status to establish the actual final state.

### Windows text encoding

The parser requires UTF-8 without a byte-order mark. Older Windows PowerShell output redirection may produce UTF-16 or a UTF-8 BOM; that file will fail validation. Use a text editor set to UTF-8 without BOM, or an explicit encoding when creating a profile:

```powershell
$lines = & .\target\release\ugreen.exe profile example
$utf8 = [System.Text.UTF8Encoding]::new($false)
[System.IO.File]::WriteAllLines((Join-Path $PWD 'my-profile.conf'), [string[]]$lines, $utf8)
.\target\release\ugreen.exe --dry-run profile apply my-profile.conf
```

## Decode and scripting

Pass a capture as one quoted argument. Whitespace and colons between hexadecimal bytes are accepted; `0x` prefixes are not. `decode` accepts only a clean, complete response stream for success. It can print valid frames yet exit nonzero if there is also noise, an incomplete frame, or a rejected checksum. It never sends captured bytes.

Successful CLI commands exit 0; errors exit nonzero and are written to stderr. Shells can separate output from errors, but diagnostic output is not a versioned machine-readable API. For an uncertain setting write, do not build an automatic retry loop around a nonzero exit code.
