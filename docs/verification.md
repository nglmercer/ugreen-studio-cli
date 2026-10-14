# Verification report

[Documentation index](README.md) · [Build instructions](building.md)

Verification date: 2026-10-01 UTC. This report separates current source/build checks from untested platform and hardware behavior. The optional earlier Serena pilot is historical and is documented separately.

## Current workspace checks — 2026-10-14

Linux x86-64, Rust/Cargo **1.99.0**. `HEAD` and fetched `origin/main` were
`b15bef554b29543b2f3e905149daaa9aa0e151f0`. The initial working tree already
contained uncommitted feature workspace and translation work. It failed formatting
and test compilation; those scaffolding failures were repaired without discarding
that work. The historical packaged checks below describe an earlier build.

Final local checks for this working tree:

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo clippy --all-targets --no-default-features -- -D warnings`: passed.
- `cargo test --all-features`: **290 passed**, one ignored live BlueZ test
  (252 library, 11 capture fixtures, 25 CLI contracts, two Linux interrupt tests).
- `cargo test --no-default-features`: **187 passed**, one ignored live BlueZ test
  (149 library, 11 capture fixtures, 25 CLI contracts, two interrupt tests).
- `cargo build --release --locked`: passed; `target/release/ugreen`.
- CLI-only locked release build: passed, separate output directory
  `/tmp/ugreen-cli-only-build/release/ugreen`.
- Windows MSVC **all-target source checks**, all-features and CLI-only: passed.
  No Windows executable or native Windows runtime verification is claimed.
- Seven offline Linux PTY scenarios passed against the release executable,
  restoring terminal attributes and alternate-screen state.
- Documentation links: **263 local links across 13 Markdown files**, passed.
- `git diff --check`: passed.
- Linux `pw-dump` subprocess smoke succeeded; `audio status` for a dummy Bluetooth
  address returned a valid schema-v1 object with an empty observation list.
  This checks daemon/process integration only, not a negotiated Bluetooth codec.

New offline checks cover exact-address PipeWire matching, standard/unknown codecs,
missing transport properties, malformed/oversized JSON, conflicting observations,
subprocess timeout/limit and kill/reap, JSON stdout contracts, changed-target host
replies, independent host/control/audio states, session-safe device browsing,
unknown firmware UI lockout, and evidence-aware SDK setting refusal before writes.
No new headset packet, live host connect/disconnect, audio playback, physical
capture, retail write, unpair or reset was performed. No remote CI outcome is
claimed. All static candidate live bits remain unset; no feature was promoted by
these tests. MediaTransport/adapter parsing, optional socket inventory and dedicated
view navigation were tested offline. Signal subscription and other remaining work are listed in the
[feature matrix](feature-matrix.md#milestone-delivery-and-remaining-work).

Release SHA-256:

```text
756c1a1ce129566db0f65f040a34d3a58d61a62a604b38cdcde1ff72e0945e58  target/release/ugreen
15a4f5fd4fab21e9d44fc990aa4717c69d0368e1592089014d953745d4791328  /tmp/ugreen-cli-only-build/release/ugreen
```

The binaries remain local build outputs, not a published release bundle. Current
ELF inspection shows strong GLIBC symbol requirements up to **2.34** and weak
2.39 references; this does not establish runtime compatibility on older systems.
Rebuild on the intended distribution. The earlier packaged binary descriptions
and hashes below remain historical.

## Historical packaged baseline checks

The current verification environment is Linux x86-64 with Rust/Cargo 1.98.1. The package's declared Rust minimum is 1.88; a separate 1.88 toolchain test has not been established.

Completed checks for this source and its packaged binaries:

- Default-feature suite: **97 tests passed** (82 library, comprising 30 core + 52 TUI; 4 binary unit; 11 CLI integration)
- CLI-only suite: **45 tests passed** (30 library, 4 binary unit, 11 CLI integration)
- `cargo fmt --all -- --check` passed
- Strict all-target Clippy (`-D warnings`) passed on Linux and for the Windows GNU target, with both default and CLI-only features
- Windows GNU all-target source checks passed for both feature sets; Windows tests were compiled, not executed
- Rustdoc with warnings denied passed for Linux and Windows GNU, with both feature sets
- Both Linux release binaries were built after the final production-source changes
- Five Linux pseudo-terminal scenarios passed against the exact packaged default binary, with restored terminal attributes and alternate-screen exit: explicit quit, no-argument TTY entry, Ctrl+C, small-screen resize/recovery, and address-edit cancellation
- Documentation checker passed: 241 local links across 11 Markdown files, including every dependency-notice link; parser and positive/negative link fixtures also passed
- README offline examples, all 30 documented setting/value combinations, generated-profile dry run, and captured-response decode passed against both exact packaged Linux binaries
- Independent arithmetic check of the 38-byte upstream capture: CCITT-FALSE `E3D0` matches its `D0 E3` wire checksum; MODBUS `D718` does not

These were local checks. No physical Bluetooth device was used. Remote CI should be checked for the exact published commit; local results are not a remote CI run.

## Reproducing checks

Run from the project root with the required toolchain/dependencies available:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo clippy --all-targets --locked --no-default-features -- -D warnings
cargo test --all-targets --locked
cargo test --all-targets --locked --no-default-features
cargo build --release --locked
cargo build --release --locked --no-default-features
python3 scripts/check_doc_links.py
```

The two build commands write the same normal output path. Copy the first binary elsewhere, or use distinct `--target-dir` values, if you need both variants at once. The CLI-only variant has no terminal UI.

On Windows, substitute `py -3` for `python3` where appropriate. Add `--offline` to Cargo commands only if all required toolchain/dependency data is cached. A successful run on one machine does not establish that another machine's cache is complete.

Offline executable smoke tests:

```sh
ugreen --help
ugreen models
ugreen commands
ugreen --dry-run set game on
ugreen profile example
ugreen decode 'DD EE FF 04 01 1E 14 FF FF A0 00 01 00 00 08 0B 00 07 00 00 00 00 02 00 00 09 00 00 04 05 00 00 00 0C 0D 0E D0 E3'
ugreen decode 'AA BB CC 0D 01 00 E1 93'
ugreen candidate peers
ugreen candidate codec
ugreen candidate find left on
ugreen analyze 'DD EE FF 0D 01 10 01 01 4D 79 20 50 68 6F 6E 65 AA BB CC DD EE FF 16 A0'
```

The included Linux-only smoke script exercises five offline lifecycle scenarios:

```sh
cargo build --release --locked
python3 scripts/test_tui_pty.py target/release/ugreen
```

Use a real or pseudo-terminal for TUI launch/quit checks. Redirected execution should stay in CLI help mode without arguments, and explicit TUI invocation without a terminal should fail clearly. Offline terminal checks must not press connect against a real address.

## Coverage and its limits

The core tests exercise known CRC vectors; the upstream captured frame; fragmentation boundaries; noise, corruption, concatenation and maximum-length frames; bounded arbitrary input; malformed hexadecimal text; short writes; missing/negative acknowledgements; mismatched readback; profile validation/round-trip; strict options and addresses; and terminal-safe device names.

Linux transport tests use local Unix socket pairs and bounded subprocess fixtures to test I/O, EOF, timeout and collection behavior. They do not exercise a Bluetooth controller or radio. Windows source contains SDK-layout assertions; a cross-target `cargo check` compiles them without running them.

TUI tests cover explicit target/model selection, proposals and confirmations, repeated keys, stale replies, busy-state restrictions, missing fields, sanitization, bounded logs and queues, cancellation at each operation stage, uncertain-write lockout, and small-screen/modal rendering. These tests establish only their covered local behavior. A pseudo-terminal check can establish terminal setup/cleanup and interaction paths; it does not validate Bluetooth firmware, Windows terminal behavior, audio playback or every terminal emulator.

## Packaged executables

The delivery ZIP contains two freshly compiled, stripped Linux x86-64 ELF executables:

- `bin/linux-x86_64/ugreen`: default TUI + CLI
- `bin/linux-x86_64/ugreen-cli-only`: CLI without terminal dependencies

SHA-256 of the packaged files:

```text
01ac42838ab263faaa15d9b99d6b6b5bc020a9f113dfcf75e2a3eded81fb6004  bin/linux-x86_64/ugreen
25ccd3d7ddc3a584fd86fefac2aa8425f357d9891401ae76a68e09338e99a6d2  bin/linux-x86_64/ugreen-cli-only
```

The actual executables were inspected with `readelf` and `ldd`; both require glibc **2.39 or newer** and `libgcc_s.so.1`. Rebuild on your distribution if it provides an older glibc. No Windows executable is included. Toolchains, dependency caches and intermediate build directories are excluded from the delivery ZIP.

## Live retail verification (2026-10-04)

Hardware: UGREEN Studio Pro, firmware **0.2.5**, RFCOMM channel 1, Linux
x86-64 (CachyOS), BlueZ adapter. Retail state at connect: battery 90%,
`anc=off eq=classic game=off spatial=off dual=off wind=on prompts=voice
volume-up-action=next volume-down-action=previous`.

- `status` and `profile export` parse cleanly: battery, all nine settings,
  firmware and 30-byte raw device info
- Same-value writes verified for `anc`, `eq`, `game`, `dual`, `wind`,
  `prompts`, `volume-up-action`, `volume-down-action`
- `game=on`/`game=off` reversible cycle verified, device restored
- **Spatial-audio finding:** `spatial=on` and `spatial=off` both take effect
  (device-info offset 20 flips and reads back), but the firmware never sends
  a `DD EE FF` ack for `0x12` — it replies `85 86 87 02 0A <echo>` plus
  `85 86 87 02 05 00`. Fixed: spatial writes skip the ack wait and verify
  via readback alone; device left at its original `spatial=off`
- Occasional transient `EBUSY` (os error 16) on rapid back-to-back
  connections; retry succeeds. One stale `target/debug/ugreen` process held
  the channel during testing and was killed

## Multi-device refactor checks (2026-10-04)

The host-side refactor (Bluetooth enumeration, device registry, protocol
split, multipoint scaffolding, capture tooling) was verified locally on
Linux x86-64:

- Default-feature suite: **199 tests passed** (170 library, 7 binary unit,
  6 fixture-capture integration, 16 CLI integration)
- CLI-only suite: **114 tests passed** (85 library, 7 binary unit,
  6 fixture-capture integration, 16 CLI integration)
- Autoconnect is on by default: a fresh registry persists `autoconnect=1`,
  an explicit `--no-autoconnect` choice round-trips as `autoconnect=0`,
  and an unparseable value keeps the default instead of guessing off
- Header linkage: an open RFCOMM session holds `Bluetooth:` at
  `CONNECTED` (the session proves the link), while a failed connect
  never claims it and a stale pairing list cannot downgrade a live
  session; closing the session still leaves `Bluetooth:` untouched
- `cargo fmt --all -- --check` passed
- Strict all-target Clippy (`-D warnings`) passed with default,
  all-features and CLI-only feature sets
- `cargo check --target x86_64-pc-windows-msvc` passed with default and
  all features
- **Live D-Bus enumeration:** `ugreen discover` listed two real paired
  devices through the system bus on the test machine
- Live RFCOMM smoke (`capture`/`status`) hit transient `EBUSY` because
  another process held channel 1; the command failed cleanly with the
  documented guidance and printed no partial stdout before connection
- Documentation checker passed: 244 local links across 11 Markdown files,
  including fixture-directory and capture-output links added by this work
- Fixture suite (`tests/fixtures/protocol/`) decodes the upstream
  device-info response, both observed notifications, a mixed
  response/notification stream, and the TX-layout-on-RX unknown case
- Peer operations were exercised only in mock tests: they refuse with
  `UnsupportedFeature` before writing any byte

Live RFCOMM command verification against retail firmware remains the
earlier 2026-10-04 session above; the enumeration and offline paths are
what the refactor changed.

## Not established

- No native Windows runtime test has been established for this version
- A Windows release executable is not claimed merely because source or cross-target checks pass
- No remote CI result is implied by the presence of [the workflow](../.github/workflows/ci.yml)
- macOS and other platforms have no Bluetooth backend here
- A passing test count is not a hardware-compatibility certification

## Recommended hardware validation record

When testing on your own confirmed Studio Pro pair, record the exact retail/model label, firmware, OS, adapter, application version, build feature set and selected RFCOMM channel. Begin with read-only status. Record raw replies and check CRC before a single supported reversible setting, then compare the acknowledgement and readback. Stop on incompatible or ambiguous behavior. Redact device addresses before sharing captures publicly.

Do not use unsupported models, raw writes, firmware updates or resets as validation shortcuts. Those actions are outside this application's interface.

## Reverse-engineering implementation checks (2026-10-12)

Static APK analysis is implemented separately from verified live status.
The offline `candidate` command covers query, finder, LDAC, buttons, AI,
volume, channel, audio sharing and reset layouts. `analyze` validates framing
and checksums before interpreting responses and accumulating peer updates.
These are implementation checks, not official-app hardware captures.

Peer, codec, ID, color and box-MAC queries now have gated client/CLI paths.
Mock peer-collection checks cover short writes, fragmented and coalesced
responses, interleaved notifications, explicit empty/single/two-peer
completion, missing peers, rejection and malformed payloads. Separate
checks prove all current query gates refuse before writing or connecting.

The APK's notification parser was inspected directly: it switches on
byte 5, not byte 4. Static parsing now reflects that offset and leaves
additional battery/finder values absent under the captured six-byte
framing. Live notification semantics and framing have not been promoted.

Remaining verification is the Phase-3 official-app capture work described
in [the report](reverse-engineering.md#phase-3-verification-recipe-live-hardware).
TUI feature integration is recorded below; these offline checks do not establish physical feature compatibility.

Phase-2 offline artifacts: ten synthetic candidate fixtures under
`tests/fixtures/protocol/` (peer query and both peer-response shapes,
codec query/response, both find-headphones start frames, the
finder-state notification, box-MAC and factory-reset requests) carry
provenance comments stating they are constructed from the report, not
captured. Checks prove each request matches the offline encoder, each
response decodes with a valid CCITT checksum, and peer payloads parse
into the documented `[flag][index][name][mac]` layout. `decode` now
prints `feature=<name>` for capture-verified instructions and
`feature=<name> (unverified)` for static candidates; notification
output separates the envelope byte from the event code at frame byte 5.
`describe_instruction` and `describe_notification_event` label the
verified set apart from the static candidates, and reserved slots
(`0x13`, `0x1A`, `0x1D`) stay deliberately unmapped. These are
offline framing and labeling checks only.

Typed control execution is also implemented behind independent capture
gates. Mock checks validate all controls refuse before I/O, LDAC/channel
acknowledgement values match, writes are not implicitly retried, readback
fields match, and sharing enable requires wind reduction off. Finder
sends report an unconfirmed outcome. CLI checks verify control gates
refuse before connecting and `--dry-run control` stays offline. These
checks do not verify playback, reset host cleanup or physical controls.

Reset cleanup now has Linux BlueZ and Windows API implementations.
Offline checks cover exact-address selection, multiple-adapter ambiguity,
D-Bus object-path marshalling, registry preservation and cleanup order/error
reporting. No host pairing was removed by these checks.

Finder tone checks validate WAV lengths, silent opposite channels, amplitude
bounds and invalid durations. Mock orchestration checks stop after failed
start/playback and report failed stop. CLI checks refuse before wire/audio
access. Actual PipeWire playback, audibility and Windows playback remain
unverified. Native interruption checks are recorded below.

Finder interruption handling is implemented using Linux sigaction with
SDK layouts supplied by pinned libc, and Windows SetConsoleCtrlHandler.
Isolated Linux subprocess tests deliver SIGINT and SIGTERM, check the
cancellation flag, and verify prior handlers/masks are restored. Separate
checks verify player cancellation reaps the spawned test process and
private generated tone files are removed. No audio was played by these
checks. Windows waveform playback has compile-time SDK-layout assertions
and passes source/Clippy checks; no Windows audio or console runtime test
is claimed. Official-app captures remain pending.

TUI feature operations now run through the session-owning worker. Offline
checks cover feature-panel input bounds, capture refusals before preflight,
confirmation/review, edit cancellation, busy-state restrictions, request-ID
matching, uncertain-write lockout, cancellation before/after writes and
reset session-close ordering for the actual connected address. The native
backend still uses the model's unchanged capture-backed capabilities.
Fake capabilities used in tests belong only to mock backends; no production
capability or hardware evidence is changed by those tests.

Six offline PTY scenarios pass with temporary registry files and startup
connection disabled, including feature-panel capture refusal and editor
cancellation. All exit cleanly and restore terminal attributes. Codec
query results update the header as a last-query value and are invalidated
by subsequent writes; rendering tests cover that distinction.

## Headset connections follow-up

The dedicated multipoint page, unknown-peer semantics, host-list isolation,
zero-query refresh, dual-setting navigation, all-language page text and small
terminals were tested offline. Candidate peer parsing now rejects payloads over
255 bytes and clear records with trailing data. The additional PTY scenario
opens the peer page, checks its gate, returns to dual-device settings and exits;
no RFCOMM connection or candidate packet is requested. Runtime peer identity,
slot status and remote management remain unverified and disabled.
