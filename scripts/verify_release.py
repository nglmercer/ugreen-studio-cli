#!/usr/bin/env python3
"""Verify a finished release ZIP without extracting or executing its contents.

Checksums establish integrity, not publisher authenticity or hardware support.
Compare --source-commit with an independently trusted revision when downloading.
"""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import sys
import zipfile


def sha256_file(path):
    with path.open('rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError('duplicate manifest key: ' + key)
        result[key] = value
    return result


def verify(archive, *, source_commit=None, require_clean=False, binary=None):
    checksum = archive.with_suffix(archive.suffix + '.sha256').read_text(encoding='ascii')
    expected = sha256_file(archive) + '  ' + archive.name + '\n'
    if checksum != expected:
        raise ValueError('archive checksum or checksum filename mismatch')
    prefix = archive.stem + '/'
    with zipfile.ZipFile(archive) as bundle:
        entries = bundle.infolist()
        names = [entry.filename for entry in entries]
        if len(set(names)) != len(names):
            raise ValueError('duplicate ZIP entries')
        for entry in entries:
            name = entry.filename
            relative = name.removeprefix(prefix)
            parts = PurePosixPath(relative).parts
            if (not name.startswith(prefix) or not relative or '\\' in relative
                    or ':' in relative or relative.startswith('/')
                    or any(part in ('', '.', '..') for part in relative.split('/'))
                    or not parts):
                raise ValueError('invalid archive path: ' + name)
        manifest = json.loads(bundle.read(prefix + 'manifest.json'), object_pairs_hook=unique_object)
        if not isinstance(manifest, dict) or manifest.get('schema_version') != 1:
            raise ValueError('unsupported manifest schema')
        version, target, variant = (manifest.get(key) for key in ('package_version', 'target', 'variant'))
        if (not isinstance(version, str) or not isinstance(target, str)
                or not re.fullmatch(r'[a-zA-Z0-9_.-]+', version + target)
                or variant not in ('tui', 'cli-only')
                or archive.stem != f'ugreen-{version}-{target}-{variant}'):
            raise ValueError('manifest identity does not match archive name')
        provenance = manifest.get('provenance')
        if not isinstance(provenance, dict):
            raise ValueError('missing source provenance')
        if source_commit is not None and provenance.get('source_commit') != source_commit:
            raise ValueError('source commit mismatch')
        if require_clean and provenance.get('dirty') is not False:
            raise ValueError('release requires explicitly clean source provenance')
        if manifest.get('hardware_validation') != 'not_established_by_packaging':
            raise ValueError('unexpected hardware validation claim')
        hashes = manifest.get('files_sha256')
        if not isinstance(hashes, dict) or set(names) != {prefix + name for name in hashes} | {prefix + 'manifest.json'}:
            raise ValueError('manifest file inventory mismatch')
        executable = 'ugreen.exe' if 'windows' in target else 'ugreen'
        if not {'LICENSE', 'Cargo.lock', 'Cargo.toml', executable}.issubset(hashes):
            raise ValueError('missing required bundle files')
        for entry in entries:
            relative = entry.filename[len(prefix):]
            mode = 0o100755 if relative == executable else 0o100644
            if (entry.date_time != (1980, 1, 1, 0, 0, 0)
                    or entry.compress_type != zipfile.ZIP_STORED
                    or entry.create_system != 3 or entry.external_attr >> 16 != mode):
                raise ValueError('unexpected ZIP metadata: ' + relative)
            if relative == 'manifest.json':
                continue
            with bundle.open(entry) as source:
                actual = hashlib.file_digest(source, 'sha256').hexdigest()
            if actual != hashes[relative]:
                raise ValueError('file checksum mismatch: ' + relative)
        if names != sorted(names):
            raise ValueError('ZIP entries are not sorted')
        if binary is not None and sha256_file(binary) != hashes[executable]:
            raise ValueError('bundled executable differs from supplied binary')
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('archive', type=Path, help='ZIP or directory containing release ZIPs')
    parser.add_argument('--source-commit')
    parser.add_argument('--require-clean', action='store_true')
    parser.add_argument('--binary', type=Path)
    args = parser.parse_args()
    try:
        archives = sorted(args.archive.glob('*.zip')) if args.archive.is_dir() else [args.archive]
        if not archives:
            raise ValueError('no release ZIPs found')
        for archive in archives:
            manifest = verify(archive, source_commit=args.source_commit,
                              require_clean=args.require_clean, binary=args.binary)
            print(f"Verified {archive.name}: {len(manifest['files_sha256'])} files")
    except (OSError, ValueError, KeyError, TypeError, zipfile.BadZipFile) as error:
        print(str(error), file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
