# Testing preferences

- Capture golden/snapshot outputs of current behavior (help, errors, dry-run, decode, status formats) before restructuring, and use them as regression baselines. Confidence: 0.9
- Establish regression tests before refactoring; don't delete old modules until their replacements pass those tests. Confidence: 0.9
- Registry-driven code should be tested for consistency: every registered command has a handler, generated help is coherent, invalid commands fail before any hardware access, and dry runs never connect. Confidence: 0.8
