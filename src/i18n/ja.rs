//! Japanese catalog. Starts from English; untranslated strings
//! stay in English. `{}` placeholders keep the same order.
use super::L;

pub fn catalog() -> L {
    let mut l = super::en::catalog();
    l.app_title = " UGREEN / Studio Pro コントロール ";
    l.connected = "接続済み";
    l.disconnected = "未接続";
    l.unknown = "不明";
    l.target = "対象：";
    l.target_none = "未選択";
    l.protocol = "プロトコル：";
    l.protocol_on = "Studio Pro HP206（ユーザー選択）";
    l.protocol_off = "未選択；m で Studio Pro HP206 を確認";
    l.battery = "バッテリー：";
    l.battery_pct = "{}%";
    l.unavailable = "利用不可";
    l.codec_line = "コーデック：未検出";
    l.firmware = "ファームウェア：";
    l.snapshot = "読み取り：";
    l.snapshot_stale = "古い / 最新ではない";
    l.snapshot_ago = "{}秒前に読み取り";
    l.snapshot_never = "未読み取り";
    l.rfcomm_line = "RFCOMM {} | 操作ごとのタイムアウト {}秒 | 小売 fw 0.2.5 を確認済み";
    l.labels = [
        "ANC",
        "イコライザー",
        "ゲームモード",
        "空間オーディオ",
        "二重接続",
        "風ノイズ低減",
        "通知音",
        "音量上げ操作",
        "音量下げ操作",
    ];
    l.proposed = "  -> {}（提案）";
    l.settings_title = " 設定 / デバイス読取値、自動書き込みなし ";
    l.settings_blocked = " 設定 / 書き込みロック中：r で更新 ";
    l.settings_stale = " 設定 / 最後の読取値が古い ";
    l.status_title = " 状態 ";
    l.status_working = " 作業中 / Esc は以降の段階をキャンセル ";
    l.log_title = " セッションログ / 上下 Home End / l Esc 閉じる ";
    l.shortcuts_title = " ショートカット / 上下スクロール / ? Esc 閉じる ";
    l.edit_title = " 対象アドレスを編集 ";
    l.edit_lines = &[
        "16進数とコロンを入力。Backspace で削除；Ctrl-U で全消去。",
        "Enter はローカル保存のみ；Esc でキャンセル。接続は開始しない。",
    ];
    l.model_title = " モデル固有プロトコルの確認 ";
    l.model_lines = &[
        "UGREEN Studio Pro HP206 でのみ使用してください。",
        "小売ファームウェア 0.2.5 で検証済み（1台）；その他の",
        "バリアントとファームウェアは未検証。",
        "HiTune Max5c はコマンド ID が衝突するため非対応。",
        "これはプロトコルの選択であり、デバイスの身元証明ではない。",
        "y / Enter：Studio Pro HP206 を選択    n / Esc：キャンセル",
    ];
    l.paired_title = " ペアリング済みキャッシュ / 上下 Enter 選択 / Esc 閉じる ";
    l.paired_empty = "キャッシュにペアリング済みデバイスがありません。Esc で閉じます。";
    l.quit_title = " 処理中に終了しますか？ ";
    l.quit_lines = &[
        "キャンセルを要求しました；以降の段階はスキップされます。",
        "進行中の書き込みが完了する場合があります。",
        "終了は worker を待ちません。",
        "q / Enter：今すぐ終了    Esc：残って結果を待つ",
    ];
    l.resize_to = "最小 {}x{} にリサイズしてください";
    l.resize_quit = "Esc：キャンセル  q：終了";
    l.resize_inflight = "進行中の書き込みが完了する場合あり。q/Enter：終了；Esc：残る";
    l.resize_disabled = "このサイズでは操作を無効化しました。";
    l.footer = [
        [
            ("a", "アドレス"),
            ("p", "デバイス"),
            ("m", "モデル"),
            ("c", "接続"),
            ("r", "更新"),
            ("d", "切断"),
        ],
        [
            ("<", "減"),
            (">", "増"),
            ("Enter", "適用"),
            ("y", "確認"),
            ("Esc", "戻る"),
            ("", ""),
        ],
        [
            ("l", "ログ"),
            ("?", "ショートカット"),
            ("L", "言語"),
            ("q", "終了"),
            ("", ""),
            ("", ""),
        ],
    ];
    l.status_startup =
        "未接続。Bluetooth へのアクセスはまだありません。アドレス編集（a）またはペアリング済み読み取り（p）。";
    l.status_loading = "ペアリング済みデバイスをキャッシュから読み込み中（スキャンなし）...";
    l.status_connecting = "接続中、その後状態とファームウェアを読み取り中...";
    l.status_refreshing = "状態とファームウェアを更新中...";
    l.status_disconnecting = "制御を切断中...";
    l.status_applying = "プレフライト後に {}={} を適用し、確認と読取を検証中...";
    l.status_paired_empty =
        "キャッシュにペアリング済みデバイスがありません。まずシステムの Bluetooth 設定でペアリングしてください。";
    l.status_paired_pick = "目的のデバイスを選んで Enter。選択だけでは接続しません。";
    l.status_read_ok = "状態の読み取りに成功。表示される設定はデバイスの読取値です。";
    l.status_firmware_note = "設定は読み取ったがファームウェアは利用不可：{}";
    l.status_blocked_note = " 書き込みのロック解除には明示的な更新（r）が必要です。";
    l.status_verified = "確認と一致するデバイス読取により {}={} を検証しました。";
    l.status_disconnected = "制御を切断しました。Bluetooth は変更されていません。";
    l.status_quit_confirm = "キャンセルを要求しました。進行中の書き込みが完了する場合があります。q または Enter で待たずに終了；Esc で残ります。";
    l.status_cancel_busy = "キャンセルを要求しました；現在の段階を待っています。進行中の書き込みが完了する場合があります。以降の段階はスキップされます。";
    l.status_address_saved = "アドレスをローカルに保存しました。明示的に接続するには c。";
    l.status_bad_address =
        "コロン区切りの16進数オクテットを6つ入力してください。例：AA:BB:CC:DD:EE:FF。";
    l.status_model_on =
        "Studio Pro HP206 プロトコルをユーザーが選択しました。デバイスの身元は検証していません。";
    l.status_device_picked =
        "アドレスを選択しました。接続には Studio Pro プロトコルを確認してください（y または Enter）。";
    l.status_device_bad = "キャッシュ内のデバイスのアドレスが不正です。手動で編集してください。";
    l.status_cancel_then_disconnect =
        "キャンセルを要求しました；切断は現在の段階の後に続きます。進行中の書き込みが完了する場合があります。";
    l.status_must_disconnect = "対象やプロトコルを変える前に切断（d）してください。";
    l.status_must_connect = "未接続です。更新の前に接続（c）してください。";
    l.status_proposed = "提案値のみです。Enter で適用；Esc で破棄。";
    l.status_unavailable_setting = "この設定はデバイス読取値にありません。書き込みは無効です。";
    l.status_already_matches = "その値は最新の読取値と一致しています。書き込みは不要です。";
    l.status_pick_first = "適用する前に左/右で提案値を選んでください。";
    l.status_disabled = "変更は無効です。接続し、不確実な操作の後は明示的に更新してください。";
    l.status_lang = "言語：{}。L で巡回切替。";
    l.err_gone = "未接続です。まず接続してください。";
    l.err_status_query = "状態クエリに失敗：{}";
    l.err_paired_list = "ペアリング済みキャッシュの読み取りに失敗：{}";
    l.err_need_model = "接続前に Studio Pro HP206 プロトコルを確認してください。";
    l.err_connected_already = "別の対象に接続する前に切断してください。";
    l.err_connect = "接続に失敗：{}";
    l.err_uncertain_blocked =
        "不確実な操作の後は書き込みがロックされています。まず明示的に更新してください。";
    l.err_preflight = "プレフライト状態に失敗；設定は送信していません：{}";
    l.err_preflight_unreadable = "プレフライトが {} を読取不可；送信していません。";
    l.err_ack = "設定を送信した可能性があります；確認に失敗：{}。次の書き込み前に明示的に更新してください。";
    l.err_readback =
        "書き込みは確認されましたが読取に失敗：{}。次の書き込み前に明示的に更新してください。";
    l.err_readback_noack =
        "書き込みを送信しましたが読取に失敗：{}。次の書き込み前に明示的に更新してください。";
    l.err_mismatch = "書き込みは確認されましたが {} が {} と一致しません。次の書き込み前に明示的に更新してください。";
    l.err_mismatch_noack = "書き込みを送信しましたが {} が {} と一致しません。次の書き込み前に明示的に更新してください。";
    l.err_cancel_uncertain = "以降の段階をキャンセルしました。進行中の書き込みが完了した可能性があります；次の書き込み前に明示的に更新してください。";
    l.err_cancel = "キャンセルしました；以降の段階は開始しません。";
    l.cli_help = "ugreen 0.1.0 — UGREEN Studio Pro の非公式 Bluetooth CLI\n\n使用法\n  ugreen [オプション] コマンド\n\nオフラインコマンド\n  help                          このヘルプを表示\n  tui                           任意のターミナル UI を開く\n  models                        プロトコルの互換性と制限を表示\n  commands                      対応設定を一覧表示\n  decode HEX                    取得した応答バイトをオフラインで検証/復号\n  profile example               設定プロファイルの例を表示\n\nBluetooth コマンド（先にシステム設定でペアリング）\n  discover                      キャッシュのペアリング済み一覧；電波スキャンなし\n  capture                       セッションの TX/RX 生バイトをフィクスチャ16進数で記録\n  status                        バッテリー・ファームウェア・設定を読取\n  set キー 値                 設定を変更し読取を検証\n  profile export                設定を読取り再利用可能なプロファイルを出力\n  profile apply ファイル        プロファイルを検証・適用し各設定を検証\n\nオプション（コマンドの前）\n  --address XX:XX:XX:XX:XX:XX    明示的な対象 Bluetooth アドレス\n  --model studio-pro            ハードウェア要求に必須；自動検出しない\n  --channel 1                   RFCOMM チャンネル，1–30（既定 1）\n  --timeout 3                   操作ごとのタイムアウト秒数，1–60\n  --lang ja                     UI 言語：en, es, pt, de, fr, it, nl, ru, zh, ja, ko（既定 en）\n  --autoconnect                 起動時に前回のデバイスへ接続（デフォルトで有効）\n  --no-autoconnect              起動時の自動接続を無効化\n  --dry-run                     Bluetooth にアクセスせず設定パケットを表示\n  --help, -h                    ヘルプ表示\n  --version, -V                 バージョン表示\n\n例\n  ugreen discover\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro status\n  ugreen --dry-run set anc ultra\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro set eq bass\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro profile export > プロファイル.conf\n  ugreen --dry-run profile apply プロファイル.conf\n\nStudio Pro HP206 プロトコルは小売ファームウェア 0.2.5 で検証済み（1台）；\nHiTune Max5c はコマンド ID が衝突します。ファームウェア・リセット・生の書き込み・\nヘッドホン検索は提供しません。プロファイルは失敗時に部分的に適用される場合があります。\n";
    l.cli_models = "studio-pro：UGREEN Studio Pro HP206 のリファレンスコードとキャプチャに基づくプロトコル\nハードウェアテスト状況：小売ファームウェア 0.2.5 で検証済み（Linux，チャンネル 1；docs/verification.md 参照）\nHiTune Max5c：非対応（コマンド ID が衝突）\nその他の UGREEN モデル/ファームウェア：未検証";
    l.cli_commands = "anc: off|ultra|general|gentle|adaptive|ambient\neq: classic|jazz|electronic|pop|classical|rock|bass|treble\ngame, spatial, dual, wind: on|off\nprompts: voice|beeps\nvolume-up-action, volume-down-action: none|next|previous";
    l.cli_banner = "Studio Pro HP206 プロトコルを {} で使用，RFCOMM チャンネル {}";
    l.cli_need_model =
        "--model studio-pro を渡してこのモデル固有プロトコルを明示的に選択してください";
    l.cli_need_address = "--address が必要です；ペアリング済み一覧は discover を使ってください";
    l.cli_connect_fail = "Bluetooth 接続に失敗：{}。ペアリング・電源・アダプタ対応・他のアプリが制御チャンネルを占有していないか確認してください";
    l.cli_preflight_fail = "プレフライト状態に失敗；設定を送信していません：{}";
    l.cli_verified = "検証済み {}={}";
    l.cli_apply_fail = "{}={} で失敗する前に {} 個／合計 {} 個の設定を検証：{}。以前の設定はロールバックされません";
    l.cli_model_note = "model=studio-pro（ユーザー選択；デバイス身元の検証ではありません）";
    l.cli_unknown_value = "不明";
    l.cli_firmware_fail = "設定は読取ましたがファームウェア・クエリに失敗：{}";
    l.cli_no_paired =
        "ペアリング済みデバイスが返されませんでした。先にシステムの Bluetooth 設定でヘッドホンをペアリングしてください。";
    l.cli_paired_fail = "ペアリング済み一覧に失敗：{}";
    l.cli_dry_run_writes = "ドライラン：Bluetooth 接続もデバイスへの書き込みもなし";
    l.cli_bad_model = "--model studio-pro のみ実装；Max5c の ID は非互換";
    l.cli_bad_option = "未知のオプション '{}'";
    l.cli_bad_channel = "チャンネルは 1–30 です";
    l.cli_bad_timeout = "タイムアウトは 1–60 秒です";
    l.cli_need_value = "{} には値が必要です";
    l.cli_unknown_command = "未知のコマンド '{}'；ugreen --help を実行してください";
    l.cli_bad_usage = "使用法：{}";
    l.cli_bad_decode = "キャプチャはクリーンで完全な応答ストリームではありません：{} フレーム，{} 拒否 CRC，{} 破棄バイト，{} 不明なフレーム，{} 保留バイト";
    l.cli_no_tty = "TUI は双方向の対話ターミナルを必要とします；CLI コマンドは ugreen --help を実行してください。";
    l.cli_no_tui_feature =
        "この CLI のみのビルドでは TUI が無効です；--no-default-features なしで再ビルドしてください";
    l.cli_tui_dry =
        "tui は --dry-run と組み合わせられません；オフラインパケットには --dry-run set キー 値 を使ってください";
    l.cli_bad_profile = "プロファイルが 16 KiB を超えています";
    l.cli_inflight_after_quit = "進行中の書き込みが完了した可能性があります。さらに変更する前にデバイス状態を明示的に更新してください。";
    l.set_bad_anc = "無効な ANC 値";
    l.set_bad_eq = "無効な EQ 値";
    l.set_bad_onoff = "on または off が必要です";
    l.set_bad_prompts = "voice または beeps が必要です";
    l.set_bad_button = "none，next，previous のいずれかが必要です";
    l.set_unknown = "未知の設定 '{}'\n{}";
    l.profile_big = "プロファイルが 16 KiB を超えています";
    l.profile_line = "プロファイルの {} 行目は キー=値 にしてください";
    l.profile_dup = "重複したプロファイルキー '{}'";
    l.profile_model = "プロファイルのモデルは studio-pro です";
    l.profile_need_model = "プロファイルには model=studio-pro が必要です";
    l.profile_empty = "プロファイルに設定がありません";
    l.req_timeout = "期限までに一致する有効な応答がありません";
    l.req_closed = "Bluetooth 接続が閉じられました";
    l.req_rejected = "ヘッドホンが命令 0x{:02X} を拒否しました";
    l.info_short = "デバイス情報のペイロードが短すぎます：{} バイト（最低 8 必要）";
    l.fw_short = "ファームウェア応答にバージョン・バイトがありません";
    l.set_ackfail =
        "設定を送信した可能性がありますが確認に失敗：{}；再試行前に状態を照会してください";
    l.set_rbfail = "書き込みを確認しましたが読取に失敗：{}";
    l.set_rbfail_noack = "書き込みを送信しましたが読取に失敗：{}";
    l.set_mismatch =
        "書き込みを確認しましたが {} の読取が {} と一致しません；状態を照会してください";
    l.set_mismatch_noack =
        "書き込みを送信しましたが {} の読取が {} と一致しません；状態を照会してください";
    l
}
