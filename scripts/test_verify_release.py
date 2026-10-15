"""Release integrity tests; no executable or hardware is invoked."""
import copy
import hashlib
from pathlib import Path
import tempfile
import unittest
import warnings
import zipfile

import package_release
import verify_release


class VerifyTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        for name in ('LICENSE', 'Cargo.toml', 'Cargo.lock'):
            (self.root / name).write_text(name, encoding='utf-8')
        self.binary = self.root / 'binary'
        self.binary.write_bytes(b'offline executable fixture')

    def package(self, target='x86_64-unknown-linux-gnu', dirty=False):
        return package_release.package(
            self.root, self.binary, self.root / 'out', '1.0.0', target, 'tui',
            {'source_commit': 'a' * 40, 'dirty': dirty, 'rustc': 'fixture'})

    def checksum(self, archive):
        archive.with_suffix('.zip.sha256').write_text(
            hashlib.sha256(archive.read_bytes()).hexdigest() + '  ' + archive.name + '\n',
            encoding='ascii')

    def rewrite(self, archive, transform):
        with zipfile.ZipFile(archive) as bundle:
            entries = [(entry, bundle.read(entry)) for entry in bundle.infolist()]
        with zipfile.ZipFile(archive, 'w') as bundle:
            for entry, data in transform(entries):
                bundle.writestr(copy.copy(entry), data)
        self.checksum(archive)

    def test_linux_and_windows_integrity_and_binary_identity(self):
        for target in ('x86_64-unknown-linux-gnu', 'x86_64-pc-windows-msvc'):
            with self.subTest(target=target):
                archive = self.package(target)
                manifest = verify_release.verify(archive, source_commit='a' * 40,
                                                 require_clean=True, binary=self.binary)
                self.assertEqual(manifest['target'], target)

    def test_archive_corruption_is_rejected(self):
        archive = self.package()
        with archive.open('ab') as output:
            output.write(b'tamper')
        with self.assertRaisesRegex(ValueError, 'archive checksum'):
            verify_release.verify(archive)

    def test_internal_corruption_with_updated_outer_checksum_is_rejected(self):
        archive = self.package()
        self.rewrite(archive, lambda entries: [
            (entry, b'altered' if entry.filename.endswith('/ugreen') else data)
            for entry, data in entries])
        with self.assertRaisesRegex(ValueError, 'file checksum mismatch: ugreen'):
            verify_release.verify(archive)

    def test_missing_and_unlisted_entries_are_rejected(self):
        archive = self.package()
        self.rewrite(archive, lambda entries: entries[:-1])
        with self.assertRaisesRegex(ValueError, 'inventory mismatch'):
            verify_release.verify(archive)
        with zipfile.ZipFile(archive, 'a') as bundle:
            bundle.writestr(archive.stem + '/extra', b'unlisted')
        self.checksum(archive)
        with self.assertRaisesRegex(ValueError, 'inventory mismatch'):
            verify_release.verify(archive)

    def test_duplicate_and_unsafe_entries_are_rejected(self):
        archive = self.package()
        with warnings.catch_warnings():
            warnings.simplefilter('ignore', UserWarning)
            self.rewrite(archive, lambda entries: entries + entries[:1])
        with self.assertRaisesRegex(ValueError, 'duplicate ZIP'):
            verify_release.verify(archive)
        self.rewrite(archive, lambda entries: entries[:-1])
        with zipfile.ZipFile(archive, 'a') as bundle:
            bundle.writestr(archive.stem + '/../escape', b'unsafe')
        self.checksum(archive)
        with self.assertRaisesRegex(ValueError, 'invalid archive path'):
            verify_release.verify(archive)

    def test_provenance_and_executable_mismatch_are_rejected(self):
        archive = self.package(dirty=True)
        with self.assertRaisesRegex(ValueError, 'clean source'):
            verify_release.verify(archive, require_clean=True)
        with self.assertRaisesRegex(ValueError, 'source commit mismatch'):
            verify_release.verify(archive, source_commit='b' * 40)
        self.binary.write_bytes(b'another binary')
        with self.assertRaisesRegex(ValueError, 'supplied binary'):
            verify_release.verify(archive, binary=self.binary)


if __name__ == '__main__':
    unittest.main()
