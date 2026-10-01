# Verification report

[Documentation index](README.md) · [Build instructions](building.md)

Verification date: 2026-10-01 UTC. This report separates current source/build checks from untested platform and hardware behavior. The optional earlier Serena pilot is historical and is documented separately.

## Current checks

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

## Not established

- No physical headphone or Bluetooth-adapter test: connection, battery, firmware, settings, acknowledgements and readback on actual retail Studio Pro firmware remain unverified
- No native Windows runtime test has been established for this version
- A Windows release executable is not claimed merely because source or cross-target checks pass
- No remote CI result is implied by the presence of [the workflow](../.github/workflows/ci.yml)
- macOS and other platforms have no Bluetooth backend here
- A passing test count is not a hardware-compatibility certification

## Recommended hardware validation record

When testing on your own confirmed Studio Pro pair, record the exact retail/model label, firmware, OS, adapter, application version, build feature set and selected RFCOMM channel. Begin with read-only status. Record raw replies and check CRC before a single supported reversible setting, then compare the acknowledgement and readback. Stop on incompatible or ambiguous behavior. Redact device addresses before sharing captures publicly.

Do not use unsupported models, raw writes, firmware updates or resets as validation shortcuts. Those actions are outside this application's interface.
