# Documentation

[Project README](../README.md) · [README en español](../README.es.md)

The application and primary documentation use English. The Spanish README covers the same startup, feature, safety and compatibility guidance; these detailed developer guides are in English.

## Using the application

1. [Building on Linux and Windows](building.md): prerequisites, TUI/CLI feature selection, outputs and cross-compilation limits
2. [Usage](usage.md): safe first connection, terminal keys, settings, profiles and scripting
3. [Troubleshooting](troubleshooting.md): pairing cache, connection failures, uncertain writes and terminal recovery

## Understanding and checking it

- [Architecture](architecture.md): module responsibilities, transport boundaries, validation and write flow
- [Protocol and provenance](protocol.md): model-specific mapping, frame layout, CRC evidence, upstream references and notices
- [Verification](verification.md): reproducible checks, current results, historical baseline and untested behavior
- [Optional Serena pilot](serena-pilot.md): what was observed previously and what that evidence does not show

## Read this before using hardware

Studio Pro HP206 is the reference protocol; the exact retail variant and firmware have not been verified on physical headphones in this build. A familiar device name and a successful connection are not proof that the command mapping is correct. Do not apply this protocol to HiTune Max5c, which has conflicting instruction IDs.

Start with offline commands, pair in OS settings, then request status before attempting a single reversible setting. The application deliberately has no firmware flashing, reset, raw-write, find-headphones or actual-volume command. A valid acknowledgement plus matching readback establishes the result of that operation, not universal compatibility.

## Documentation maintenance

Commands and claims should follow the source and a dated verification result. Keep historical tests separate from current checks. Update both READMEs when a user-facing feature or limitation changes.

Run the standard-library-only local link checker from the project root:

```sh
python3 scripts/check_doc_links.py
```

On Windows, use `py -3 scripts/check_doc_links.py` if Python is available through the launcher. Python is needed only for this optional documentation check, not for the Rust application. The checker validates local inline Markdown links and heading fragments in both READMEs, these guides, and the Rust dependency-notice inventory; it does not fetch external websites or prove every possible Markdown construct correct.
