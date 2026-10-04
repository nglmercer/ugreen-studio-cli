# Troubleshooting

[Documentation index](README.md) · [Usage](usage.md) · [Build guide](building.md)

Start with `ugreen --help`, `ugreen models`, and `ugreen --dry-run set game on`. These are offline checks. Do not use repeated setting writes as a connection diagnostic.

## The TUI does not open

- Run `ugreen tui` in an interactive terminal with both stdin and stdout attached
- A no-default-features binary has no TUI; rebuild with the default feature set
- No arguments with redirected output intentionally prints help
- If using an older supplied executable, rebuild the current source; an older binary does not acquire new features from updated documentation
- Enlarge a small terminal and follow its displayed help; use explicit CLI commands if the terminal cannot support the interface

If normal exit leaves the terminal damaged, close and reopen that terminal tab. On a Unix terminal, `stty sane` or `reset` may restore its display/input. Abrupt process termination cannot guarantee cleanup. Report whether the issue followed a normal quit, an error, a panic or a forced kill.

## Paired-device list is empty or fails

`discover` is a cache listing, not a scan. Pair the headphones using your OS first. Cached names may be old, unrelated, or unavailable devices. Verify the address separately before selecting a target.

On Linux:

- Check that the Bluetooth service and adapter are available through your normal OS tools
- The primary listing path reads the system D-Bus; a missing or restricted bus is reported, not guessed around
- The fallback uses BlueZ's `bluetoothctl devices Paired`; unexpected output from an incompatible version is reported instead of guessing
- A missing-controller or collection-timeout error is not a reason to reset the headphones
- If you already know the correct paired address, direct connection requires neither listing path

On Windows, listing uses Microsoft's cached authenticated-device APIs with radio inquiry disabled. Check pairing and adapter state in Windows settings. The application does not enable an adapter, request pairing, or change permissions for you.

## Connection refused, timeout or unsupported socket

Check, in order:

1. The headphones are on and paired to this OS
2. The address has six colon-separated hexadecimal octets
3. You explicitly selected `--model studio-pro`; its warning is intentional
4. The adapter/driver supports Bluetooth Classic RFCOMM
5. Another app, including UGREEN Connect on a phone, is not holding the control channel
6. The selected RFCOMM channel is correct; default 1 comes from the reference, not service discovery

If the device is merely slow, a read-only status attempt with `--timeout 5` can help diagnose it. Do not sweep channels or change device settings blindly. Containers, virtual machines and WSL can lack access to the appropriate native Bluetooth stack even when compilation succeeds. The current backend is not a macOS implementation.

Do not automatically elevate privileges or change system security settings to work around an error. Use the OS's supported Bluetooth setup and investigate the precise error.

## Status shows unknown values

Unknown is intentional. Optional bytes may be absent or have an unrecognized value. Battery 0, 255 and values outside 1–100 are reported as unknown. Do not interpret this as a measured 0% charge. Firmware can fail independently after settings are printed, causing a nonzero exit status.

Codec reporting and codec selection are not implemented for this Studio Pro mapping. Features found in a Max5c project must not be copied over by assuming matching command IDs. Provide a read-only capture — `ugreen --address ... --model studio-pro capture > session.hex` records one — together with exact model/firmware details for protocol investigation instead.

## A setting timed out or readback did not match

Stop writing and query status. A timeout or missing acknowledgement does **not** establish that the headphones ignored the write. Readback failure means the result was not verified; it is not a rollback.

In the TUI, perform an explicit refresh after an uncertain write before another setting change. Reconnection does not substitute for understanding the current state. With profiles, earlier verified settings stay applied, and the failing setting may have changed too. Compare current state with the intended profile before deciding whether to apply anything else.

If a response is rejected for CRC or contains unexpected fields, retain the capture for offline analysis. Do not disable CRC validation or switch to Max5c mappings to force success.

## A profile cannot be read

Check that it is UTF-8 without BOM, no larger than 16 KiB, contains `model=studio-pro`, and has at least one supported setting. Remove duplicate keys. Use full-line comments only. A `.conf` extension does not determine encoding.

Older Windows PowerShell redirection can produce UTF-16 or BOM-prefixed text. See the [Windows profile example](usage.md#windows-text-encoding). Re-run `--dry-run profile apply FILE` after correcting the file; no device is needed.

## Capture decoding exits nonzero

Put the whole capture in one quoted argument, or pass a capture file's contents directly; hex digits may be separated by spaces or colons, `#` starts a comment, and `0x` must be omitted. The offline command succeeds only for a clean, complete stream: complete `DD EE FF` response frames with CCITT-FALSE CRC, optionally with six-byte `85 86 87` notifications (these print with `event=unknown` and do not fail the command). Noise, truncated bytes, a wrong length, a bad checksum, or well-framed `AA BB CC` bytes with no verified RX meaning all make the command exit nonzero, even when another frame was valid.

Record a suspect session with `capture` instead of reconstructing it from logs: its stdout is already in decode/fixture format, and the bytes printed before a mid-session failure are kept. See [Protocol](protocol.md) for the known fixture and CRC difference. Offline decoding sends nothing.

## Build failures or an executable will not run

- Check Rust version and platform prerequisites in [Building](building.md)
- Use the supplied lockfile; a missing cache/network error is separate from a Rust source error
- A Windows cross-check does not supply MinGW libraries or create a linked `.exe`
- A Linux executable requiring a newer glibc should be rebuilt on the target distribution
- Build from source when the available binary predates these features

## Useful issue information

Use `l` in the TUI to inspect the latest 100 session events; the log exists only in memory. Collect the application version, OS/architecture, Rust version if built locally, feature set, exact command or TUI action, error text, headphone retail/model label, and firmware if a read succeeds. A `capture` file from the failing session is the single most useful attachment. Include whether a write may already have happened. A minimal profile and offline capture can help reproduce parsing issues.

Bluetooth addresses and paired-device names can identify devices or people; redact them from public reports. Never include account credentials or unrelated captures. No report should describe a mock/cross-target test as a physical headphone test.
