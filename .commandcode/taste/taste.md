# General engineering preferences

- Declarative registry as the single authority: define a command/setting once, and derive parsing, help, validation, completions, docs, and TUI choices from that definition. Adding a feature should require one registration and one handler, never edits to a central dispatch. Confidence: 0.95
- Preserve existing verification and safety boundaries (Bluetooth transport, protocol, capability gates); refactor around them rather than rewriting them. Confidence: 0.9
- Prefer minimal dependencies: a lightweight custom parser is acceptable if it consumes the same registry; pull in a framework (e.g. clap) only when its value is clear. Confidence: 0.8
- Use structured typed errors (e.g. `AppError`/`CliError` with stable variants) instead of ad-hoc `String` errors; localize messages only at presentation boundaries. Confidence: 0.85
- Offline/read-only operations must never initiate hardware or Bluetooth connections; dry runs print packets without touching devices. Confidence: 0.9
- Every build configuration must pass: `cargo fmt`, strict Clippy, and tests under both default features and `--no-default-features`. Confidence: 0.85
