# Protocol research and provenance

[Documentation index](README.md) · [Architecture](architecture.md)

Research date: 2026-10-01 UTC. The local source, fixture and CRC arithmetic were independently inspected. No new physical-headphone capture was taken.

## Model selection and sources

The original Python candidate, [tzy3454u/ugreen-headphone-control](https://github.com/tzy3454u/ugreen-headphone-control), is MIT-licensed, uses the Python standard library and targets **HiTune Max5c**. Its README reports Windows testing, with other systems only potential targets. The initial research identified `ugreen_ht.py` blob SHA `9c6149886f7ca1d0290735c3932e3b99e2ceb4bb`.

This project targets **UGREEN Studio Pro**. The model-specific reference is [sanild/ugreen-studio-controller-macos at commit 7de78471158fab65654188b90fa775b862eb949c](https://github.com/sanild/ugreen-studio-controller-macos/tree/7de78471158fab65654188b90fa775b862eb949c), also MIT, which identifies its model as **HP206**. That code and its captured response support this mapping; they do not verify the exact retail variant or firmware of another pair.

| Setting | Studio Pro instruction | Max5c instruction |
| --- | --- | --- |
| EQ | `0x05` | `0x05` |
| Dual device | `0x06` | `0x0B` |
| Game | `0x08` | `0x06` |
| Wind reduction | `0x17` | `0x08` |
| ANC | `0x09` | `0x09` |
| Spatial | `0x12` | `0x12` |

Some bytes collide with a different function. Consequently Max5c is not accepted as an alias, and its codec/raw/find commands are not exposed here. A selected model and plausible status bytes are not device-identity authentication.

## Frame layouts

### Host to headphones

```text
AA BB CC  instruction  length  payload...  crc_lo crc_hi
```

`length` is a single byte, limiting the payload to 255 bytes. CRC-16/MODBUS uses initial value `0xFFFF`, reflected polynomial `0xA001`, and no final XOR. It covers instruction, length and payload, excluding the three-byte header and checksum. The numerical CRC is appended little-endian.

Device-info query (`0x04`, payload `00`):

```text
AA BB CC 04 01 00 31 91
```

Firmware query uses instruction `0x01` and payload `00`. Firmware is read-only; no update command is present.

### Headphones to host

```text
DD EE FF  instruction  success  length  payload...  crc_lo crc_hi
```

CRC-16/CCITT-FALSE uses initial value `0xFFFF`, polynomial `0x1021`, no reflection and no final XOR. It covers instruction, success, length and payload. Its numerical value is also stored little-endian on the wire. Header/length establish the candidate frame; only a matching checksum lets the decoder accept it.

The maximum response is 263 bytes: 3 header + 1 instruction + 1 success + 1 length + 255 payload + 2 checksum. The implementation treats a nonzero success byte as success; zero is rejection. Requests are sequential and matched by instruction, not a sequence number.

## CRC evidence and reference correction

Known algorithm checks for ASCII `123456789`:

| Algorithm | Result |
| --- | --- |
| MODBUS | `0x4B37` |
| CCITT-FALSE | `0x29B1` |

The upstream `testCapturedStudioProResponse` fixture is in [Tests/ProtocolTestRunner/main.swift](https://github.com/sanild/ugreen-studio-controller-macos/blob/7de78471158fab65654188b90fa775b862eb949c/Tests/ProtocolTestRunner/main.swift):

```text
DD EE FF 04 01 1E 14 FF FF A0 00 01 00 00 08 0B 00 07 00 00 00 00
02 00 00 09 00 00 04 05 00 00 00 0C 0D 0E D0 E3
```

For its response body, including instruction, success and length but excluding header/checksum:

- Payload length: 30 bytes; full frame: 38 bytes
- Captured checksum bytes: `D0 E3`, numerical value `0xE3D0`
- CCITT-FALSE calculation: `0xE3D0`, matching the capture
- MODBUS calculation: `0xD718`, not matching the capture

The initial source review found that Swift's `consumeResponseFrames` parsed frames without validating their response CRC, while its synthetic fragmentation test built a MODBUS checksum. That synthetic test is not sufficient evidence for the received-CRC algorithm. This Rust decoder uses CCITT-FALSE and tests the captured bytes. The Python project's stated response-CRC algorithm agrees.

During current verification, an independent Python standard-library calculation (`binascii.crc_hqx(body, 0xffff)`) again produced `0xE3D0`; a separate MODBUS calculation produced `0xD718`. This validates arithmetic against an existing fixture, not new Bluetooth hardware behavior.

## Implemented settings and device-info offsets

Offsets are zero-based within the device-info **payload**, following the reference `HeadphoneModels.swift` mapping and [Rust settings source](../src/settings.rs).

| Key | Set instruction | Payload byte(s) | Device-info offset |
| --- | --- | --- | --- |
| `anc` | `09` | off `A0`, ultra `A1`, general `B1`, gentle `C1`, adaptive `D1`, ambient `A2` | 3 |
| `eq` | `05` | classic through treble, `00`–`07` in listed order | 4 |
| `dual` | `06` | off `00`, on `01` | 5 |
| `game` | `08` | off `00`, on `01` | 6 |
| `prompts` | `0C` | voice `00`, beeps `02` | 16 |
| `spatial` | `12` | off `00`, on `01` | 20 |
| `volume-up-action` | `14` | none `00`, next `04`, previous `05` | 22 |
| `volume-down-action` | `15` | none `00`, next `04`, previous `05` | 23 |
| `wind` | `17` | off `00`, on `01` | 25 |

Each implemented setting writes one payload byte. EQ order is `classic`, `jazz`, `electronic`, `pop`, `classical`, `rock`, `bass`, `treble`. Volume-action entries are button-hold mappings only. No codec field or codec command is claimed.

Battery is payload offset 0; only 1–100 becomes a percentage. Unknown/absent values stay unknown. Device info must contain at least eight bytes; shorter replies fail, while later absent fields remain unavailable. A successful preflight therefore means basic device-info parsing succeeded, not that every optional field exists or the device identity is proven.

For ANC off/ambient, some firmware preserves a prior depth in the high nibble. Reading those modes and verifying their writes uses the low nibble (`0` or `2`). Other ANC values require an exact byte match.

Firmware interpretation uses the first three response payload bytes unless all three are zero, then bytes 3–5; the selected bytes are formatted as decimal `major.minor.patch`. Insufficient bytes are an error.

## Decoder behavior and limits

RFCOMM provides a stream, not message boundaries. The decoder accepts splits at arbitrary byte boundaries, concatenated frames and noise. It retains at most 263 pending bytes. A bad CRC increments a rejection counter and resynchronization resumes; non-response bytes, including unsupported unsolicited data, are discarded rather than interpreted as status.

A corrupted length can hold a partial candidate until enough data arrives or the client times out. There is no length-repair heuristic and no fabricated acknowledgement. The live client can ignore unrelated instructions while waiting. The offline `decode` command instead returns failure if any bytes were discarded, rejected, or left pending.

### Spatial-audio write replies

Retail firmware (observed on 0.2.5) applies spatial-audio writes (`0x12`)
but never returns a `DD EE FF` acknowledgement frame: the reply is an
`85 86 87 02 0A <echo>` notification followed by `85 86 87 02 05 00`.
Readback confirms the write took effect, so this implementation sends the
`0x12` frame fire-and-forget and verifies via the device-info readback
alone. All other settings use the normal request/acknowledgement/readback
flow. This matches the reference macOS app, which also sends settings
without waiting for per-write acknowledgements before refreshing.

Framing validity is weaker than model compatibility. Matching CRCs do not guarantee the selected function means the same thing on another model. Readback is required, and ambiguous writes are not automatically retried.

## Platform references

- [Microsoft Bluetooth sockets](https://learn.microsoft.com/en-us/windows/win32/bluetooth/bluetooth-and-socket): `AF_BTH`, stream sockets, RFCOMM
- [Microsoft SOCKADDR_BTH](https://learn.microsoft.com/en-us/windows/win32/api/ws2bth/ns-ws2bth-sockaddr_bth): target address and RFCOMM channel semantics
- [Microsoft BluetoothFindFirstDevice](https://learn.microsoft.com/en-us/windows/win32/api/bluetoothapis/nf-bluetoothapis-bluetoothfindfirstdevice): cached device enumeration API
- [BlueZ RFCOMM header](https://github.com/bluez/bluez/blob/master/lib/bluetooth/rfcomm.h) and [bluetoothctl documentation](https://github.com/bluez/bluez/blob/master/doc/bluetoothctl.rst)
- [Linux RFCOMM socket implementation](https://github.com/torvalds/linux/blob/master/net/bluetooth/rfcomm/sock.c)

## Licensing and provenance boundaries

Retain [the project MIT license](../LICENSE), [Studio controller MIT notice](../third-party/studio-controller-MIT.txt), and [headphone-control MIT notice](../third-party/headphone-control-MIT.txt). The [Rust dependency inventory](../third-party/RUST-DEPENDENCIES.md) covers the added terminal dependency set.

No source from the other discovered GUI candidate `tung1883/ugreen-desktop-gui` was reused. The initial research identified that its inspected tree lacked a license file; Studio Pro already had a specifically licensed reference. That historical observation is not a claim about its current repository contents.
