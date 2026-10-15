"""Deterministic packaging checks using tiny offline fixtures."""
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
import zipfile

import package_release as release


class PackageTests(unittest.TestCase):
    def test_archive_is_deterministic_and_contains_matching_manifest_and_notices(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'LICENSE').write_text('license', encoding='utf-8')
            (root / 'Cargo.lock').write_text('lock', encoding='utf-8')
            (root / 'Cargo.toml').write_text('[package]\nversion="1.2.3"\n', encoding='utf-8')
            (root / 'docs' / 'evidence').mkdir(parents=True)
            (root / 'docs' / 'evidence' / 'read.json').write_text('{}', encoding='utf-8')
            (root / 'docs' / 'unredistributed.apk').write_bytes(b'excluded reference')
            (root / 'third-party').mkdir()
            (root / 'third-party' / 'NOTICE').write_text('upstream', encoding='utf-8')
            binary = root / 'test-binary'
            binary.write_bytes(b'offline binary fixture')
            first = release.package(root, binary, root / 'one', '1.2.3', 'x86_64-linux', 'tui', {'dirty': False})
            second = release.package(root, binary, root / 'two', '1.2.3', 'x86_64-linux', 'tui', {'dirty': False})
            self.assertEqual(first.read_bytes(), second.read_bytes())
            checksum = first.with_suffix('.zip.sha256').read_text().split()[0]
            self.assertEqual(checksum, hashlib.sha256(first.read_bytes()).hexdigest())
            with zipfile.ZipFile(first) as archive:
                prefix = first.stem + '/'
                manifest = json.loads(archive.read(prefix + 'manifest.json'))
                self.assertIn('third-party/NOTICE', manifest['files_sha256'])
                self.assertIn('docs/evidence/read.json', manifest['files_sha256'])
                self.assertNotIn('docs/unredistributed.apk', manifest['files_sha256'])
                self.assertIn('Cargo.toml', manifest['files_sha256'])
                for name, expected in manifest['files_sha256'].items():
                    self.assertEqual(hashlib.sha256(archive.read(prefix + name)).hexdigest(), expected)
                self.assertEqual(archive.getinfo(prefix + 'ugreen').external_attr >> 16, 0o100755)
            with self.assertRaises(FileExistsError):
                release.package(root, binary, root / 'one', '1.2.3', 'x86_64-linux', 'tui', {'dirty': False})

    def test_windows_archive_preserves_executable_name(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            binary = root / 'binary.exe'
            binary.write_bytes(b'fake exe')
            # A minimal source fixture contains the required metadata files.
            (root / 'LICENSE').write_text('license')
            (root / 'Cargo.lock').write_text('lock')
            archive = release.package(root, binary, root / 'out', '1.0.0', 'x86_64-pc-windows-msvc', 'cli-only', {})
            with zipfile.ZipFile(archive) as data:
                self.assertIn(archive.stem + '/ugreen.exe', data.namelist())


if __name__ == '__main__':
    unittest.main()
