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

The TUI cannot be used with redirected stdin/stdout. Use explicit CLI commands for scripts and pipelines. Starting it restores the last target and reconnects automatically (default on, `--no-autoconnect` opts out); with no saved target it auto-loads the paired-device list — no scan, no writes — for an explicit pick. The interface needs at least 56 columns by 20 rows; at smaller sizes it asks you to resize and disables device actions.

## Language

Both the CLI and the TUI speak English and Spanish. The CLI follows `--lang en|es` first, then the `UGREEN_LANG` environment variable, then the OS locale (`LANG`/`LC_*`/`LANGUAGE`); protocol values, profile syntax and `key=value` output labels always stay in English. The TUI starts in the CLI's language and `s` opens app settings to change it.

You can supply a target and protocol before launching:

```sh
ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro tui
```

This supplies the initial choices; connect remains a separate action. For offline packet previews, use CLI `--dry-run set` or `--dry-run profile apply`.

## Terminal controls

The `?` key opens the shortcuts screen with every binding, including
mouse, as a two-column table (key, action) grouped by section.
The footer bar mirrors the same keys; clicking a footer label presses it.

| Key / mouse | Action |
| --- | --- |
| `a` | Edit target Bluetooth address while disconnected |
| `p` | Reload the OS paired-device list (a fresh list on every press) and choose an address (auto-loads at startup) |
| `m`, then `y` | Review and confirm the Studio Pro/HP206 protocol choice |
| `c` | Connect to the selected target and read status |
| `r` | Refresh status explicitly |
| `d` | Disconnect the control session; the Bluetooth link itself is not modified |
| `s` | Open app settings: language, RFCOMM channel, timeout, target |
| Up / Down, wheel | Select a setting (crossing category boundaries), navigate the paired-device list |
| Tab / Shift+Tab | Switch the setting category (Audio, Connection, Environment, Feedback) |
| `1`–`9` | Jump to the Nth setting of the active category |
| Click | Select a setting or device; second click advances the proposal |
| Left / Right | Cycle a setting's proposed value; in app settings, change the selected option |
| Enter | Apply the proposed change, open the value list when no proposal exists, or accept a dialog choice |
| Right-click / Esc | Cancel a dialog, discard the selected proposal, or request cancellation of busy work |
| `l` | Open/close the session log; Up/Down and Home/End navigate |
| `L` | Open app settings with the language row selected |
| `?` | Open/close the shortcuts screen; Up/Down scroll and Home returns to the top |
| `q` or Ctrl+C | Quit; while a write or refresh is in flight, ask for `q`/Enter again to exit (a lone paired-cache load quits at once) |

The TUI remembers your last target, channel, timeout and language in
`~/.config/ugreen-cli/state` (override with `UGREEN_STATE_FILE`). On the next
start it reconnects to that device automatically — autoconnect is on by default;
`--no-autoconnect` disables it for future runs and `--autoconnect` turns it back
on. With no cached target, or when autoconnect is off, the address is pre-filled
and the protocol choice is already confirmed, so connecting is a single `c`.
The cache only stores your own explicit choices — it never detects the device
model, and every connection still runs its preflight query.

App settings (`s`) change the interface language from a direct list
(`Enter` on the language row, or Left/Right to cycle), the RFCOMM channel
(1–30) and the operation timeout (1–60 s). Channel and timeout apply to the
next connection, not to an open session; the language applies at once.
Changes are saved to the same cache file the TUI reads at startup.

Mouse clicks and the wheel need a terminal that reports mouse events; when
capture is unavailable the keyboard keeps full control. Quitting while busy
disables capture cleanup through Ratatui's normal restore path.

Address entry accepts hexadecimal digits and colons, Backspace deletes, Ctrl+U clears, Enter validates, and Esc discards the edit; follow the dialog hints. In the protocol dialog, `n` also cancels. The session log retains the latest 100 status events in memory and is not saved to disk. Esc also closes the log or help.

Settings are grouped into four categories — Audio (ANC, equalizer, game mode, spatial audio), Connection (dual connection), Environment (wind reduction) and Feedback (prompts, volume-button actions). `Tab` switches category, digits `1`–`9` jump within it, and Up/Down cross into the adjacent category at a boundary. With no proposal pending, `Enter` opens a value list for the selected setting: Up/Down move, `Enter` applies, `Esc` closes, and a click applies the clicked value at once. The value the device currently reports is marked `*`.

A listed device is just a paired device of this OS and may be offline or a different model. Rows show the link state explicitly (`[CONNECTED]`, `[DISCONNECTED]` or `[UNKNOWN]`) and the registry-confirmed model (`model=studio-pro`) when you confirmed it for that address; enumeration never guesses a model. The list orders the current target first, then connected, disconnected and unknown devices, breaking ties by name and address. Selecting a device alone does not open a Bluetooth connection — it only updates the target and the header's Bluetooth state — and the protocol confirmation is re-derived for the selected address, so confirming one device does not confirm another.

The header reports the two links separately: `Bluetooth:` is the OS link state of the current target — from paired-device enumerations, and held at `CONNECTED` whenever this application's RFCOMM `Control:` session is open, since an open session proves the link — while `Control:` is the session itself. The control line may read `DISCONNECTED` while Bluetooth reads `CONNECTED` (after `d`, or for a link this application never opened); neither is wrong, and `d` only ever changes the control side. A failed connect never claims `CONNECTED`, and a stale pairing list cannot downgrade a live session.

The interface distinguishes last-read state from a proposed value. Editing a proposal is local. Fields missing or unknown in device readback cannot be written from the TUI, and proposing the current value does not send a redundant write. Applying with Enter requires device-info preflight, acknowledgement (except spatial audio: retail firmware applies the write but answers with an `85 86 87` notification instead of a `DD EE FF` ack, so only the readback verifies it), and matching readback. If a write fails or its result is uncertain, further writes are blocked until an explicit successful refresh. Do not infer failure to apply from a timeout alone.

Pressing `d` while busy requests cancellation and disconnection after the current stage. Esc requests cancellation but keeps the interface open; quitting while busy asks for a second `q` or Enter and does not wait for the operation. Cancellation cannot undo bytes already sent. A setting may have changed if you disconnect or exit during a write. Query status before another attempt. Device operations use bounded timeouts, but a whole workflow can require several operations. Follow the on-screen busy/cancellation state rather than repeatedly submitting a write.

Battery is read from the vendor device-info response. No platform battery API is used as a fallback. Codec information is unavailable, and there is no codec-selection command. Unknown fields are not replaced with guessed defaults.

## Internationalization

Interface text lives in `src/i18n/`, one file per language: `en.rs`
is the source-of-truth catalog, and every other language module
(`es`, `pt`, `de`, `fr`, `it`, `nl`, `ru`, `zh`, `ja`, `ko`) starts
from the English catalog and overrides only the strings it
translates. A string left in English is a deliberate fallback, not a
missing entry. `{}` placeholders keep the same argument order in
every translation, and a test asserts the placeholder count matches
English for all argument-carrying fields, so a translation cannot
break formatting. `L` in the TUI opens the app settings, where a
language list picks the language directly and the choice is
remembered in the target cache. Protocol values, profile
syntax, `key=value` output labels and wire bytes are never
translated. To add a language, copy `en.rs` to a new module,
translate the strings you can, and register the variant in `Lang`.

## CLI commands

| Command | Bluetooth behavior |
| --- | --- |
| `help`, `--help`, `-h` | Offline help |
| `--version`, `-V` | Offline package version |
| `models` | Offline compatibility warning |
| `commands` | Offline supported keys/values |
| `decode HEX` | Offline response-frame validation |
| `profile example` | Offline example profile |
| `discover` | Read OS paired devices; annotate the registry-confirmed model; no scan or connection |
| `capture` | Connect; print the session's raw TX/RX bytes as fixture hex on stdout |
| `status` | Connect; query device info, then firmware |
| `set KEY VALUE` | Connect; preflight, write, acknowledge, read back |
| `profile export` | Connect; query info; emit known settings |
| `profile apply FILE` | Validate full file, then sequential verified writes |

Place global options **before** the command:

- `--address`: exactly six colon-separated hexadecimal octets; either letter case is accepted
- `--model studio-pro`: required for hardware commands; no other protocol is implemented
- `--channel 1`: RFCOMM channel 1–30, default 1; no automatic service/channel discovery
- `--timeout 3`: 1–60 seconds, default 3, for individual operations
- `--lang en`: interface language `en`, `es`, `pt`, `de`, `fr`, `it`, `nl`, `ru`, `zh`, `ja` or `ko`; without it, `UGREEN_LANG`, then the OS locale
- `--dry-run`: preview packets for `set` or `profile apply`; no connection or writes

`--dry-run` is not a simulated status query. It is rejected with `tui`, `discover`, `status`, `capture`, and `profile export`. Use `decode` for captured response bytes and `profile example` for an offline template.

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

## Decode, capture and scripting

Pass a capture as one quoted argument. Whitespace and colons between hexadecimal bytes are accepted; `0x` prefixes are not, but `#` starts a comment that runs to the end of its line, so a recorded capture file can be passed unchanged. `decode` accepts only a clean, complete response stream for success: it prints valid frames yet exits nonzero when the input also contains noise, an incomplete frame, a rejected checksum, or well-framed `AA BB CC` bytes with no verified RX meaning. Six-byte notifications print with `event=unknown` and count as decoded frames, not noise. It never sends captured bytes.

To record a live session instead of copying hex from a terminal, use `capture`. It runs the same read-only queries as `status` and prints every byte the session exchanged as fixture-style hex on stdout — TX folded into `#` comments, RX as hex lines — while the human-readable context goes to stderr, so redirection keeps the record pure:

```sh
ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro capture > session.hex
ugreen decode "$(cat session.hex)"
```

If a query fails mid-session, the bytes recorded so far are still printed before the error exits nonzero, because a failed session is exactly when the record matters. The output shape matches `tests/fixtures/protocol/`, so a reported session can become a reviewed fixture after review.

Successful CLI commands exit 0; errors exit nonzero and are written to stderr. Shells can separate output from errors, but diagnostic output is not a versioned machine-readable API. For an uncertain setting write, do not build an automatic retry loop around a nonzero exit code.

## Offline APK analysis

`ugreen analyze 'DD EE FF ...'` interprets complete CRC-valid response
frames using the static findings in [the reverse-engineering report](reverse-engineering.md).
It also accepts the six-byte notification frames. Output is explicitly
unverified; it does not change live status or enable headset commands.
Names are printed with Rust debug escaping to prevent terminal control sequences.

The library's `protocol::static_analysis` module provides candidate packet
builders for peer and codec queries, finder start/stop, LDAC (plain and
MAC-bound), button and AI actions, volume, channel mode, audio sharing,
AI control, factory reset, ID/color and box-MAC queries. These builders
only return bytes. They have no transport methods. The volume byte has
no documented range beyond its byte representation; offset 19 remains
ambiguous between box status and volume.

Peer responses preserve UTF-8 names and raw MAC order. `PeerList` accepts
multiple response updates, clears both slots on index zero, and clears
slot two when peer one's absence flag is set. Box-MAC query responses
reverse the six MAC bytes. Unknown codec values remain explicit.

Static notification interpretations remain separate from live events.
The APK reads notification event codes from byte 5, after the byte-4
envelope. Six-byte captures expose one data byte: finder-right and
additional battery values remain unknown pending longer-frame captures.
Reserved instructions, the other chipset dialect, custom EQ and firmware
updates have no candidate command interface.

Live operations for these static findings remain gated until official-app
captures satisfy the report's verification requirement. Per-peer
connection control remains unsupported because no corresponding frame
was found.

### Candidate packets

`candidate` is always offline, including when address/model flags are
present. It prints a comment identifying static evidence followed by the
request frame with its MODBUS checksum. It does not play audio, reset a
device, or send bytes. Supported forms:

```sh
ugreen candidate firmware
ugreen candidate id
ugreen candidate color
ugreen candidate peers
ugreen candidate codec
ugreen candidate box-mac
ugreen candidate factory-reset
ugreen candidate find left on
ugreen candidate find right off
ugreen candidate ldac on
ugreen candidate ldac on 01:02:03:04:05:06
ugreen candidate ldac off 01:02:03:04:05:06
ugreen candidate high-quality off
ugreen candidate volume 8
ugreen candidate channel stereo
ugreen candidate ai-control on
ugreen candidate buttons 0,1,2,3,4,5,9,11
ugreen candidate ai-actions 6,7,8,0
ugreen candidate sharing off
ugreen candidate sharing single
ugreen candidate sharing 2 01:02:03:04:05:06
```

Finder sides are `left` and `right`; toggles accept `on`/`off`. Channel
accepts `mono`/`stereo`. Button action bytes follow the report's action
map; eight values specify left tap/double/triple/long followed by right
tap/double/triple/long. AI actions take four values in tap/double/triple/long
order. Reserved action 10 is rejected. `sharing single` builds the
single-peer sentinel; explicit sharing takes peer index 1 or 2 and a MAC.
MAC-bound LDAC disable emits six zero MAC bytes as the app does.

### Capture-gated queries

`peers`, `codec`, `id`, `color` and `box-mac` have client and CLI
implementations. Each checks its model capability before connecting or
sending anything. All five currently refuse with an unverified-feature
error because official-app hardware captures are missing. Use `candidate`
to inspect their request frames and `analyze` to inspect captured replies.
`--dry-run` directs users to `candidate` rather than implying a verified query.

After capture-backed promotion, `peers` will query all headset peer slots
and print each slot's name and address. Names are escaped for terminal
safety. A timeout before an explicit empty list, single-peer absence flag,
or two populated slots fails the query rather than claiming completion.
Codec output preserves both type/code pairs and unknown codec values.
The box-MAC query reverses the response bytes before displaying the MAC.

For a capture containing instruction-13 peer responses, `analyze` also
prints `static_peer_list complete=...`. One peer without an absence flag
is incomplete; it never implies that the other peer is absent. Explicit
clear/single-peer absence or both populated slots establish completion
within the supplied capture only. This remains a static interpretation.

### Capture-gated controls

`ugreen [OPTIONS] control OP [ARGS]` accepts the control forms listed
under `candidate`: finder, buttons, AI actions, LDAC, high-quality,
volume, channel, AI control, sharing and factory reset. Query forms are
rejected. Every live control currently refuses before connecting because
its official-app capture gate is unset. `--dry-run control ...` prints
an explicitly unverified candidate and never opens Bluetooth.

After capture-backed promotion, the executor checks echoed values before
accepting acknowledgements. Buttons use device-info readback without
waiting for an undocumented acknowledgement. AI actions, volume, AI control
and sharing also require device-info readback. Sharing enable additionally
requires wind reduction to read off. LDAC and channel mode return an
acknowledged result, without claiming independent device-state verification.
The executor never retries writes implicitly.

Raw finder controls return an unconfirmed result; they do not play audio.
The bounded `find` command adds host playback as described below. Factory
reset closes the control session after acknowledgement, then removes host
pairing and the selected registry entry. Cleanup failures distinguish reset,
unpairing and registry persistence so users do not resend an applied reset. No live controls
are enabled by these implementation additions.

### Finder playback and reset cleanup

`ugreen [OPTIONS] find left|right SECONDS AUDIO_TARGET` is a gated,
bounded finder operation. Duration must be 1–30 seconds. On Linux it
plays an original 900 Hz pulsed stereo WAV through `pw-play` to the given
sink name or serial; only the selected channel has samples. PipeWire
properties disable fallback, reconnection and target movement. The player
has a duration-plus-five-second deadline and the generated private
file is removed afterward. See [PipeWire routing properties](https://docs.pipewire.org/page_man_pipewire-props_7.html).

The command attempts finder stop even when start or playback fails,
and reports stop errors separately. Successful host playback cannot
prove audible output or headset finder state. Cancellation kills/reaps
the Linux player or resets Windows audio before attempting finder stop.
Scoped CLI handlers catch Linux SIGINT/SIGTERM or Windows Ctrl+C/Ctrl+Break
and restore prior handlers after cleanup. Stop cannot be guaranteed after
forced process termination or a lost Bluetooth connection. This command
is still disabled by its capture gate. Raw `control find ...` only sends the control frame.

After an acknowledged `control factory-reset`, the CLI closes RFCOMM
before unpairing. Linux uses [BlueZ RemoveDevice](https://bluez.readthedocs.io/en/latest/adapter-api/)
for the exact address; multiple matching adapters fail as ambiguous.
Windows uses [BluetoothRemoveDevice](https://learn.microsoft.com/en-us/windows/win32/api/bluetoothapis/nf-bluetoothapis-bluetoothremovedevice).
An absent host device is already unpaired. The registry forgets only
the reset target and clears the selection if it selected that target;
other devices and preferences remain. Cleanup failure does not resend
reset. The factory-reset capture gate remains closed.

On Windows, `ugreen audio-devices` lists cached waveform output indices
and names without playing audio or opening Bluetooth. Pass an explicit
index (including 0) to `find`; the default WAVE_MAPPER is rejected. Native
[waveOut playback](https://learn.microsoft.com/en-us/windows/win32/api/mmeapi/nf-mmeapi-waveoutopen)
uses the same generated stereo tone, with a completion callback,
duration-plus-five-second deadline and reset/unprepare/close cleanup.
Windows playback and console interruption need native runtime verification.
On Linux, identify the headset's sink using `wpctl status`; `audio-devices`
is Windows-only. Target selection is explicit and separate from the
headset's RFCOMM Bluetooth address.

### TUI feature commands

Press `F` in the main TUI to open the headset feature panel. Its commands
are `peers`, `codec`, `id`, `color`, `box-mac`, `control OP ARGS` and
`find left|right SECONDS TARGET`, using the same arguments as the CLI.
Command names stay canonical across interface languages. All static
features currently refuse with a capture-required message.

After capture-backed promotion, Enter runs a query or reviews a control;
a second Enter confirms the control. Editing the command cancels its
confirmation. Reset review states that pairing and the saved device will
be removed. Esc cancels review, closes the panel, or cancels active work;
Ctrl+U clears the input. Input is limited to 256 printable ASCII bytes.
Mouse clicks cannot apply underlying settings while this panel is open.

Only the worker owns Bluetooth and audio execution. The UI and worker each
check capabilities. Unconfirmed or acknowledgement-only controls block
further writes until explicit refresh. Finder uses the worker cancellation
flag, stops host playback and attempts headset stop. Reset closes the
session before host cleanup and clears the UI's saved device state.
