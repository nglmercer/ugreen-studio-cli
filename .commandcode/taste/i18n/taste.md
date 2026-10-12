# Internationalization preferences

- Translate interface prose only (labels, descriptions, error messages). Protocol identifiers — setting keys, value names, wire bytes, command syntax, output keys — stay language-independent so scripts and profiles keep working when the display language changes. Confidence: 0.95
- Prefer resource-based translations (e.g. `.ftl` catalogs) over large localized code structs; the help screen should be generated from a command registry, with only prose fragments localized. Confidence: 0.85
- Support explicit CLI language selection, environment detection, and English fallback; keep all supported locales rendering generated output. Confidence: 0.8
