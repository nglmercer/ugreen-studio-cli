#!/usr/bin/env python3
"""Opt-in, read-only hardware validation. Never sends candidate or setting writes.

Records contain local device addresses. Review/redact before sharing them.
A successful core read never grants capabilities to unrelated instructions.
"""
import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import platform
import re
import subprocess
import sys

MAX_OUTPUT = 64 * 1024


def parser():
    result = argparse.ArgumentParser(description=__doc__)
    result.add_argument('binary', type=Path)
    result.add_argument('--live', action='store_true', required=True,
                        help='Explicitly permit read-only RFCOMM queries')
    result.add_argument('--address', required=True)
    result.add_argument('--model', choices=['studio-pro'], required=True)
    result.add_argument('--firmware', required=True, help='Expected exact firmware version')
    result.add_argument('--retail-label', required=True)
    result.add_argument('--adapter', required=True, help='Adapter model/driver identification')
    result.add_argument('--channel', type=int, default=1, choices=range(1, 31))
    result.add_argument('--timeout', type=int, default=3, choices=range(1, 61))
    result.add_argument('--output', type=Path, required=True, help='New directory; never overwrites')
    return result


def sha256(path):
    digest = hashlib.sha256()
    with path.open('rb') as source:
        for block in iter(lambda: source.read(65536), b''):
            digest.update(block)
    return digest.hexdigest()


def invoke(binary, flags, command, directory, name, timeout):
    """No shell interpolation. Preserve stdout/stderr even on a failed query."""
    out = directory / (name + '.stdout')
    err = directory / (name + '.stderr')
    with out.open('xb') as stdout, err.open('xb') as stderr:
        completed = subprocess.run([str(binary), *flags, *command], stdin=subprocess.DEVNULL,
                                   stdout=stdout, stderr=stderr, timeout=timeout, check=False)
    if out.stat().st_size > MAX_OUTPUT or err.stat().st_size > MAX_OUTPUT:
        raise ValueError(name + ': output exceeds validation bound')
    if completed.returncode:
        raise ValueError(name + ': command failed; inspect saved stdout/stderr')
    return out.read_text(encoding='utf-8'), err.read_text(encoding='utf-8')


def collect(args):
    if not args.live:
        raise ValueError('--live is required before any hardware work')
    if not re.fullmatch(r'(?:[0-9a-fA-F]{2}:){5}[0-9a-fA-F]{2}', args.address):
        raise ValueError('address must contain six colon-separated hexadecimal octets')
    if not re.fullmatch(r'\d+\.\d+\.\d+', args.firmware):
        raise ValueError('firmware must be an exact major.minor.patch version')
    binary = args.binary.resolve(strict=True)
    # Refuse an existing directory even when empty: previous evidence is immutable.
    args.output.mkdir(parents=True, exist_ok=False, mode=0o700)
    record = {
        'schema_version': 1, 'kind': 'hardware_core_read_validation',
        'started_at': datetime.now(timezone.utc).isoformat(),
        'platform': platform.platform(), 'adapter': args.adapter,
        'retail_label': args.retail_label, 'model': args.model,
        'expected_firmware': args.firmware, 'address': args.address.upper(),
        'rfcomm_channel': args.channel, 'query_timeout_seconds': args.timeout,
        'binary_sha256': sha256(binary), 'outcome': 'running',
        'operations': ['device_info_read', 'firmware_read'],
        'candidate_operations_executed': False, 'capabilities_promoted': False,
    }
    flags = ['--lang', 'en', '--address', args.address.upper(), '--model', args.model,
             '--channel', str(args.channel), '--timeout', str(args.timeout)]
    try:
        version, _ = invoke(binary, [], ['--version'], args.output, 'version', 10)
        record['binary_version'] = version.strip()
        output, _ = invoke(binary, [*flags, '--json'], ['status'], args.output,
                           'status', args.timeout * 8 + 10)
        status = json.loads(output)
        if (not isinstance(status, dict) or status.get('schema_version') != 1 or status.get('kind') != 'status'
                or status.get('model') != args.model
                or not isinstance(status.get('address'), str)
                or status['address'].upper() != args.address.upper()):
            raise ValueError('status identity/schema mismatch')
        record['observed_firmware'] = status.get('firmware')
        if status.get('firmware') != args.firmware:
            raise ValueError('firmware mismatch; no further device queries performed')
        if not isinstance(status.get('settings'), dict) or not status.get('raw_device_info'):
            raise ValueError('status lacks parsed settings/raw device information')
        capture, context = invoke(binary, flags, ['capture'], args.output,
                                  'capture', args.timeout * 8 + 10)
        versions = re.findall(r'^firmware=(.+)$', context, re.MULTILINE)
        if versions != [args.firmware]:
            raise ValueError('capture firmware missing or changed')
        decoded, _ = invoke(binary, ['--lang', 'en'], ['decode', capture],
                            args.output, 'decode', 10)
        for instruction in ['0x04', '0x01']:
            if not re.search(r'^instruction=' + instruction + r' success=true crc=valid ',
                             decoded, re.MULTILINE):
                raise ValueError('capture lacks a successful CRC-valid response for ' + instruction)
        record['outcome'] = 'passed_core_reads_only'
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        record['outcome'] = 'failed'
        record['error'] = str(error)
    finally:
        record['finished_at'] = datetime.now(timezone.utc).isoformat()
        record['artifacts_sha256'] = {
            path.name: sha256(path) for path in sorted(args.output.iterdir()) if path.is_file()
        }
        (args.output / 'record.json').write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
    return record


def main():
    args = parser().parse_args()
    try:
        record = collect(args)
    except (OSError, ValueError) as error:
        print(str(error), file=sys.stderr)
        return 1
    print(str(args.output / 'record.json'))
    return 0 if record['outcome'] == 'passed_core_reads_only' else 1


if __name__ == '__main__':
    sys.exit(main())
