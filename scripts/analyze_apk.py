#!/usr/bin/env python3
"""Reproduce static protocol evidence from the supplied UGREEN Audio APK.

Build-specific byte anchors are deliberate: another APK must be reviewed rather
than silently inheriting these findings. No APK code is executed or extracted.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct
import sys
import zipfile

APK_SHA256 = 'b2ed391d9c0a94769cb00a1f8a35a2bb8461b8d084721c66988ca011d1a2f45b'
BASE = 'assets/static/device/'
MAIN = BASE + 'assets/index-D9whH_hy.js'
DUAL = BASE + 'assets/double-link-B1wNDsyj.js'
CODEC = BASE + 'assets/acoustics-code-GbTtk_At.js'
SHARE = BASE + 'assets/audio-share-DSkfrSYZ.js'
FINDER = BASE + 'assets/headphone-finder-DBnHuGCY.js'

# Minimal source anchors, not redistributed application bundles. Offsets in the
# report are bytes in the uncompressed ZIP member, never Unicode character indices.
ANCHORS = {
    BASE + 'index.html': {
        'modern_entry': './assets/index-D9whH_hy.js',
    },
    MAIN: {
        'dual_route': './double-link-B1wNDsyj.js',
        'codec_route': './acoustics-code-GbTtk_At.js',
        'sharing_route': './audio-share-DSkfrSYZ.js',
        'finder_route': './headphone-finder-DBnHuGCY.js',
        'protocol_ids': 'const Qs={UGREEN_Universal:1,Zhongke_Lanxun:2}',
        'default_protocol': 'protocol_type:Qs.UGREEN_Universal',
        'config_merge': 'setConfig(e){this.config=Object.assign({},this.config,e)}',
        'studio_pro_config': 'case"UGREEN Studio Pro":',
        'dual_constants': 'eU={INSTRUCTION:6,OPEN:[1],CLOSE:[0]}',
        'peers_constant': 'rU={INSTRUCTION:13,CMD_DATA:[0]}',
        'codec_constant': 'yU={INSTRUCTION:15,CMD_DATA:[0]',
        'plain_ldac_constant': 'aU={INSTRUCTION:11,OPEN:[1],CLOSE:[0]}',
        'addressed_ldac_constant': 'gU={INSTRUCTION:30,OPEN:1,CLOSE:0}',
        'dual_export': 'eU as aE',
        'plain_ldac_export': 'aU as aV',
        'addressed_ldac_export': 'gU as aU',
        'crc_modbus': 's&1?s=s>>1^40961:s=s>>1',
        'frame_builder': 'buildPacket(e,s){const a=[170,187,204]',
        'peer_name_parser': 'new TextDecoder("utf-8").decode(q)',
        'peer_clear_parser': '_e===0&&',
        'notification_offset': 'parsePacketHardware(e){let s={};switch(e[4])',
        'master_slave_event': 'ot.emit("switchMasterSlave")',
        'volume_constant': 'cU={INSTRUCTION:16,CMD_DATA:0}',
        'in_box_constant': 'lU={INSTRUCTION:17,BACK_INSTRUCTION:23}',
        'volume_response_case': 'case lU.INSTRUCTION:s.state&&(s.sound_volume=e[6])',
        'in_box_response_case': 'case cU.INSTRUCTION:s.state&&(s.ear_Phone_in_box=e[6])',
        'lanxun_close': 'case S4:y=Af.length+7,r=[ui,Af.CMD,1,0,y,3,6,...a]',
    },
    DUAL: {
        'main_import': './index-D9whH_hy.js',
        'dual_import': 'aE as v',
        'peer_slot_selection': 'O=l.phoneBleName===e.deviceInfo.mobilePhoneName1?2:1',
        'targeted_close': 'new Uint8Array([...v.CLOSE,O,...d])',
        'targeted_send': 'm.$keyVoiceStatusChange(v.INSTRUCTION,b)',
        'lanxun_close_call': 'p(H,{value:b})',
        'peer_refresh_delay': 'await N(200)',
    },
    CODEC: {
        'main_import': './index-D9whH_hy.js',
        'addressed_import': 'aU as y',
        'plain_import': 'aV as E',
        'addressed_enable': 'new Uint8Array([y.OPEN,...t])',
        'addressed_disable': 'new Uint8Array([y.CLOSE,0,0,0,0,0,0])',
        'plain_enable': 'd.$keyVoiceStatusChange(E.INSTRUCTION,E.OPEN)',
        'retry_bounds': 've=3,he=2e3',
    },
    SHARE: {
        'targeted_sharing': 'new Uint8Array([l.OPEN,s,...a])',
        'single_peer_sharing': 'new Uint8Array([l.OPEN,255,255,255,255,255,255,255])',
        'local_wind_state': 't.setDeviceInfo({windNoise:!1})',
    },
    FINDER: {
        'finder_send': 'w.$keyVoiceStatusChange(f.INSTRUCTION',
    },
}


def android_metadata(data):
    """Read manifest/SDK attributes from binary Android XML, without Android tools."""
    if data[:4] != b'\x03\x00\x08\x00':
        raise ValueError('expected binary Android XML')
    strings = []
    result = {}
    offset = 8
    while offset < len(data):
        kind, header, size = struct.unpack_from('<HHI', data, offset)
        if size < header or header < 8 or offset + size > len(data):
            raise ValueError('invalid Android XML chunk')
        if kind == 1:
            count, _, flags, start, _ = struct.unpack_from('<IIIII', data, offset + 8)

            def length(at, utf8):
                value = data[at] if utf8 else struct.unpack_from('<H', data, at)[0]
                width, mask = (1, 0x80) if utf8 else (2, 0x8000)
                if value & mask:
                    following = data[at + width] if utf8 else struct.unpack_from('<H', data, at + width)[0]
                    return ((value & (mask - 1)) << (8 if utf8 else 16)) | following, at + 2 * width
                return value, at + width

            for index in range(count):
                relative = struct.unpack_from('<I', data, offset + header + index * 4)[0]
                at = offset + start + relative
                if flags & 0x100:
                    _, at = length(at, True)
                    count_bytes, at = length(at, True)
                    strings.append(data[at:at + count_bytes].decode('utf-8'))
                else:
                    count_chars, at = length(at, False)
                    strings.append(data[at:at + count_chars * 2].decode('utf-16le'))
        elif kind == 0x102:
            _, name, attr_start, attr_size, attr_count = struct.unpack_from('<IIHHH', data, offset + 16)
            element = strings[name]
            if element in ('manifest', 'uses-sdk'):
                attrs = {}
                for index in range(attr_count):
                    at = offset + 16 + attr_start + attr_size * index
                    _, key, raw, _, _, value_type, value = struct.unpack_from('<IIIHBBI', data, at)
                    attrs[strings[key]] = strings[raw] if raw != 0xffffffff else (strings[value] if value_type == 3 else value)
                result[element] = attrs
        offset += size
    return result


def inspect(bundle):
    sources = {}
    for name, anchors in ANCHORS.items():
        data = bundle.read(name)
        found = {}
        for label, text in anchors.items():
            needle = text.encode('utf-8')
            offset = data.find(needle)
            if offset < 0:
                raise ValueError(f'missing build-specific anchor {name}: {label}')
            found[label] = {'byte_offset': offset, 'needle': text,
                            'occurrences': data.count(needle)}
        sources[name] = {'size_bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest(),
                         'anchors': found}
    main = bundle.read(MAIN).decode('utf-8')
    begin = main.index('case"UGREEN Studio Pro":')
    config = main[begin:main.index('break;', begin)]
    flags = {}
    for key in ('is_headphone_finder_enabled', 'is_high_quality_decoding_enabled',
                'is_dual_device_pairing_enabled', 'is_audio_sharing_available',
                'is_ldac_mac_address', 'protocol_type'):
        match = re.search(r'\b' + key + r':([^,}]+)', config)
        flags[key] = match[1] if match else 'not_overridden_in_local_model_config'
    names = bundle.namelist()
    return {'schema_version': 1, 'kind': 'apk_static_analysis',
            'hardware_validation': False, 'capabilities_promoted': False,
            'android_metadata': android_metadata(bundle.read('AndroidManifest.xml')),
            'studio_pro_local_config': flags, 'sources': sources,
            'dex_sha256': {name: hashlib.sha256(bundle.read(name)).hexdigest()
                           for name in names if name.endswith('.dex')},
            'native_libraries': sorted(name for name in names if name.startswith('lib/') and name.endswith('.so'))}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('apk', type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    try:
        with args.apk.open('rb') as source:
            sha = hashlib.file_digest(source, 'sha256').hexdigest()
        if sha != APK_SHA256:
            raise ValueError('APK SHA-256 differs from the reviewed build')
        with zipfile.ZipFile(args.apk) as bundle:
            result = inspect(bundle)
        result['apk_sha256'] = sha
        result['apk_size_bytes'] = args.apk.stat().st_size
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(result, indent=2, sort_keys=True) + '\n', encoding='utf-8')
    except (OSError, ValueError, KeyError, IndexError, struct.error, zipfile.BadZipFile) as error:
        print(str(error), file=sys.stderr)
        return 1
    print(f'Wrote static evidence for {len(result["sources"])} APK members to {args.output}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
