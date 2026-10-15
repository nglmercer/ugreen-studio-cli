//! Korean catalog. Starts from English; untranslated strings
//! stay in English. `{}` placeholders keep the same order.
use super::L;

pub fn catalog() -> L {
    let mut l = super::en::catalog();
    l.cli_json_unsupported = "이 명령은 --json을 지원하지 않습니다.";
    l.status_applying_noack =
        "사전 확인 후 {}={} 적용, 다시 읽은 값 확인 중(공간 오디오는 확인 응답 없음)...";
    l.status_verified_noack =
        "다시 읽은 값이 일치하여 {}={} 검증됨. 공간 오디오는 확인 응답을 보내지 않습니다.";
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
    l.cli_models = "studio-pro: UGREEN Studio Pro HP206 참조 코드와 캡처에 기반한 프로토콜\n하드웨어 테스트 상태: 소매 펌웨어 0.2.5에서 검증됨(Linux, 채널 1; docs/verification.md 참조)\nHiTune Max5c: 지원 안 함(명령 ID 충돌)\n다른 UGREEN 모델/펌웨어: 미검증";
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
    // 생성된 도움말 조각; 화면 자체는
    // 명령 레지스트리에서 렌더링됩니다.
    l.help_subtitle = "UGREEN Studio Pro 비공식 Bluetooth CLI";
    l.help_usage = "사용법";
    l.help_offline = "오프라인 명령";
    l.help_bluetooth = "BLUETOOTH 명령 (먼저 시스템 설정에서 페어링)";
    l.help_capture_gated = "캡처 보호 쿼리 (연결 전에 거부됩니다)";
    l.help_options = "옵션 (명령 앞)";
    l.help_examples = "예시";
    l.help_footer = "Studio Pro HP206 프로토콜은 retail 펌웨어 0.2.5에서 검증됨(1대);\nHiTune Max5c는 명령 ID가 충돌합니다. 펌웨어, 초기화, 원시 쓰기, 헤드폰 찾기\n작업은 제공되지 않습니다. 프로필은 실패 시 일부만 적용될 수 있습니다.";
    l.opt_address = "명시적 대상 Bluetooth 주소";
    l.opt_model = "하드웨어 요청에 필수; 자동 감지하지 않음";
    l.opt_channel = "RFCOMM 채널, 1–30 (기본 1)";
    l.opt_timeout = "작업별 시간 제한, 초, 1–60";
    l.opt_lang = "인터페이스 언어: en, es, pt, de, fr, it, nl, ru, zh, ja, ko (기본 en)";
    l.opt_autoconnect = "시작 시 마지막 기기에 연결 (기본 켜짐)";
    l.opt_no_autoconnect = "시작 시 자동 연결 비활성화";
    l.opt_json = "지원되는 읽기 전용 명령을 버전이 있는 JSON으로 출력";
    l.opt_dry_run = "Bluetooth 접근 없이 설정 패킷 출력";
    l.opt_help = "도움말 표시";
    l.opt_version = "버전 표시";
    l.cmd_help = "이 도움말 표시";
    l.cmd_tui = "선택적 대화형 터미널 화면 열기";
    l.cmd_models = "프로토콜 호환성 및 제한 표시";
    l.cmd_commands = "지원되는 설정 나열";
    l.cmd_decode = "캡처한 응답 바이트를 오프라인에서 검증/디코딩";
    l.cmd_analyze = "APK 정적 분석 결과를 오프라인에서 해석";
    l.cmd_candidate = "정적 패킷 후보 출력; 전송되지 않음";
    l.cmd_profile = "설정 프로필 관리";
    l.cmd_profile_example = "설정 프로필 예시 출력";
    l.cmd_profile_export = "설정을 읽어 재사용 가능한 프로필 출력";
    l.cmd_profile_apply = "프로필 검증 후 적용하고 각 설정 검증";
    l.cmd_discover = "캐시된 페어링된 기기 나열 (무선 스캔 없음)";
    l.cmd_capture = "세션의 원시 TX/RX 바이트를 픽스처 hex로 기록";
    l.cmd_status = "배터리, 펌웨어, 설정 읽기";
    l.cmd_set = "설정 변경 후 읽기로 검증";
    l.cmd_query = "캡처 보호 쿼리; 이 펌웨어에서 미검증";
    l.cmd_control = "캡처 보호 제어; --dry-run이 바이트 출력";
    l.cmd_find = "캡처 보호 찾음 음";
    l.navigation = ["홈", "오디오", "장치", "설정", "진단", "연구"];
    l.headset_connections = ["헤드셋 연결", "헤드셋에 연결된 장치이며 이 PC의 페어링 목록과 다릅니다.", "HP206 조회(0x0D)는 해당 펌웨어의 공식 앱 캡처가 필요합니다. 조회를 전송하지 않았으며 장치 수와 정보는 알 수 없습니다.", "원격 장치 연결 해제, 재연결 및 전환은 지원하지 않습니다. 호스트 Bluetooth 작업은 이 PC에만 적용됩니다.", "r: 지원 확인 · d: 이중 연결 · Esc: 뒤로"];
    l.headset_connections[4] = "r: 이중 연결 상태 새로 고침 · d: 이중 연결 · Esc: 뒤로";
    l.page_labels = [
        "장치",
        "오디오",
        "헤드셋 설정",
        "멀티포인트",
        "실험적 기능",
        "진단",
    ];
    l.page_hint = "↑/↓ Enter: 열기 · r: 새로고침 · Esc: 뒤로";
    l.host_commands = [
        "OS 오디오 전송 상태 보기",
        "호스트 오디오 출력 목록",
        "기능 검증 근거 보기",
        "호스트 진단 읽기",
        "호스트 Bluetooth 연결",
        "호스트 Bluetooth 연결 해제",
    ];
    l.cmd_audio_devices = "Windows 웨이브출력 장치 나열";
    l.features_title = " 기능 ";
    l.features_search = "검색: ";
    l.features_all = "전체";
    l.feature_categories = [
        "오디오",
        "연결",
        "환경",
        "피드백",
        "기기 정보",
        "멀티포인트",
        "코덱",
        "고급 오디오",
        "제어",
        "오디오 공유",
        "찾기",
        "기기 관리",
        "진단",
    ];
    l.feature_names = [
        "ANC",
        "이퀄라이저",
        "게임 모드",
        "공간 오디오",
        "이중 연결",
        "바람 소음 저감",
        "음성 안내",
        "음량 올림 길게 누름 동작",
        "음량 내림 길게 누름 동작",
        "멀티포인트 기기",
        "오디오 코덱",
        "헤드폰 ID",
        "헤드폰 색상",
        "케이스 MAC",
        "LDAC 제어",
        "한 기기에 LDAC",
        "고품질 디코딩",
        "음량 명령",
        "모노/스테레오 채널",
        "버튼 매핑",
        "AI 동작",
        "AI 제어",
        "오디오 공유 끄기",
        "한 기기와 공유",
        "선택한 기기와 공유",
        "왼쪽 이어폰 찾기",
        "오른쪽 이어폰 찾기",
        "공장 초기화",
        "캡처 디코더",
        "캡처 분석",
        "패킷 미리보기",
    ];
    l.feature_descriptions = [
        "능동 소음 제거 깊이. 검증된 설정입니다. 설정 화면에서 편집하세요.",
        "이퀄라이저 프리셋. 검증된 설정입니다. 설정 화면에서 편집하세요.",
        "게임 모드 저지연 스위치. 검증된 설정입니다. 설정 화면에서 편집하세요.",
        "공간 오디오 스위치. 검증된 설정입니다. 설정 화면에서 편집하세요.",
        "두 기기 연결 스위치. 검증된 설정입니다. 설정 화면에서 편집하세요.",
        "바람 소음 저감 스위치. 검증된 설정입니다. 설정 화면에서 편집하세요.",
        "음성 안내 스타일. 검증된 설정입니다. 설정 화면에서 편집하세요.",
        "음량 올림 길게 누름 동작. 검증된 설정입니다. 설정 화면에서 편집하세요.",
        "음량 내림 길게 누름 동작. 검증된 설정입니다. 설정 화면에서 편집하세요.",
        "연결된 멀티포인트 기기 목록을 읽습니다. 피어 프로토콜은 아직 하드웨어 검증이 필요합니다.",
        "현재 Bluetooth 오디오 코덱을 읽습니다. 조회 형식은 아직 하드웨어 검증이 필요합니다.",
        "헤드폰 하드웨어 ID를 읽습니다. 조회 형식은 아직 하드웨어 검증이 필요합니다.",
        "헤드폰 색상을 읽습니다. 조회 형식은 아직 하드웨어 검증이 필요합니다.",
        "충전 케이스 MAC 주소를 읽습니다. 조회 형식은 아직 하드웨어 검증이 필요합니다.",
        "현재 연결에서 LDAC 인코딩을 전환합니다. 제어 형식은 아직 하드웨어 검증이 필요합니다.",
        "페어링된 한 기기에서 LDAC 인코딩을 전환합니다. 제어 형식은 아직 하드웨어 검증이 필요합니다.",
        "고품질 오디오 디코딩을 전환합니다. 제어 형식은 아직 하드웨어 검증이 필요합니다.",
        "절대 음량 바이트를 보냅니다. 하드웨어 범위는 검증되지 않았습니다.",
        "오디오 채널을 모노와 스테레오 사이로 전환합니다. 제어 형식은 아직 하드웨어 검증이 필요합니다.",
        "여덟 개의 길게 누름 동작을 재지정합니다. 제어 형식은 아직 하드웨어 검증이 필요합니다.",
        "AI가 실행하는 네 개의 동작을 재지정합니다. 제어 형식은 아직 하드웨어 검증이 필요합니다.",
        "AI 오디오 제어를 전환합니다. 제어 형식은 아직 하드웨어 검증이 필요합니다.",
        "오디오 공유를 끕니다. 제어 형식은 아직 하드웨어 검증이 필요합니다.",
        "유일하게 연결된 기기와 오디오를 공유합니다. 제어 형식은 아직 하드웨어 검증이 필요합니다.",
        "선택한 기기와 오디오를 공유합니다. 제어 형식은 아직 하드웨어 검증이 필요합니다.",
        "호스트를 통해 왼쪽 이어폰에서 위치 확인음을 재생합니다. 찾기 프로토콜은 아직 하드웨어 검증이 필요합니다.",
        "호스트를 통해 오른쪽 이어폰에서 위치 확인음을 재생합니다. 찾기 프로토콜은 아직 하드웨어 검증이 필요합니다.",
        "페어링을 해제하고 공장 초기값으로 복원합니다. 파괴적입니다. 제어 형식은 아직 하드웨어 검증이 필요합니다.",
        "캡처한 응답 바이트를 오프라인에서 디코딩합니다. Bluetooth 접근 없이 동작합니다.",
        "완전한 캡처 스트림을 오프라인에서 해석합니다. Bluetooth 접근 없이 동작합니다.",
        "제어가 보낼 정적 패킷을 출력합니다. 전송되지 않습니다.",
    ];
    l.feature_available = "사용 가능";
    l.feature_pending = "검증 대기";
    l.feature_pending_reason = "선로 형식은 공식 앱의 하드웨어 검증이 아직 필요합니다.";
    l.feature_live_disabled = "기기에서 검증된 증거가 있을 때까지 라이브 실행이 비활성화됩니다.";
    l.feature_offline_workflow = "오프라인 분석 전용입니다. 헤드폰에는 아무것도 전송하지 않습니다.";
    l.feature_offline_hint = "실행: ugreen {}";
    l.feature_kind_setting = "검증된 설정";
    l.feature_kind_query = "기기 조회";
    l.feature_kind_control = "기기 제어";
    l.feature_kind_offline = "오프라인 분석";
    l.feature_category_label = "분류: {}";
    l.feature_current = "현재: {}";
    l.feature_edit_in_settings = "설정에서 편집 (s)";
    l.feature_run_query = "Enter: 조회 실행";
    l.feature_configure = "Enter: 설정";
    l.feature_back = "Esc: 뒤로";
    l.feature_form_title = " {} 설정 ";
    l.feature_err_required = "검토 전에 모든 필드를 설정하세요";
    l.feature_field_unset = "설정되지 않음";
    l.feature_form_hint = "위 아래 항목 / 좌우 선택 / Enter 확인 / Esc 뒤로";
    l.feature_review_title = " {} 확인 ";
    l.review_write_title = " 쓰기 확인 ";
    l.feature_review_target = "대상: {}";
    l.feature_review_previous = "현재: {}";
    l.feature_review_proposed = "제안: {}";
    l.feature_review_values = "값: {}";
    l.feature_review_readback = "읽기: {}";
    l.feature_readback_verified = "확인 응답과 일치하는 읽기로 검증됨";
    l.feature_readback_noack = "일치하는 읽기로 검증됨(공간 오디오는 확인 응답을 보내지 않음)";
    l.feature_readback_unconfirmed = "독립적으로 확인되지 않았습니다. 적용한 후 새로고침하세요.";
    l.feature_readback_query = "읽기 전용 조회";
    l.feature_confirm_apply = "Enter: 적용    Esc: 취소";
    l.feature_confirm_reset = "파괴적: 호스트 페어링을 제거하고 공장 초기값으로 복원합니다.";
    l.features_footer = [
        [
            ("위", "이동"),
            ("아래", "이동"),
            ("Tab", "분류"),
            ("Enter", "열기"),
            ("Esc", "뒤로"),
            ("", ""),
        ],
        [
            ("글자", "검색"),
            ("백스페이스", "삭제"),
            ("Ctrl+U", "지우기"),
            ("", ""),
            ("", ""),
            ("", ""),
        ],
        [
            ("Esc", "닫기"),
            ("?", "단축키"),
            ("", ""),
            ("", ""),
            ("", ""),
            ("", ""),
        ],
    ];
    l.status_feature_running = "기능 작업을 실행하는 중...";
    l.status_edit_in_settings = "{}는 검증된 설정입니다. 설정 화면에서 편집하세요 (F4).";
    l.codec_unverified = "조회 미검증";
    l.arg_enabled = "활성화";
    l.arg_side = "쪽";
    l.arg_duration = "시간(초)";
    l.arg_target = "대상";
    l.arg_value = "값(0-255)";
    l.arg_mode = "모드";
    l.arg_peer = "페어링됨";
    l.arg_actions = "동작";
    l.arg_address = "페어링된 기기 MAC";
    l.arg_hint_toggle = "on 또는 off";
    l.arg_hint_side = "왼쪽 또는 오른쪽";
    l.arg_hint_seconds = "1..30초";
    l.arg_hint_byte = "0..255";
    l.arg_hint_mac = "콜론으로 구분된 16진수 옥텟 6개";
    l.arg_hint_peer = "1 또는 2";
    l.arg_hint_actions = "콤마로 구분된 동작 값 {}개";
    l.tui_no_tty = "이 화면은 대화형 stdin과 stdout 터미널을 필요로 합니다. CLI 명령은 ugreen --help를 실행하세요.";
    l.tui_bad_channel = "화면 채널은 1-30이어야 하고 타임아웃은 0 초과 최대 60초여야 합니다.";
    l.status_worker_busy = "이미 작업이 실행 중입니다. 대기하거나 Esc로 취소하세요.";
    l.status_worker_sequence = "요청 시퀀스가 소진되었습니다. 화면을 다시 시작하세요.";
    l.status_worker_stopped = "Bluetooth 워커가 중지되었습니다. 화면을 다시 시작하세요.";
    l.status_worker_queue = "워커 큐가 가득 찼습니다. 잠시 후 다시 시도하세요.";
    l.status_reset_unavailable = "초기화 대상을 사용할 수 없습니다";
    l.status_reset_done = "초기화가 확인되었습니다. 호스트 페어링과 저장된 기기를 제거했습니다.";
    l.status_inflight_write =
        "진행 중인 쓰기가 완료되었을 수 있습니다. 다시 시작한 후 새로고침하세요.";
    l
}
