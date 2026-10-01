#!/usr/bin/env python3
"""Check local inline links in READMEs, docs and Rust notices, without network I/O.

This intentionally covers the simple Markdown used in these guides, not every
Markdown extension. Reference-style links, HTML anchors and external link
availability are outside this check. Python 3.8+; standard library only.
"""

import re
import sys
from pathlib import Path
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]
LINK = re.compile(r"\[[^\]\n]*\]\(([^\s)]+)(?:\s+\"[^\"]*\")?\)")
HEADING = re.compile(r"^ {0,3}#{1,6}\s+(.+?)\s*#*\s*$")


def visible_lines(text):
    """Skip fenced code so examples are not mistaken for links/headings."""
    fence = None
    for number, line in enumerate(text.splitlines(), 1):
        marker = re.match(r"^ {0,3}(`{3,}|~{3,})", line)
        if marker:
            value = marker.group(1)
            if fence is None:
                fence = value
            elif value[0] == fence[0] and len(value) >= len(fence):
                fence = None
            continue
        if fence is None:
            yield number, line


def heading_ids(text):
    """GitHub-style IDs for the plain headings used by this documentation."""
    identifiers = set()
    counts = {}
    for _, line in visible_lines(text):
        match = HEADING.match(line)
        if not match:
            continue
        title = re.sub(r"[`*_]", "", match.group(1)).lower()
        slug = re.sub(r"[^\w\- ]", "", title).replace(" ", "-")
        count = counts.get(slug, 0)
        counts[slug] = count + 1
        identifiers.add(slug if count == 0 else "{}-{}".format(slug, count))
    return identifiers


def main():
    paths = sorted(
        set(ROOT.glob("README*.md"))
        | set((ROOT / "docs").rglob("*.md"))
        | {ROOT / "third-party" / "RUST-DEPENDENCIES.md"}
    )
    failures = []
    checked = 0
    for source in paths:
        text = source.read_text(encoding="utf-8")
        for number, line in visible_lines(text):
            for match in LINK.finditer(line):
                raw = match.group(1).strip("<>")
                url = urlsplit(raw)
                if url.scheme or url.netloc:
                    continue
                checked += 1
                target = (source.parent / unquote(url.path)).resolve() if url.path else source
                label = "{}:{}: {}".format(source.relative_to(ROOT), number, raw)
                try:
                    target.relative_to(ROOT)
                except ValueError:
                    failures.append(label + " (outside project)")
                    continue
                if not target.exists():
                    failures.append(label + " (missing target)")
                    continue
                if url.fragment and target.suffix.lower() == ".md":
                    ids = heading_ids(target.read_text(encoding="utf-8"))
                    if unquote(url.fragment) not in ids:
                        failures.append(label + " (missing heading)")
    if failures:
        print("Documentation link check failed:", file=sys.stderr)
        for failure in failures:
            print("  " + failure, file=sys.stderr)
        return 1
    print("Checked {} local links in {} Markdown files; no broken targets or headings.".format(checked, len(paths)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
