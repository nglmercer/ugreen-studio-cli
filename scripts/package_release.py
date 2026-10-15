#!/usr/bin/env python3
"""Create deterministic release ZIPs and SHA-256 files; never publishes them.

Build the exact binary from the recorded source before calling this script.
Dirty worktrees require --allow-dirty and remain marked as development artifacts.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def digest(data):
    return hashlib.sha256(data).hexdigest()


def package(root, binary, output, version, target, variant, provenance):
    if not re.fullmatch(r'[a-zA-Z0-9_.-]+', version + target):
        raise ValueError('invalid version or target for archive filename')
    if variant not in ['tui', 'cli-only']:
        raise ValueError('variant must be tui or cli-only')
    name = 'ugreen-' + version + '-' + target + '-' + variant
    output.mkdir(parents=True, exist_ok=True)
    archive = output / (name + '.zip')
    checksum = output / (name + '.zip.sha256')
    if archive.exists() or checksum.exists():
        raise FileExistsError('release archive/checksum already exists')
    files = {'LICENSE': (root / 'LICENSE').read_bytes(),
             'Cargo.lock': (root / 'Cargo.lock').read_bytes()}
    sources = set(root.glob('README*.md')) | {root / 'Cargo.toml'}
    for directory, suffixes in [('docs', {'.md', '.json'}), ('src', {'.rs'}),
                                ('tests', {'.rs', '.hex'}), ('scripts', {'.py'}),
                                ('.github', {'.yml', '.yaml'})]:
        sources.update(source for source in (root / directory).rglob('*') if source.suffix in suffixes)
    sources.update((root / 'third-party').rglob('*'))
    for source in sorted(sources):
        if source.is_file():
            files[source.relative_to(root).as_posix()] = source.read_bytes()
    executable = 'ugreen.exe' if 'windows' in target else 'ugreen'
    files[executable] = binary.read_bytes()
    manifest = {
        'schema_version': 1, 'package_version': version, 'target': target,
        'variant': variant, 'provenance': provenance,
        'files_sha256': {name: digest(data) for name, data in sorted(files.items())},
        'hardware_validation': 'not_established_by_packaging',
    }
    files['manifest.json'] = (json.dumps(manifest, sort_keys=True, indent=2) + '\n').encode()
    # ZIP_STORED removes compressor-version variability. Fixed metadata and entry
    # order make identical inputs produce an identical archive on either OS.
    with zipfile.ZipFile(archive, 'x', compression=zipfile.ZIP_STORED) as destination:
        for path, data in sorted(files.items()):
            info = zipfile.ZipInfo(name + '/' + path, date_time=(1980, 1, 1, 0, 0, 0))
            info.create_system = 3
            info.external_attr = (0o100755 if path == executable else 0o100644) << 16
            destination.writestr(info, data)
    checksum.write_text(digest(archive.read_bytes()) + '  ' + archive.name + '\n', encoding='ascii')
    return archive


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('binary', type=Path)
    parser.add_argument('--target', required=True)
    parser.add_argument('--variant', choices=['tui', 'cli-only'], required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--allow-dirty', action='store_true', help='Development packaging only')
    args = parser.parse_args()
    try:
        dirty = bool(git('status', '--porcelain', '--untracked-files=normal'))
        if dirty and not args.allow_dirty:
            raise ValueError('worktree is dirty; commit source or explicitly use --allow-dirty for a development artifact')
        version = tomllib.loads((ROOT / 'Cargo.toml').read_text(encoding='utf-8'))['package']['version']
        if os.environ.get('GITHUB_REF_TYPE') == 'tag' and os.environ.get('GITHUB_REF_NAME') != 'v' + version:
            raise ValueError('release tag must exactly match Cargo package version')
        provenance = {'source_commit': git('rev-parse', 'HEAD'), 'dirty': dirty,
                      'rustc': subprocess.check_output(['rustc', '--version'], text=True).strip()}
        archive = package(ROOT, args.binary.resolve(strict=True), args.output, version,
                          args.target, args.variant, provenance)
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        print(str(error), file=sys.stderr)
        return 1
    print(archive)
    return 0


if __name__ == '__main__':
    sys.exit(main())
