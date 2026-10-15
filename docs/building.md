# Building on Linux and Windows

[Documentation index](README.md) · [Verification results](verification.md)

## Toolchain and features

Use Rust 1.88 or newer; a recent stable toolchain is the simplest choice. The declared minimum is not evidence of a test on that exact compiler. Install Rust through the [official installation page](https://rust-lang.org/tools/install/) and check `rustc --version` and `cargo --version` in a new terminal.

The binary name is `ugreen`; the Cargo package is `ugreen-cli`. `Cargo.lock` is included for reproducible dependency resolution. Run commands below from the directory containing `Cargo.toml`.

| Build | Command | Contents |
| --- | --- | --- |
| Default | `cargo build --release --locked` | CLI plus optional-in-source, default-enabled TUI |
| CLI only | `cargo build --release --locked --no-default-features` | No TUI dependencies; Linux uses libc for signal handling |
| Explicit TUI | `cargo build --release --locked --no-default-features --features tui` | Same terminal feature enabled explicitly |

The `tui` feature selects Ratatui 0.30.2 and Crossterm 0.29.0. Their transitive dependencies are pinned by the lockfile. The complete default build has third-party Cargo dependencies; the CLI-only variant omits terminal dependencies and uses pinned libc 0.2.189 on Linux for native signal layouts. Windows uses direct native APIs.

`--locked` prevents an unnoticed lockfile update; it does not mean offline. If building without a network connection, first obtain the correct toolchain and cache the locked dependencies on a connected machine using Cargo's supported workflow. Then add `--offline`. A Linux CLI-only build compiles libc; other platforms omit that target-specific dependency. Cargo can still require registry metadata for lockfile resolution. Do not delete `Cargo.lock` to hide a dependency-resolution error.

## Linux

Prerequisites:

- Rust and a working system C linker/toolchain
- For hardware use: a Bluetooth Classic adapter, kernel RFCOMM support, and headphones paired through the OS
- For `discover` and the TUI paired-device picker: BlueZ's `bluetoothctl`, supporting `devices Paired`

Direct connection to an explicitly supplied address uses native sockets. It does not invoke `bluetoothctl`, need a Python package, or link to `libbluetooth`. The transport is not a BLE GATT implementation. The source guards unsupported Linux CPU ABIs; a Linux target name alone is not proof that an architecture has been validated.

```sh
cargo test --all-targets --locked
cargo build --release --locked
./target/release/ugreen --help
./target/release/ugreen --dry-run set game on
./target/release/ugreen tui
```

The release output is `target/release/ugreen`. The earlier delivery ZIP contained `bin/linux-x86_64/ugreen` (default features) and `bin/linux-x86_64/ugreen-cli-only`. Both packaged Linux x86-64 binaries require glibc 2.39 or newer and `libgcc_s.so.1`; rebuild locally for older distributions. These executables are delivery artifacts, not committed build outputs. Refer to the [verification report](verification.md) for tested behavior and remaining limits.

## Windows

Use a native Windows terminal and the MSVC Rust toolchain for the usual setup. Follow Rust's [MSVC prerequisites](https://rust-lang.github.io/rustup/installation/windows-msvc.html) to install Microsoft C++ build tools and a Windows SDK if needed. A complete MinGW toolchain is an alternative for the Windows GNU target, but selecting a Rust target alone does not install its linker or import libraries.

```powershell
rustc --version
cargo test --all-targets --locked
cargo build --release --locked
.\target\release\ugreen.exe --help
.\target\release\ugreen.exe --dry-run set game on
.\target\release\ugreen.exe tui
```

The release output is `target\release\ugreen.exe`. The backend calls Microsoft Bluetooth/Winsock directly, linking `ws2_32` and `bthprops`. Pair in Windows Bluetooth settings. The OS/device stack must support Bluetooth Classic RFCOMM; a working audio route alone does not verify the vendor control channel. See Microsoft's [Bluetooth socket documentation](https://learn.microsoft.com/en-us/windows/win32/bluetooth/bluetooth-and-socket).

PowerShell may require the `.\` prefix when invoking a binary in the current directory. A Linux executable cannot run as a Windows `.exe`; renaming it does not convert it. WSL is not evidence of native Windows Bluetooth transport support.

## Cross-compilation is only one layer

After installing the desired Rust target, a source check may be run from another OS:

```sh
cargo check --all-targets --locked --target x86_64-pc-windows-gnu
cargo check --all-targets --locked --no-default-features --target x86_64-pc-windows-gnu
```

A successful check type-checks target-specific source. It does not link an executable, execute Windows tests, exercise Windows terminal restoration, or communicate with a device. Linking additionally requires a compatible Windows linker and import/runtime libraries. A native Windows CI run is a separate result, and a real-headphone trial is another.

## Developer checks

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo clippy --all-targets --locked --no-default-features -- -D warnings
cargo test --all-targets --locked
cargo test --all-targets --locked --no-default-features
python3 scripts/check_doc_links.py
```

The workflow in [CI](../.github/workflows/ci.yml) covers Linux and Windows with default and CLI-only feature sets. Consult [Verification](verification.md) before treating a check as completed. Do not use a cross-target check or mock test to claim physical compatibility.

## Other systems

Offline parsing and profiles are separate from transport, but macOS and other OS targets return an unsupported-platform error for Bluetooth operations. The upstream Swift project is a protocol reference; its macOS support does not confer macOS support on this Rust program.

## Release bundles

The [release workflow](../.github/workflows/release.yml) builds/tests both variants
on native Linux and Windows runners using Rust 1.88.0. It uploads ZIPs as workflow
artifacts; it does not publish a GitHub release or establish headset compatibility.
Run it manually or from a version tag. Native Windows hardware validation remains
separate from those build/test jobs.

After a fresh locked build from a clean checkout, package the exact executable:

```sh
python3 scripts/package_release.py target/release/ugreen --target x86_64-unknown-linux-gnu --variant tui --output target/release-bundles
python3 scripts/verify_release.py target/release-bundles --require-clean --binary target/release/ugreen
```

For Windows, use the `.exe` path and `--target x86_64-pc-windows-msvc`. For CLI-only
builds, use `--variant cli-only`. The packager includes license notices, Markdown
documentation, public evidence records, Rust source/tests, build scripts, the lockfile and a manifest recording source commit, compiler,
variant, target and SHA-256 for every bundled file. It refuses to overwrite an
archive. `--allow-dirty` is only for development artifacts and records that state.

The verifier accepts a ZIP or a directory of ZIPs and checks the outer checksum,
manifest inventory, every file hash, executable identity and fixed ZIP metadata
without extracting or executing files. The release workflow also requires clean
provenance and an exact match to its source revision before upload. When checking
a downloaded bundle, pass `--source-commit` with an independently trusted full
revision. Checksums alone do not authenticate a publisher. For a development
bundle, omit `--require-clean`; this does not change its recorded dirty status.

ZIP order, timestamps, permissions and uncompressed storage are fixed, so identical
inputs yield byte-identical bundles independently of host timestamps or zlib
versions. This establishes packaging reproducibility, not identical compiler output
across toolchain/OS versions; binary hashes and compiler identity make that limit
reviewable. Keep the compiler, linker, OS image and build inputs fixed when checking
binary reproducibility.
