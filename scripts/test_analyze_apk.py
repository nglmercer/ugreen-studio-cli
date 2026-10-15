"""Offline tests for binary-manifest decoding and APK evidence failures."""
import io
import struct
import unittest
from unittest.mock import patch
import zipfile

import analyze_apk as analysis


def chunk(kind, header, payload):
    return struct.pack('<HHI', kind, len(header) + 8, len(header) + len(payload) + 8) + header + payload


def manifest(utf8=True, package='com.example.test'):
    strings = ['manifest', 'package', package, 'versionCode', 'versionName', '2.6.0.933']
    encoded = bytearray()
    offsets = []

    def length8(value):
        return bytes([value]) if value < 128 else bytes([0x80 | value >> 8, value & 255])

    for text in strings:
        offsets.append(len(encoded))
        if utf8:
            data = text.encode('utf-8')
            encoded.extend(length8(len(text)) + length8(len(data)) + data + b'\0')
        else:
            encoded.extend(struct.pack('<H', len(text)) + text.encode('utf-16le') + b'\0\0')
    pool = chunk(1, struct.pack('<IIIII', len(strings), 0, 0x100 if utf8 else 0,
                               28 + 4 * len(strings), 0),
                 struct.pack('<' + 'I' * len(offsets), *offsets) + encoded)
    attrs = b''.join(struct.pack('<IIIHBBI', 0xffffffff, key, raw, 8, 0, kind, value)
                     for key, raw, kind, value in [(1, 2, 3, 2), (3, 0xffffffff, 0x10, 933), (4, 5, 3, 5)])
    node = chunk(0x102, struct.pack('<II', 1, 0xffffffff),
                 struct.pack('<IIHHHHHH', 0xffffffff, 0, 20, 20, 3, 0, 0, 0) + attrs)
    return chunk(3, b'', pool + node)


class AnalysisTests(unittest.TestCase):
    def test_manifest_decodes_both_string_pool_encodings_and_integer_version(self):
        for utf8 in [True, False]:
            self.assertEqual(analysis.android_metadata(manifest(utf8)), {
                'manifest': {'package': 'com.example.test', 'versionCode': 933, 'versionName': '2.6.0.933'}})

    def test_utf8_extended_lengths_and_multibyte_content(self):
        package = 'x' * 130 + 'é' * 70
        self.assertEqual(analysis.android_metadata(manifest(package=package))['manifest']['package'], package)

    def test_invalid_manifest_is_rejected(self):
        for data in [b'not Android XML', b'\x03\0\x08\0' + b'\0' * 12]:
            with self.assertRaises(ValueError):
                analysis.android_metadata(data)

    def test_missing_source_anchor_is_not_treated_as_evidence(self):
        output = io.BytesIO()
        with zipfile.ZipFile(output, 'w') as bundle:
            bundle.writestr('entry.js', 'different build')
        output.seek(0)
        with zipfile.ZipFile(output) as bundle, patch.object(analysis, 'ANCHORS', {'entry.js': {'send': 'required()'}}):
            with self.assertRaisesRegex(ValueError, 'missing build-specific anchor'):
                analysis.inspect(bundle)


if __name__ == '__main__':
    unittest.main()
