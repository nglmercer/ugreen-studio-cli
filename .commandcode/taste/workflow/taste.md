# Workflow preferences

- Before refactoring: verify the baseline build and test suite are green, then capture golden outputs of current CLI behavior into a scratchpad for later diffing. Confidence: 0.85
- Use small Python scripts for mechanical, repetitive multi-file edits (e.g. stripping the same field from 11 translation catalogs) instead of editing each file by hand. Confidence: 0.8
- When reproducing existing output format (alignment columns, spacing), measure the exact values from the golden output rather than assuming them from memory. Confidence: 0.8
