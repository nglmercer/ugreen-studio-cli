# UGREEN Audio APK re-audit — October 15, 2026

The supplied APK contains useful protocol evidence beyond the earlier
[report](reverse-engineering.md). This audit corrects three conclusions:
there is a targeted dual-close packet, MAC-bound LDAC uses `0x1E`, and the
finder tone source was not established. All findings below are **static app
behavior**, without any new headset capture or live capability promotion.

## Input and reproducibility

| Property | Observed value |
|---|---|
| Local input | `docs/ugreen-audio-overseas-prod-v260933933-20260930197-ed209d775.apk` |
| Size | 329,173,903 bytes |
| SHA-256 | `b2ed391d9c0a94769cb00a1f8a35a2bb8461b8d084721c66988ca011d1a2f45b` |
| Manifest package | `com.ugreen.iot` |
| Manifest version | `2.6.0.933`, version code `933` |
| Android SDK | minimum 24, target 37 |

The filename's `260933933` is not the Android manifest version code.
The hash identifies the supplied file; this audit did not authenticate its signer.

Reproduce the [evidence record](evidence/apk-static-2026-10-15.json) with
Python 3.11 or later; no Android tools, network or Bluetooth access are needed:

```sh
python3 scripts/analyze_apk.py docs/ugreen-audio-overseas-prod-v260933933-20260930197-ed209d775.apk --output target/apk-static.json
```

The [analyzer](../scripts/analyze_apk.py) reads the binary Android manifest and
checks build-specific byte anchors directly inside ZIP members. It records
uncompressed asset hashes, byte offsets, source occurrence counts, DEX hashes and
native-library inventory. It refuses a different APK hash or missing anchors.
It never executes app code. The evidence contains short source locators rather
than entire proprietary bundles; extracted working files stay under ignored
`target/` and the original APK remains ignored.

## Trace the active device-page code

`assets/static/device/index.html` selects `assets/index-D9whH_hy.js` as its
modern entry. That entry names the routes for `double-link-B1wNDsyj.js` and
`acoustics-code-GbTtk_At.js`. Those pages import their constants back from the
same entry. Similar chunks from other builds and legacy paths also ship in the
APK; identifying the HTML entry avoids mixing their minified aliases.

The store defaults to `UGREEN_Universal = 1`; `Zhongke_Lanxun = 2` is separate.
`setConfig` merges incoming configuration, so local defaults are not a guarantee
of the final configuration received by a particular installation. The local
`UGREEN Studio Pro` model case enables dual-device pairing, high-quality decoding
and audio sharing, but sets `is_headphone_finder_enabled` to false. It does not
override `is_ldac_mac_address` or the default protocol. The model case uses a
`fake-jwt-token` placeholder; it is a local configuration path, not a recorded
backend response or firmware capability report. No `HP206` identifier was found
in the extracted device JavaScript; retail identity still needs independent
confirmation.

The existing native decompilation provides navigation context:
`com.ugreen.webview.refactor.WebCommandDispatcher` forwards
`earphoneDetailMessage` to `BluetoothController.writeBuffer`, which decodes
base64 and selects BLE writes or `SPPUtil`. `BleUUIDs` assigns Studio Pro
`ISBLE=false` and the standard SPP service UUID, while the similarly named
HiTune Max6 entry has `ISBLE=true`. They must not inherit each other's transport
policy merely because some UI flags match. The reproducible byte-anchor record
covers JavaScript and the manifest; native method semantics were inspected in
the existing `.re/` decompilation, not independently re-decompiled in this audit.

## Targeted dual close: instruction `0x06`

The main entry defines `eU` as instruction 6 with `OPEN=[1]`, `CLOSE=[0]`,
then exports it as `aE`. The dual page imports `aE` as `v`. Its confirmation
handler constructs:

```text
payload = [0, otherPeerIndex, otherPeerMac[0..5]]
request = AA BB CC 06 08 <payload> <MODBUS CRC, low byte first>
```

The UI chooses peer 2 when `phoneBleName == mobilePhoneName1`, otherwise peer 1.
It selects the corresponding MAC, asks the user to confirm closing the other
device, sends the extended dual-close request, updates local peer fields, waits
200 ms and queries peers again with instruction `0x0D` and payload `[0]`.
The peer parser reads UTF-8 name bytes followed by six raw MAC bytes; index zero
clears both entries, and peer-1 flag one clears peer 2.

This disproves the previous blanket claim that no targeted peer-close format
exists. It does **not** establish disconnect while keeping dual mode enabled,
reconnection, or selection of the active audio peer. Matching the current phone
by name can also be ambiguous; a future implementation cannot treat this as
reliable identity evidence. The verified one-byte dual toggle does not authorize
the eight-byte extension. Existing live peer methods continue to refuse.

For protocol 2 the same handler uses `WRITE_DEVICE_CLOSE`, after a dual-mode
write. Its builder emits command `0x33`, TLV type `3`, length `6`, then the MAC.
That is a different frame dialect, not an alternative HP206 instruction.
No independent reconnect or active-peer-switch send path was established.
Notification event 8 emits `switchMasterSlave`; a notification name does not
prove an outbound peer-switch command.

## LDAC opcode correction

| Path | Constant/export/import trace | Instruction and payload |
|---|---|---|
| Plain preference | `aU` → export `aV` → codec page `E` | `0x0B`, `[1]` or `[0]` |
| MAC-bound preference | `gU` → export `aU` → codec page `y` | `0x1E`, `[1, mac×6]` or `[0, zero×6]` |
| Codec query | `yU` → export `aS` | `0x0F`, `[0]` |

The reuse of `aU` as both a local identifier and an exported alias caused the
earlier MAC-bound mapping error. The `is_ldac_mac_address` branch selects the
addressed path; the Studio Pro local model case does not set that flag. A
MAC-bound candidate is therefore shared-app evidence, not a claim that this
Studio Pro configuration uses it.

The page sends at most three attempts, two seconds apart, stopping on either
`acousticsResponse` or `highAudioDecodingResponse`. This describes the vendor
app, not permission to retry CLI writes. The CLI still never retries implicitly.
The legacy offline `high-quality` one-byte `0x1E` candidate is only a hypothesis
from scalar constants: this handler provides an actual send site for the
seven-byte addressed form, not the one-byte form.

The repository's offline MAC-bound LDAC builder and CLI packet expectations now
use `0x1E`. Capability gates remain unchanged. The query response parser assigns
codec values by peer index; it does not establish the negotiated host codec or
audible playback quality.

## Finder and sharing limits

The shared finder page sends instruction `0x07` for left/right start/stop and
updates its local animation state. Its handler does not establish a host tone
playback chain. In the existing native decompilation, `example_audio.mp3` is
assigned by `io.mimi.sdk.profile.personalized.MimiPlayerView`; its presence,
or the presence of an Oboe library, does not connect it to headphone finding.
The previous report's assertion that the phone plays this MP3 as the finder
tone is withdrawn. The local Studio Pro finder flag is false. Whether HP206
rings internally, needs streamed audio, or supports the command at all remains
unverified. The CLI's generated finder tone is its own gated implementation,
not a reproduction proven by this APK.

The sharing page constructs instruction `0x16` with `[1,index,mac×6]`, or
`[1,FF×7]` when only one peer is listed, and `[0]` to disable. After enabling,
it calls `setDeviceInfo({windNoise:false})`; it does not send an additional
wind-noise command in that handler. This is evidence of a local UI update,
not proof that the headset disables wind reduction. The CLI's wind-off guard
remains a conservative independent policy pending physical readback.

## What remains to prove

The app is evidence of its own implementation, not an infallible protocol
specification. For example, its constants label `cU` as instruction 16 (volume)
and `lU` as 17 (in-box), but the response switch assigns `sound_volume` under
`lU` and `ear_Phone_in_box` under `cU`. This mismatch needs captures, not an
automatic swap of the CLI parser. Similarly, the app's peer parser lacks the
CLI's strict bounds and rejection handling. Preserve those checks rather than
copying permissive app behavior.

Capture official-app traffic on an identified HP206 and exact firmware while
opening peers/codecs, disabling dual mode with a second peer, and changing each
supported reversible control. Correlate requests, acknowledgments, independent
readback and actual link/audio outcomes. Record final runtime configuration as
well as local app defaults. Reconnect, independent disconnect and audio-peer
switching require their own evidence. Finder requires a demonstrated supported
model and an established tone source before any live exposure. Native Windows
and physical BlueZ validation remain separate acceptance requirements in the
[implementation checklist](implementation-progress.md).
