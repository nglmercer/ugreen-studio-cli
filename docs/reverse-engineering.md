# UGREEN Studio Pro — reverse-engineering report

This report records what the official **UGREEN Audio** app
(`ugreen-audio-overseas-prod-v260933933-20260930197-ed209d775.apk`,
version 260933933, build 20260930197) reveals about the Studio Pro
control protocol, and how each finding maps onto this CLI.

**Status of every item is stated explicitly.** The project rule is
unchanged: a frame is only *verified* after a live capture from the
official app proves it. Everything below marked **static** was extracted
from the APK and cross-checked against the already-verified instruction
set, but has **not** been captured on hardware yet. Nothing marked
static may be sent to a headset until Phase-3 verification promotes it.

## Method

1. `apktool d` → smali + decoded resources; `jadx` → Java/Kotlin.
2. Located the transport and the protocol constants in smali and in the
   device-page JavaScript bundles under `assets/static/device/assets/`
   (the app is a V8/J2V8 hybrid: the UI pages build the frames, the
   Kotlin side relays raw bytes over RFCOMM).
3. **Validation first:** the analysis was trusted only after it
   reproduced every instruction this CLI already verifies on firmware
   0.2.5 — `0x04` info, `0x05` eq, `0x06` dual, `0x08` game, `0x09`
   anc, `0x0C` prompts, `0x12` spatial, `0x14`/`0x15` volume button
   actions, `0x17` wind. All nine matched the app's own constant
   tables and payload layouts exactly, which validates the extraction
   method for the unknowns.

## Transport (confirmed)

- Bluetooth **Classic RFCOMM**, opened with
  `createRfcommSocketToServiceRecord` using the standard SPP UUID
  `00001101-0000-1000-8000-00805F9B34FB`
  (`com/ugreen/unisdk/bluetooth/SPPUtil.smali`, `BTSocket.smali`).
  This matches the CLI's channel-1 default.
- The Kotlin bridge (`com/ugreen/rnmodule/ear/UGRNBluetoothBridge`,
  `UGRNUniversalModule`) relays base64 byte buffers between the JS
  pages and the socket; the JS sends via
  `uni.postMessage({type:"earphoneDetailMessage", payload:{dataBuffer}})`.
- The Studio Pro's device config sets `protocol_type: Qs.UGREEN_Universal`
  (= **1**). The other family, `Zhongke_Lanxun` (= 2), is a different
  frame dialect used by other chipsets (documented at the end for
  reference only).

## Frame shapes (UGreen_Universal / protocol_type 1)

These are exactly the shapes the CLI's `src/protocol/` already
implements and verifies:

- **Request** host→headset: `AA BB CC <instr> <len> <payload…> <crc_lo> <crc_hi>`
  — CRC-16/MODBUS (init `0xFFFF`, reflected poly `0xA001`),
  little-endian, over `instr‖len‖payload`. The app's `buildPacket`
  computes the identical CRC (`s^=byte; 8×{s&1 ? (s>>1)^40961 : s>>1}`).
- **Response** headset→host: `DD EE FF <instr> <success> <len> <payload…> <crc_lo> <crc_hi>`
  — CRC-16/CCITT-FALSE (init `0xFFFF`, poly `0x1021`).
- **Notification** headset→host: captured frames are
  `85 86 87 <envelope> <event> <value>` — six bytes, no checksum.
  The APK reads the event from offset 4 (byte 5), not offset 3. Its
  battery and finder parsers expect further bytes; longer framing
  remains unverified.

## Instruction map — Studio Pro (protocol_type 1)

Instruction numbers come from the app's constant objects
(`index-D9whH_hy.js` and its re-export chunks); payload layouts come
from the page send-sites and the `parsePacketApp` response parser.

| Instr | Hex | Request payload | Meaning | Response payload | Status |
|---|---|---|---|---|---|
| 1 | `0x01` | `[0]` | firmware version query | `v1[0..2]‖v2[3..5]` (two 3-byte versions) | **verified** |
| 2 | `0x02` | `[0]` | headphone ID query | `[id]` | static |
| 3 | `0x03` | `[0]` | headphone color query | `[color]` | static |
| 4 | `0x04` | `[0]` | device info query | 31-byte payload, see below | **verified** |
| 5 | `0x05` | `[mode]` | EQ preset | `[EQ_mode]` | **verified** |
| 6 | `0x06` | `[1]`/`[0]` | dual device on/off | `[double_link_mode]` | **verified** |
| 7 | `0x07` | `[1,1]`/`[1,2]`/`[0,1]`/`[0,2]` | find headphones start/stop left/right | none (state via notification event 9) | static |
| 8 | `0x08` | `[1]`/`[0]` | game mode | via notification event 4 | **verified** |
| 9 | `0x09` | `[depth]` | ANC mode | `[noise_mode]` | **verified** |
| 10 | `0x0A` | 8 bytes: tap/double/triple/long × left/right | button control | none | static |
| 11 | `0x0B` | `[1]`/`[0]`, or `[1,mac×6]`/`[0,0×6]` | high-quality decoding (LDAC) on/off | `acousticsResponse{state, value=[p0]}` | static |
| 12 | `0x0C` | `[0]` voice / `[2]` beeps | prompt sound type | `[sound_type]` | **verified** |
| 13 | `0x0D` | `[0]` | **connected device info (multipoint peer) query** | `[flag][peerIdx][name UTF-8][mac×6]` | static |
| 14 | `0x0E` | `[0]` | factory reset | logs 恢复出厂设置成功, then unpair+delete | static |
| 15 | `0x0F` | `[0]` | **codec query** | `[type1,code1,type2,code2]` | static |
| 16 | `0x10` | `[volume]` | volume set | `[sound_volume]` | static |
| 17 | `0x11` | — | ear-in-box (response only) | `[ear_Phone_in_box]` | static |
| 18 | `0x12` | `[1]`/`[0]` | spatial audio | `[space_acoustics]` | **verified** |
| 19 | `0x13` | `[1]`/`[0]` | reserved (no response case) | — | static |
| 20 | `0x14` | `[action]` | volume-up action | `[volume_add]` | **verified** |
| 21 | `0x15` | `[action]` | volume-down action | `[volume_minus]` | **verified** |
| 22 | `0x16` | `[1,peerIdx,mac×6]` / `[1,FF×7]` / `[0]` | **audio sharing** on/off | `[audio_sharing]` | static |
| 23 | `0x17` | `[1]`/`[0]` | wind noise reduction | `[windNoise]` | **verified** |
| 24 | `0x18` | `[tap,double,triple,long]` | AI tap actions | `[ai_tap,ai_double_tap,ai_triple_tap,ai_long_tap]` | static |
| 25 | `0x19` | `[0]`/`[1]` | channel mode mono/stereo | `[0=mono,1=stereo]` | static |
| 26 | `0x1A` | `[1]`/`[0]` | reserved (no response case) | — | static |
| 27 | `0x1B` | `[1]`/`[0]` | AI control state | `[ai_control_state]` | static |
| 29 | `0x1D` | `[1]`/`[0]` | reserved (no response case) | — | static |
| 30 | `0x1E` | `[1]`/`[0]` | high-quality decoding (alt path) | `highAudioDecodingResponse{state,value}` | static |
| 31 | `0x1F` | `[0]` | box MAC query | `[mac×6]` (reversed) | static |

### Device-info payload (instruction 4 response, 31 bytes)

Offsets already used by this CLI are marked **verified**; the rest are
new fields the app reads and the CLI could expose after verification.

| Offset | Field | Status |
|---|---|---|
| 0 | battery (`255` = unknown) | verified |
| 1 | right_battery | static |
| 2 | box_battery | static |
| 3 | noise_mode (ANC) | verified |
| 4 | EQ_mode | verified |
| 5 | double_link_mode | verified |
| 6 | game_mode | verified |
| 7 | **acoustics_code (codec)** — readable from device info | static |
| 8–15 | tap, double_tap, triple_tap, long_tap, right_tap, right_double_tap, right_triple_tap, right_long_tap | static |
| 16 | sound_type (prompts) | verified |
| 19 | box_status (pairing settings) or sound_volume (default 8) | static |
| 20 | space_acoustics | verified |
| 21 | wear_detection_type | static |
| 22 | volume_add | verified |
| 23 | volume_minus | verified |
| 24 | audio_sharing | static |
| 25 | windNoise | verified |
| 26–29 | ai_tap, ai_double_tap, ai_triple_tap, ai_long_tap | static |
| 30 | ai_control_state | static |

### ANC depth values (instruction 9)

The app's `sU` table confirms and extends the CLI's mapping. Odd bytes
are depth modes, even bytes are their "penetrate" (ambient) variants:

`0xA0` off · `0xA1` ultra · `0xA2` ambient · `0xB1` general ·
`0xB2` · `0xC1` gentle · `0xC2` · `0xD1` adaptive (AUTO) · `0xD2`.
The CLI's six values (`A0 A1 B1 C1 D1 A2`) all appear in the app's
`DEPTHList`/`PENETRATE` maps.

### Button-action values (instruction 10 and tap payloads)

`0` none · `1` play/pause · `2` volume up · `3` volume down ·
`4` next track · `5` previous track · `6` voice assistant ·
`7` game/music mode · `8` noise control · `9` answer/hang up ·
`11` long-press EQ. (The CLI's `volume-up/down-action` currently maps
`none/next/previous` to `0/4/5`, consistent with this table.)

### Codec values (instruction 15 response and device-info offset 7)

`0` SBC · `2` AAC · `255` LDAC (UGreen codec table; Zhongke Lanxun
devices use `1/2/3`). The instruction-15 response carries two
`[type, code]` pairs — left and right (or current and target) codecs.

## The three requested features, decoded

### 1. "Discover devices connected to the audio device" — instruction 13

This is the missing multipoint peer list. The app sends
`AA BB CC 0D 01 00 <MODBUS-CRC>` when a device page loads and again
200 ms after toggling dual mode. Each response frame carries **one**
peer:

```
payload[0]     flag — when peerIdx==1 and flag==1, peer 2 is absent
payload[1]     peer index: 0 = nothing connected (clears both),
               1 = peer 1, 2 = peer 2
payload[2..n-6] peer Bluetooth name (UTF-8)
payload[n-6..n] peer MAC (6 raw bytes — not obfuscated)
```

`mobilePhoneName1/mobileMac1` and `mobilePhoneName2/mobileMac2` in
the app are populated from these responses. The device emits one
response per connected peer, so a full list is one query plus the
frames that follow.

**No per-peer disconnect/reconnect/switch frame exists in this
protocol family.** The app manages peers with the dual-mode toggle
(instruction 6, already implemented as `dual`) and relies on
OS-level Bluetooth reconnection for switching. `peer_disconnect`,
`peer_reconnect` and `peer_switch` therefore stay `UnsupportedFeature`
— not a gap in the analysis, a gap in the headset's exposed protocol.

### 2. Find-headphones sound — instruction 7

`[0x01,0x01]` start left · `[0x01,0x02]` start right ·
`[0x00,0x01]` stop left · `[0x00,0x02]` stop right. State is
reported through notification kind 9
(`is_headphone_finder_left`, `is_headphone_finder_right`). The
"ring" sound itself is played by the phone app
(`libhte-native-lib.so`, an Oboe player, `assets/example_audio.mp3`)
— i.e. the classic "phone rings the earbuds" model, not a tone stored
in the headset.

### 3. Codec status/control — instructions 15 and 11

- **Read:** query instruction 15 (`[0]`) for the `[type,code]` pairs,
  or read offset 7 of the device-info response (instruction 4) — the
  CLI already fetches device info on `status`, so codec status is
  nearly free once offset 7 is trusted.
- **Control (LDAC):** instruction 11. Plain enable/disable is
  `[1]`/`[0]`. When the config flag `is_ldac_mac_address` is set, the
  app binds LDAC to the peer that should use it:
  `[0x01, mac0..mac5]` (7 bytes) to enable, `[0x00, 0×6]` to disable.
  The MAC is taken from the instruction-13 peer list. The app retries
  up to 3 times at 2 s intervals, waiting for `acousticsResponse` /
  `highAudioDecodingResponse`.

### Bonus findings (static)

- **Audio sharing** (instruction 22): enable to a peer
  `[1, peerIdx, mac×6]`, or `[1, FF×7]` when only one peer is
  connected; disable `[0]`. `peerIdx` is 2 when the phone is peer 1
  (share *to* the other peer), else 1. Enabling forces wind noise
  off.
- **Factory reset** (instruction 14): `[0]`; the app then unpairs and
  deletes the device.
- **Volume** (instruction 16): `[volume]`; response echoes
  `sound_volume` (default 8).
- **Mono/stereo** (instruction 25): `[0]`/`[1]`.
- **Box MAC** (instruction 31): `[0]`; response is the 6 MAC bytes
  reversed.

## Notification event codes (85 86 87)

| Event (byte 5) | Data (byte 6 onward) | Meaning |
|---|---|---|
| 1 | `[battery, right_battery, box_battery]` | battery change |
| 2 | `[noise_mode]` | ANC change |
| 4 | `[game_mode]` | game mode change |
| 5 | `[EQ_mode]` | EQ change |
| 8 | — | master↔slave switch |
| 9 | `[finder_left, finder_right]` | headphone-finder state |
| 10 | `[space_acoustics]` | spatial change |
| 11 | `[audio_sharing]` | audio-sharing change |
| 12 | `[double_link_mode]` | dual-connection change |
| 14 | `[p0]` | unknown |

All meanings in this table are static. Existing captures
`85 86 87 02 0A <echo>` and `85 86 87 02 05 00` align with event
codes 10 and 5, but live event semantics remain unknown in the CLI.
The APK accesses bytes 6 and 7 for additional battery values and byte 6
for finder-right. Six-byte captured framing cannot expose those values;
longer frames require hardware evidence before changing the decoder.

## Reference: the Zhongke_Lanxun dialect (protocol_type 2 — not the Studio Pro)

Other UGREEN devices (Zhongke Lanxun chipsets) use a CMD/TLV dialect
over the same framing: `[seq][CMD][1][0][len][TLV…]`. Commands include
`39` (query/response), `53` (ANC), `3` (wind), `32` (EQ), `34`
(button control). Instruction-39 TLV types: `battery=1`,
`firmwareVersion=2`, `deviceName=3`, `equalizer=4`, `controlInfo=5`,
`gameMode=8`, `promptSoundInfo=10`, `ANCType=12`,
`spatialSoundEffects=24`, `deviceDualConnection=25`,
`connectedDeviceInfo=26`, `highQualityDecoding=143`, `boxMac=144`,
`ANCDepth=145`, `findHeadphonesLeft=146`, `findHeadphonesRight=147`,
`inBox=148`, `windNoise=149`. Its `connectedDeviceInfo` TLV is
`[count]{[len][mac XOR 0xAD][codeType][name UTF-8]}` — note the MAC is
XOR-obfuscated there, unlike the raw MAC in the Studio Pro's
instruction 13. Documented so the two dialects are never confused.

## Native stack observed

- `com.jieli.jl_rcsp` — Jieli RCSP SDK (`PROTOCOL_TYPE_SPP = 1`),
  with `RcspConstant`, `AttrAndFunCode`, task framework.
- `com.jieli.jl_bt_ota` — OTA firmware update (`RcspAuth`,
  `cd03_crc_encode`, `parse_fw_info`).
- `com.jieli.bt_decryption` — BT-con encryption (`btcon_hash`,
  rijndael/AES tables) — the control channel can be AES-wrapped on
  some firmwares; the Studio Pro's verified frames are plaintext.
- `com.jieli.jl_pack_format` — firmware resource pack parser.
- `libDspConfig.so` — ECC secp192r1 device binding + AES.
- `libhte-native-lib.so` — Oboe audio playback (in-app sounds).

## What this analysis does **not** unlock (yet)

- **Per-peer disconnect/reconnect/switch** — no classic-protocol frame
  exists; only the dual-mode toggle (already implemented).
- **Custom EQ curves** — the app exposes presets only (instruction 5 /
  CMD 32 list); no curve-upload frame was found.
- **Firmware update** — handled by the separate `jl_bt_ota`
  multi-packet stack; not decoded here and out of scope for safe
  CLI automation.

## Interoperability and licensing

Findings are protocol *facts* (instruction IDs, payload layouts, CRC
behaviour) extracted for interoperability; no application code was
copied — this is a clean-room reimplementation, consistent with the
project's MIT-only provenance policy. Before shipping anything derived
from this analysis, check the UGREEN Audio app's EULA: reverse
engineering for interoperability is permitted in some jurisdictions
(e.g. EU, US DMCA 1201(f)) but the app's terms may restrict it.

## Phase-3 verification recipe (live hardware)

The fastest trust path, cheapest first:

1. **Direct probe** (no app needed): connect the CLI to the headset
   and send the candidate frames by hand, e.g. peer query
   `AA BB CC 0D 01 00 <MODBUS-CRC>`, codec query
   `AA BB CC 0F 01 00 <CRC>`, box MAC `AA BB CC 1F 01 00 <CRC>`;
   decode the `DD EE FF …` answers with `ugreen decode`. (Requires a
   raw-send facility — see the plan; `find` makes noise, so run it
   away from ears.)
2. **App ground truth:** pair the Studio Pro with an Android phone
   running UGREEN Audio v260933933. Enable *Bluetooth HCI snoop log*
   (developer options), open the app's device / headphone-finder /
   acoustics-code / audio-share pages, pull `/sdcard/btsnoop_hci.log`,
   and extract the RFCOMM frames. This is the reference capture.
3. **CLI capture:** with the app closed (it holds the RFCOMM channel),
   `ugreen --address <MAC> --model studio-pro capture > session.hex`
   while triggering the same pages' behaviour.
4. Match observed frames to the tables above; promote matches to
   verified in `docs/verification.md` and flip the capability bits in
   `src/models/studio_pro.rs` (`MULTIPOINT_PEERS`, codec, find, …)
   only for the operations a capture proves end-to-end.

Evidence pointers (apktool output, `.re/` — gitignored):
`assets/static/device/assets/index-D9whH_hy.js` (protocol constants,
`buildPacket`, `parsePacketApp`, `parsePacketHardware`, `$keyVoiceStatusChange`),
`assets/static/device/assets/{acoustics-code,audio-share,double-link,headphone-finder,control}-*.js`
(page send-sites), `smali_classes8/com/ugreen/unisdk/bluetooth/SPPUtil.smali`
(transport), `smali_classes8/com/ugreen/rnmodule/ear/` (bridge).

## Implementation progress

`src/protocol/static_analysis.rs` implements offline candidate builders,
response interpretations, peer-list accumulation and extended device-info
fields. `ugreen analyze` makes the static response and notification
interpretations available without a Bluetooth connection. See
[usage](usage.md#offline-apk-analysis). These additions do not promote
static evidence or change live capability bits. Client and CLI paths for
peer, codec, ID, color and box-MAC queries are implemented behind separate
capture gates. The peer collector processes fragmented/coalesced responses
and requires explicit empty/single/two-peer completion. Typed controls
have client/CLI execution paths behind independent capability gates,
with readback for buttons, AI actions, volume, AI control and sharing,
and value-checked acknowledgements for LDAC and channel mode.
Raw finder controls report an unconfirmed send. The gated `find` command
adds Linux PipeWire playback of an original stereo tone and attempts stop
after start/playback failures. Reset CLI cleanup closes the control session,
unpairs the exact address through the native host API and removes its
registry entry after acknowledgement. Windows finder playback now uses an
explicit waveform device index. Cancellation callbacks stop Linux/Windows
audio and attempt headset stop; scoped CLI handlers catch Linux SIGINT/
SIGTERM and Windows Ctrl+C/Ctrl+Break. The TUI feature panel and worker
now integrate queries, controls, cancellable finder playback and reset
cleanup. Official-app captures remain pending. Windows runtime playback/interrupt behavior is
unverified. No live capability is promoted by mock or source checks.
