"""Offline tests for the opt-in hardware runner; no Bluetooth access."""
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import hardware_validate as runner


class HardwareValidationTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='ugreen-hil-test-')
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        binary = self.root / 'ugreen'
        binary.write_text('offline mock binary\n', encoding='utf-8')
        self.args = runner.parser().parse_args([
            str(binary), '--live', '--address', 'AA:BB:CC:DD:EE:FF',
            '--model', 'studio-pro', '--firmware', '0.2.5',
            '--retail-label', 'HP206 test fixture', '--adapter', 'offline fake',
            '--output', str(self.root / 'evidence')])
        self.calls = []
        self.firmware = '0.2.5'
        self.decode = ('instruction=0x04 success=true crc=valid payload=00\n'
                       'instruction=0x01 success=true crc=valid payload=00\n')

    def invoke(self, binary, flags, command, directory, name, timeout):
        self.calls.append(command[0])
        if name == 'version':
            output, error = 'ugreen test\n', ''
        elif name == 'status':
            output, error = json.dumps({
                'schema_version': 1, 'kind': 'status', 'model': 'studio-pro',
                'firmware': self.firmware, 'address': self.args.address,
                'settings': {'dual': 'on'}, 'raw_device_info': '00'}), ''
        elif name == 'capture':
            output, error = '00\n', 'firmware=0.2.5\n'
        else:
            output, error = self.decode, ''
        (directory / (name + '.stdout')).write_text(output, encoding='utf-8')
        (directory / (name + '.stderr')).write_text(error, encoding='utf-8')
        return output, error

    def test_explicit_live_opt_in_precedes_all_execution(self):
        self.args.live = False
        with patch.object(runner, 'invoke') as invoke:
            with self.assertRaisesRegex(ValueError, '--live'):
                runner.collect(self.args)
            invoke.assert_not_called()
        self.assertFalse(self.args.output.exists())

    def test_valid_core_record_never_promotes_candidates(self):
        with patch.object(runner, 'invoke', self.invoke):
            record = runner.collect(self.args)
        self.assertEqual(record['outcome'], 'passed_core_reads_only')
        self.assertEqual(self.calls, ['--version', 'status', 'capture', 'decode'])
        self.assertFalse(record['capabilities_promoted'])
        self.assertFalse(record['candidate_operations_executed'])
        self.assertEqual(record['observed_firmware'], '0.2.5')
        self.assertEqual(len(record['artifacts_sha256']), 8)
        self.assertEqual(json.loads((self.args.output / 'record.json').read_text()), record)

    def test_firmware_mismatch_stops_before_capture_and_records_failure(self):
        self.firmware = '0.2.6'
        with patch.object(runner, 'invoke', self.invoke):
            record = runner.collect(self.args)
        self.assertEqual(self.calls, ['--version', 'status'])
        self.assertEqual(record['outcome'], 'failed')
        self.assertIn('firmware mismatch', record['error'])

    def test_missing_crc_valid_response_cannot_pass(self):
        self.decode = 'instruction=0x04 success=true crc=valid payload=00\n'
        with patch.object(runner, 'invoke', self.invoke):
            record = runner.collect(self.args)
        self.assertEqual(record['outcome'], 'failed')
        self.assertIn('0x01', record['error'])
        self.assertIn('capture.stdout', record['artifacts_sha256'])

    def test_existing_evidence_is_never_overwritten(self):
        self.args.output.mkdir()
        sentinel = self.args.output / 'record.json'
        sentinel.write_text('original', encoding='utf-8')
        with patch.object(runner, 'invoke') as invoke:
            with self.assertRaises(FileExistsError):
                runner.collect(self.args)
            invoke.assert_not_called()
        self.assertEqual(sentinel.read_text(), 'original')

    def test_invalid_address_does_not_create_record_or_open_transport(self):
        self.args.address = 'not a MAC address'
        with patch.object(runner, 'invoke') as invoke:
            with self.assertRaisesRegex(ValueError, 'address'):
                runner.collect(self.args)
            invoke.assert_not_called()
        self.assertFalse(self.args.output.exists())


if __name__ == '__main__':
    unittest.main()
