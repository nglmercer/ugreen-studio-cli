//! Korean catalog. Starts from English; untranslated strings
//! stay in English. `{}` placeholders keep the same order.
use super::L;

pub fn catalog() -> L {
    let mut l = super::en::catalog();
    l.app_title = " UGREEN / Studio Pro 제어 ";
    l.connected = "연결됨";
    l.disconnected = "연결 안 됨";
    l.unknown = "알 수 없음";
    l.target = "대상: ";
    l.target_none = "선택 안 됨";
    l.protocol = "프로토콜: ";
    l.protocol_on = "Studio Pro HP206 (사용자 선택)";
    l.protocol_off = "선택 안 됨; m로 Studio Pro HP206 확인";
    l.battery = "배터리: ";
    l.battery_pct = "{}%";
    l.unavailable = "사용 불가";
    l.codec_line = "코덱: 감지되지 않음";
    l.firmware = "펌웨어: ";
    l.snapshot = "읽기: ";
    l.snapshot_stale = "만료됨 / 최신 아님";
    l.snapshot_ago = "{}초 전에 읽음";
    l.snapshot_never = "읽지 않음";
    l.rfcomm_line = "RFCOMM {} | 작업당 타임아웃 {}초 | 소매 fw 0.2.5 확인됨";
    l.labels = [
        "ANC",
        "이퀄라이저",
        "게임 모드",
        "공간 오디오",
        "이중 연결",
        "바람 소음 감소",
        "알림음",
        "볼륨 높이기 동작",
        "볼륨 낮추기 동작",
    ];
    l.proposed = "  -> {} (제안)";
    l.settings_title = " 설정 / 기기 읽기 값, 자동 쓰기 없음 ";
    l.settings_blocked = " 설정 / 쓰기 잠김: r로 새로 고침 ";
    l.settings_stale = " 설정 / 마지막 읽기 값이 만료됨 ";
    l.status_title = " 상태 ";
    l.status_working = " 작업 중 / Esc가 이후 단계를 취소 ";
    l.log_title = " 세션 로그 / 위 아래 Home End / l Esc 닫기 ";
    l.shortcuts_title = " 단축키 / 위 아래 스크롤 / ? Esc 닫기 ";
    l.edit_title = " 대상 주소 편집 ";
    l.edit_lines = &[
        "16진수와 콜론을 입력하세요. Backspace로 삭제; Ctrl-U로 지우기.",
        "Enter는 로컬에만 저장; Esc로 취소. 연결은 시작되지 않습니다.",
    ];
    l.model_title = " 모델 전용 프로토콜 확인 ";
    l.model_lines = &[
        "UGREEN Studio Pro HP206에서만 사용하세요.",
        "소매 펌웨어 0.2.5에서 검증됨(1대); 다른",
        "변형과 펌웨어는 검증되지 않았습니다.",
        "HiTune Max5c는 명령 ID가 충돌하여 지원되지 않습니다.",
        "이는 프로토콜 선택이며 기기 신원을 증명하지 않습니다.",
        "y / Enter: Studio Pro HP206 선택    n / Esc: 취소",
    ];
    l.paired_title = " 페어링된 캐시 / 위 아래 Enter 선택 / Esc 닫기 ";
    l.paired_empty = "캐시에 페어링된 기기가 없습니다. Esc로 닫습니다.";
    l.quit_title = " 작업 중에 종료할까요? ";
    l.quit_lines = &[
        "취소를 요청했습니다. 이후 단계는 건너뜁니다.",
        "진행 중인 쓰기가 완료될 수 있습니다.",
        "종료는 worker를 기다리지 않습니다.",
        "q / Enter: 지금 종료    Esc: 남아서 결과 대기",
    ];
    l.resize_to = "최소 {}x{}로 크기를 조절하세요";
    l.resize_quit = "Esc: 취소  q: 종료";
    l.resize_inflight = "진행 중인 쓰기가 완료될 수 있습니다. q/Enter: 종료; Esc: 남기";
    l.resize_disabled = "이 크기에서는 동작을 비활성화했습니다.";
    l.footer = [
        [
            ("a", "주소"),
            ("p", "기기"),
            ("m", "모델"),
            ("c", "연결"),
            ("r", "새로고침"),
            ("d", "끊기"),
        ],
        [
            ("<", "적게"),
            (">", "많이"),
            ("Enter", "적용"),
            ("y", "확인"),
            ("Esc", "뒤로"),
            ("", ""),
        ],
        [
            ("l", "로그"),
            ("?", "단축키"),
            ("L", "언어"),
            ("q", "종료"),
            ("", ""),
            ("", ""),
        ],
    ];
    l.status_startup =
        "연결 안 됨. 아직 Bluetooth 접근이 없습니다. 주소를 편집(a)하거나 페어링된 캐시를 읽으세요(p).";
    l.status_loading = "캐시에서 페어링된 기기를 불러오는 중(검색 없음)...";
    l.status_connecting = "연결하는 중, 그다음 상태와 펌웨어를 읽는 중...";
    l.status_refreshing = "상태와 펌웨어를 새로 고치는 중...";
    l.status_disconnecting = "제어 연결을 끊는 중...";
    l.status_applying = "프리플라이트 후 {}={}를 적용하고 확인과 읽기를 검증하는 중...";
    l.status_paired_empty =
        "캐시에 페어링된 기기가 없습니다. 먼저 시스템 Bluetooth 설정에서 페어링하세요.";
    l.status_paired_pick =
        "원하는 기기를 선택하고 Enter를 누르세요. 선택만으로는 연결되지 않습니다.";
    l.status_read_ok = "상태를 읽었습니다. 표시되는 설정은 기기 읽기 값입니다.";
    l.status_firmware_note = "설정은 읽었으나 펌웨어를 사용할 수 없음: {}";
    l.status_blocked_note = " 쓰기 잠금 해제에는 명시적 새로 고침(r)이 여전히 필요합니다.";
    l.status_verified = "확인과 일치하는 기기 읽기로 {}={}을(를) 검증했습니다.";
    l.status_disconnected = "제어 연결이 끊겼습니다. Bluetooth는 변경되지 않았습니다.";
    l.status_quit_confirm = "취소를 요청했습니다. 진행 중인 쓰기가 완료될 수 있습니다. 기다리지 않고 종료하려면 q 또는 Enter; 남으려면 Esc.";
    l.status_cancel_busy = "취소를 요청했습니다; 현재 단계를 기다리는 중. 진행 중인 쓰기가 완료될 수 있습니다. 이후 단계는 건너뜁니다.";
    l.status_address_saved = "주소를 로컬에 저장했습니다. 명시적으로 연결하려면 c.";
    l.status_bad_address = "콜론으로 구분된 16진수 옥텟 6개를 입력하세요. 예: AA:BB:CC:DD:EE:FF.";
    l.status_model_on =
        "Studio Pro HP206 프로토콜을 사용자가 선택했습니다. 기기 신원은 검증하지 않았습니다.";
    l.status_device_picked =
        "주소를 선택했습니다. 연결하려면 Studio Pro 프로토콜을 확인하세요(y 또는 Enter).";
    l.status_device_bad = "캐시의 기기 주소가 올바르지 않습니다. 직접 편집하세요.";
    l.status_cancel_then_disconnect =
        "취소를 요청했습니다; 끊기는 현재 단계 뒤에 이어집니다. 진행 중인 쓰기가 완료될 수 있습니다.";
    l.status_must_disconnect = "대상이나 프로토콜을 바꾸기 전에 끊으세요(d).";
    l.status_must_connect = "연결 안 됨; 새로 고치기 전에 연결하세요(c).";
    l.status_proposed = "제안된 값일 뿐입니다. Enter로 적용; Esc로 버림.";
    l.status_unavailable_setting = "이 설정은 기기 읽기 값에 없습니다. 쓰기가 비활성화되었습니다.";
    l.status_already_matches = "그 값은 이미 마지막 읽기 값과 같습니다. 쓰기가 필요 없습니다.";
    l.status_pick_first = "적용 전에 왼/오른쪽으로 제안된 값을 고르세요.";
    l.status_disabled =
        "변경이 비활성화되었습니다. 연결하고 불확실한 작업 후에는 명시적으로 새로 고치세요.";
    l.status_lang = "언어: {}. L로 순환 전환.";
    l.err_gone = "연결 안 됨; 먼저 연결하세요.";
    l.err_status_query = "상태 쿼리 실패: {}";
    l.err_paired_list = "페어링된 캐시 읽기 실패: {}";
    l.err_need_model = "연결 전에 Studio Pro HP206 프로토콜을 확인하세요.";
    l.err_connected_already = "다른 대상을 연결하기 전에 끊으세요.";
    l.err_connect = "연결 실패: {}";
    l.err_uncertain_blocked = "불확실한 작업 후 쓰기가 잠겼습니다. 먼저 명시적으로 새로 고치세요.";
    l.err_preflight = "프리플라이트 상태 실패; 설정을 보내지 않았습니다: {}";
    l.err_preflight_unreadable =
        "프리플라이트가 {}을(를) 읽을 수 없음; 아무것도 보내지 않았습니다.";
    l.err_ack =
        "설정을 보냈을 수 있지만 확인이 실패했습니다: {}. 다시 쓰기 전에 명시적으로 새로 고치세요.";
    l.err_readback =
        "쓰기는 확인됐지만 읽기가 실패했습니다: {}. 다시 쓰기 전에 명시적으로 새로 고치세요.";
    l.err_readback_noack =
        "쓰기를 보냈지만 읽기가 실패했습니다: {}. 다시 쓰기 전에 명시적으로 새로 고치세요.";
    l.err_mismatch = "쓰기는 확인됐지만 {}이(가) {}과(와) 일치하지 않습니다. 다시 쓰기 전에 명시적으로 새로 고치세요.";
    l.err_mismatch_noack = "쓰기를 보냈지만 {}이(가) {}과(와) 일치하지 않습니다. 다시 쓰기 전에 명시적으로 새로 고치세요.";
    l.err_cancel_uncertain = "이후 단계를 취소했습니다. 진행 중인 쓰기가 완료됐을 수 있습니다; 다시 쓰기 전에 명시적으로 새로 고치세요.";
    l.err_cancel = "취소했습니다; 이후 단계는 시작하지 않습니다.";
    l.cli_help = "ugreen 0.1.0 — 비공식 UGREEN Studio Pro Bluetooth CLI\n\n사용법\n  ugreen [옵션] 명령\n\n오프라인 명령\n  help                          이 도움말 표시\n  tui                           선택적 터미널 UI 열기\n  models                        프로토콜 호환성과 제한 표시\n  commands                      지원되는 설정 나열\n  decode HEX                    캡처한 응답 바이트를 오프라인으로 검증/복호화\n  profile example               설정 프로필 예시 표시\n\nBluetooth 명령(먼저 시스템 설정에서 페어링)\n  discover                      캐시의 페어링된 기기 나열; 무선 검색 없음\n  capture                       세션의 TX/RX 원시 바이트를 피처 16진수로 기록\n  status                        배터리, 펌웨어, 설정 읽기\n  set 키 값                   설정 하나를 변경하고 읽기 검증\n  profile export                설정을 읽어 재사용 가능한 프로필 출력\n  profile apply 파일          프로필 검증, 적용, 각 설정 검증\n\n옵션(명령 앞)\n  --address XX:XX:XX:XX:XX:XX    명시적인 대상 Bluetooth 주소\n  --model studio-pro            하드웨어 요청에 필수; 자동 감지 안 함\n  --channel 1                   RFCOMM 채널, 1–30(기본 1)\n  --timeout 3                   작업당 타임아웃 초, 1–60\n  --lang ko                     인터페이스 언어: en, es, pt, de, fr, it, nl, ru, zh, ja, ko(기본 en)\n  --autoconnect                 시작 시 마지막 기기에 연결(기본값 활성화)\n  --no-autoconnect              시작 시 자동 연결 비활성화\n  --dry-run                     Bluetooth 접근 없이 설정 패킷 표시\n  --help, -h                    도움말 표시\n  --version, -V                 버전 표시\n\n예시\n  ugreen discover\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro status\n  ugreen --dry-run set anc ultra\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro set eq bass\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro profile export > 프로필.conf\n  ugreen --dry-run profile apply 프로필.conf\n\nStudio Pro HP206 프로토콜은 소매 펌웨어 0.2.5에서 검증됨(1대);\nHiTune Max5c는 명령 ID가 충돌합니다. 펌웨어, 리셋, 직접 쓰기,\n헤드폰 찾기 기능은 제공하지 않습니다. 프로필은 실패 시 일부만 적용될 수 있습니다.\n";
    l.cli_models = "studio-pro: UGREEN Studio Pro HP206 참조 코드와 캡처에 기반한 프로토콜\n하드웨어 테스트 상태: 소매 펌웨어 0.2.5에서 검증됨(Linux, 채널 1; docs/verification.md 참조)\nHiTune Max5c: 지원 안 함(명령 ID 충돌)\n다른 UGREEN 모델/펌웨어: 미검증";
    l.cli_commands = "anc: off|ultra|general|gentle|adaptive|ambient\neq: classic|jazz|electronic|pop|classical|rock|bass|treble\ngame, spatial, dual, wind: on|off\nprompts: voice|beeps\nvolume-up-action, volume-down-action: none|next|previous";
    l.cli_banner = "Studio Pro HP206 프로토콜을 {}에서 사용, RFCOMM 채널 {}";
    l.cli_need_model =
        "이 모델 전용 프로토콜을 명시적으로 선택하려면 --model studio-pro를 전달하세요";
    l.cli_need_address = "--address가 필요합니다; 페어링된 기기 나열은 discover를 사용하세요";
    l.cli_connect_fail = "Bluetooth 연결 실패: {}. 페어링, 전원, 어댑터 지원, 다른 앱이 제어 채널을 점유하지 않는지 확인하세요";
    l.cli_preflight_fail = "프리플라이트 상태 실패; 설정을 보내지 않았습니다: {}";
    l.cli_verified = "검증됨 {}={}";
    l.cli_apply_fail =
        "{}={}에서 실패하기 전에 총 {}개 중 {}개 설정 검증: {}. 이전 설정은 롤백되지 않습니다";
    l.cli_model_note = "model=studio-pro(사용자 선택; 기기 신원 검증이 아님)";
    l.cli_unknown_value = "알 수 없음";
    l.cli_firmware_fail = "설정은 읽었으나 펌웨어 쿼리 실패: {}";
    l.cli_no_paired =
        "페어링된 기기가 반환되지 않았습니다. 먼저 시스템 Bluetooth 설정에서 헤드폰을 페어링하세요.";
    l.cli_paired_fail = "페어링된 기기 나열 실패: {}";
    l.cli_dry_run_writes = "실행 연습: Bluetooth 연결도 기기 쓰기도 없음";
    l.cli_bad_model = "--model studio-pro만 구현됨; Max5c ID는 호환 안 됨";
    l.cli_bad_option = "알 수 없는 옵션 '{}'";
    l.cli_bad_channel = "채널은 1–30입니다";
    l.cli_bad_timeout = "타임아웃은 1–60초입니다";
    l.cli_need_value = "{}에 값이 필요합니다";
    l.cli_unknown_command = "알 수 없는 명령 '{}'; ugreen --help를 실행하세요";
    l.cli_bad_usage = "사용법: {}";
    l.cli_bad_decode = "캡처가 깨끗하고 완전한 응답 스트림이 아닙니다: {} 프레임, {} 거부된 CRC, {} 버린 바이트, {} 알 수 없는 프레임, {} 남은 바이트";
    l.cli_no_tty =
        "TUI는 대화형 stdin과 stdout 터미널이 필요합니다; CLI 명령은 ugreen --help를 실행하세요.";
    l.cli_no_tui_feature =
        "이 CLI 전용 빌드에서는 TUI가 비활성화되어 있습니다; --no-default-features 없이 다시 빌드하세요";
    l.cli_tui_dry =
        "tui는 --dry-run과 조합할 수 없습니다; 오프라인 패킷은 --dry-run set 키 값을 사용하세요";
    l.cli_bad_profile = "프로필이 16 KiB를 초과합니다";
    l.cli_inflight_after_quit = "진행 중인 쓰기가 완료됐을 수 있습니다. 더 변경하기 전에 기기 상태를 명시적으로 새로 고치세요.";
    l.set_bad_anc = "올바르지 않은 ANC 값";
    l.set_bad_eq = "올바르지 않은 EQ 값";
    l.set_bad_onoff = "on 또는 off가 필요합니다";
    l.set_bad_prompts = "voice 또는 beeps가 필요합니다";
    l.set_bad_button = "none, next, previous 중 하나가 필요합니다";
    l.set_unknown = "알 수 없는 설정 '{}'\n{}";
    l.profile_big = "프로필이 16 KiB를 초과합니다";
    l.profile_line = "프로필 {}번째 줄은 키=값이어야 합니다";
    l.profile_dup = "중복된 프로필 키 '{}'";
    l.profile_model = "프로필의 모델은 studio-pro여야 합니다";
    l.profile_need_model = "프로필에 model=studio-pro가 필요합니다";
    l.profile_empty = "프로필에 설정이 없습니다";
    l.req_timeout = "기한까지 일치하는 유효한 응답이 없습니다";
    l.req_closed = "Bluetooth 연결이 닫혔습니다";
    l.req_rejected = "헤드폰이 명령 0x{:02X}를 거부했습니다";
    l.info_short = "기기 정보 페이로드가 너무 짧습니다: {}바이트(최소 8 필요)";
    l.fw_short = "펌웨어 응답에 버전 바이트가 없습니다";
    l.set_ackfail =
        "설정을 보냈을 수 있지만 확인이 실패했습니다: {}; 재시도 전에 상태를 조회하세요";
    l.set_rbfail = "쓰기는 확인됐지만 읽기가 실패했습니다: {}";
    l.set_rbfail_noack = "쓰기를 보냈지만 읽기가 실패했습니다: {}";
    l.set_mismatch = "쓰기는 확인됐지만 {}의 읽기가 {}과(와) 일치하지 않습니다; 상태를 조회하세요";
    l.set_mismatch_noack =
        "쓰기를 보냈지만 {}의 읽기가 {}과(와) 일치하지 않습니다; 상태를 조회하세요";
    l
}
