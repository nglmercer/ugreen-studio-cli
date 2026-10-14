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
    l.cli_models = "studio-pro：UGREEN Studio Pro HP206 のリファレンスコードとキャプチャに基づくプロトコル\nハードウェアテスト状況：小売ファームウェア 0.2.5 で検証済み（Linux，チャンネル 1；docs/verification.md 参照）\nHiTune Max5c：非対応（コマンド ID が衝突）\nその他の UGREEN モデル/ファームウェア：未検証";
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
    // 生成されたヘルプの断片；画面自体は
    // コマンドレジストリからレンダリングされる。
    l.help_subtitle = "UGREEN Studio Pro の非公式 Bluetooth CLI";
    l.help_usage = "使用法";
    l.help_offline = "オフラインコマンド";
    l.help_bluetooth = "BLUETOOTH コマンド（先にシステム設定でペアリング）";
    l.help_capture_gated = "キャプチャ保護クエリ（接続前に拒否されます）";
    l.help_options = "オプション（コマンドの前）";
    l.help_examples = "例";
    l.help_footer = "Studio Pro HP206 プロトコルは retail ファームウェア 0.2.5 で検証済み（1 台）；\nHiTune Max5c はコマンド ID が衝突します。ファームウェア、リセット、生書き込み、\nヘッドフォン検索の各操作は提供されません。プロファイルは失敗時に部分的に適用される場合があります。";
    l.opt_address = "明示的なターゲット Bluetooth アドレス";
    l.opt_model = "ハードウェア要求に必須；自動検出はされません";
    l.opt_channel = "RFCOMM チャンネル，1–30（既定 1）";
    l.opt_timeout = "操作ごとのタイムアウト，秒，1–60";
    l.opt_lang = "インターフェース言語：en, es, pt, de, fr, it, nl, ru, zh, ja, ko（既定 en）";
    l.opt_autoconnect = "起動時に最後のデバイスへ接続（既定オン）";
    l.opt_no_autoconnect = "起動時の自動接続を無効化";
    l.opt_json = "対応する読み取り専用コマンドをバージョン付き JSON で出力";
    l.opt_dry_run = "Bluetooth アクセスなしで設定パケットを表示";
    l.opt_help = "ヘルプを表示";
    l.opt_version = "バージョンを表示";
    l.cmd_help = "このヘルプを表示";
    l.cmd_tui = "オプションの対話ターミナル画面を開く";
    l.cmd_models = "プロトコルの互換性と制限を表示";
    l.cmd_commands = "サポートされる設定を一覧表示";
    l.cmd_decode = "キャプチャした応答バイトをオフラインで検証/デコード";
    l.cmd_analyze = "APK の静的解析結果をオフラインで解釈";
    l.cmd_candidate = "静的パケット候補を表示；送信はされません";
    l.cmd_profile = "設定プロファイルを管理";
    l.cmd_profile_example = "設定プロファイルの例を表示";
    l.cmd_profile_export = "設定を読み取って再利用可能なプロファイルを表示";
    l.cmd_profile_apply = "プロファイルを検証し、適用して各設定を検証";
    l.cmd_discover = "キャッシュされたペアリング済みデバイスを一覧表示（電波スキャンなし）";
    l.cmd_capture = "セッションの生 TX/RX バイトをテスト用 hex で記録";
    l.cmd_status = "バッテリー、ファームウェア、設定を読み取り";
    l.cmd_set = "設定を変更し、読み取りで検証";
    l.cmd_query = "キャプチャ保護クエリ；このファームウェアでは未検証";
    l.cmd_control = "キャプチャ保護制御；--dry-run でバイトを表示";
    l.cmd_find = "キャプチャ保護の検索音";
    l.headset_connections = ["ヘッドセットの接続", "ヘッドセットの接続先です。このPCのペアリング一覧とは別です。", "HP206の照会（0x0D）には、このファームウェアでの公式アプリのキャプチャが必要です。照会は送信されていません。台数と接続先は不明です。", "リモート接続先の切断、再接続、切り替えは未対応です。ホストのBluetooth操作はこのPCだけに適用されます。", "r: 対応状況 · d: デュアル接続 · Esc: 戻る"];
    l.page_labels = [
        "デバイス",
        "オーディオ",
        "ヘッドセット設定",
        "マルチポイント",
        "実験的機能",
        "診断",
    ];
    l.page_hint = "↑/↓ Enter: 開く · r: 更新 · Esc: 戻る";
    l.host_commands = [
        "OS の音声接続を表示",
        "ホストの音声出力を一覧表示",
        "機能の検証状況を表示",
        "ホストの診断を読む",
        "ホストの Bluetooth を接続",
        "ホストの Bluetooth を切断",
    ];
    l.cmd_audio_devices = "Windows の波形出力デバイスを一覧表示";
    l.features_title = " 機能 ";
    l.features_search = "検索：";
    l.features_all = "すべて";
    l.feature_categories = [
        "オーディオ",
        "接続",
        "環境",
        "フィードバック",
        "デバイス情報",
        "マルチポイント",
        "コーデック",
        "高度なオーディオ",
        "操作",
        "オーディオ共有",
        "検索",
        "デバイス管理",
        "診断",
    ];
    l.feature_names = [
        "ANC",
        "イコライザー",
        "ゲームモード",
        "空間オーディオ",
        "デュアル接続",
        "風騒音低減",
        "音声案内",
        "音量上げの長押し操作",
        "音量下げの長押し操作",
        "マルチポイント機器",
        "オーディオコーデック",
        "ヘッドフォン ID",
        "ヘッドフォンの色",
        "ケースの MAC",
        "LDAC 制御",
        "一台向け LDAC",
        "高品質デコード",
        "音量コマンド",
        "モノラル/ステレオチャンネル",
        "ボタン割り当て",
        "AI アクション",
        "AI 制御",
        "オーディオ共有を停止",
        "一台と共有",
        "選択した一台と共有",
        "左のイヤホンを探す",
        "右のイヤホンを探す",
        "工場出荷状態に戻す",
        "キャプチャデコーダー",
        "キャプチャ分析",
        "パケットのプレビュー",
    ];
    l.feature_descriptions = [
        "アクティブノイズキャンセリングの深さ。検証済みの設定です。設定画面で編集してください。",
        "イコライザーのプリセット。検証済みの設定です。設定画面で編集してください。",
        "ゲームモードの低遅延スイッチ。検証済みの設定です。設定画面で編集してください。",
        "空間オーディオのスイッチ。検証済みの設定です。設定画面で編集してください。",
        "二台接続のスイッチ。検証済みの設定です。設定画面で編集してください。",
        "風騒音低減のスイッチ。検証済みの設定です。設定画面で編集してください。",
        "音声案内のスタイル。検証済みの設定です。設定画面で編集してください。",
        "音量上げ長押しの操作。検証済みの設定です。設定画面で編集してください。",
        "音量下げ長押しの操作。検証済みの設定です。設定画面で編集してください。",
        "接続中のマルチポイント機器一覧を読み取ります。ピアプロトコルはハードウェア検証がまだ必要です。",
        "現在の Bluetooth オーディオコーデックを読み取ります。照会形式はハードウェア検証がまだ必要です。",
        "ヘッドフォンのハードウェア ID を読み取ります。照会形式はハードウェア検証がまだ必要です。",
        "ヘッドフォンの色を読み取ります。照会形式はハードウェア検証がまだ必要です。",
        "充電ケースの MAC アドレスを読み取ります。照会形式はハードウェア検証がまだ必要です。",
        "現在の接続で LDAC 符号化を切り替えます。制御形式はハードウェア検証がまだ必要です。",
        "ペアリング済みの一台で LDAC 符号化を切り替えます。制御形式はハードウェア検証がまだ必要です。",
        "高品質オーディオデコードを切り替えます。制御形式はハードウェア検証がまだ必要です。",
        "絶対音量バイトを送信します。ハードウェアの範囲は検証されていません。",
        "オーディオチャンネルをモノラルとステレオで切り替えます。制御形式はハードウェア検証がまだ必要です。",
        "8 個の長押し操作を再割り当てします。制御形式はハードウェア検証がまだ必要です。",
        "AI が起動する 4 個の操作を再割り当てします。制御形式はハードウェア検証がまだ必要です。",
        "AI オーディオ制御を切り替えます。制御形式はハードウェア検証がまだ必要です。",
        "オーディオ共有を停止します。制御形式はハードウェア検証がまだ必要です。",
        "接続中の唯一の一台とオーディオを共有します。制御形式はハードウェア検証がまだ必要です。",
        "選択した一台とオーディオを共有します。制御形式はハードウェア検証がまだ必要です。",
        "ホスト経由で左のイヤホンに定位音を再生します。検索プロトコルはハードウェア検証がまだ必要です。",
        "ホスト経由で右のイヤホンに定位音を再生します。検索プロトコルはハードウェア検証がまだ必要です。",
        "ペアリングを解除し工場出荷値に戻します。破壊的です。制御形式はハードウェア検証がまだ必要です。",
        "キャプチャした応答バイトをオフラインでデコードします。Bluetooth アクセスはありません。",
        "完全なキャプチャストリームをオフラインで解釈します。Bluetooth アクセスはありません。",
        "制御が送信する静的パケットを表示します。送信されることはありません。",
    ];
    l.feature_available = "利用可能";
    l.feature_pending = "検証待ち";
    l.feature_pending_reason = "線路形式は公式アプリでのハードウェア検証がまだ必要です。";
    l.feature_live_disabled = "デバイス上の検証済み証拠があるまで、ライブ実行は無効です。";
    l.feature_offline_workflow = "オフライン分析のみ。ヘッドフォンには何も送信しません。";
    l.feature_offline_hint = "実行：ugreen {}";
    l.feature_kind_setting = "検証済みの設定";
    l.feature_kind_query = "デバイス照会";
    l.feature_kind_control = "デバイス制御";
    l.feature_kind_offline = "オフライン分析";
    l.feature_category_label = "カテゴリ：{}";
    l.feature_current = "現在：{}";
    l.feature_edit_in_settings = "設定で編集 (s)";
    l.feature_run_query = "Enter：照会を実行";
    l.feature_configure = "Enter：設定";
    l.feature_back = "Esc：戻る";
    l.feature_form_title = " {} を設定 ";
    l.feature_err_required = "確認する前にすべての項目を設定してください";
    l.feature_field_unset = "未設定";
    l.feature_form_hint = "上下で項目 / 左右で選択 / Enter で確認 / Esc で戻る";
    l.feature_review_title = " {} を確認 ";
    l.review_write_title = " 書き込みを確認 ";
    l.feature_review_target = "対象：{}";
    l.feature_review_previous = "現在：{}";
    l.feature_review_proposed = "提案：{}";
    l.feature_review_values = "値：{}";
    l.feature_review_readback = "読み取り：{}";
    l.feature_readback_verified = "確認応答と一致する読み取りで検証済み";
    l.feature_readback_noack = "一致する読み取りで検証済み（空間オーディオは確認応答を送らない）";
    l.feature_readback_unconfirmed = "個別に確認されていません。適用後に更新してください。";
    l.feature_readback_query = "読み取り専用の照会";
    l.feature_confirm_apply = "Enter：適用    Esc：キャンセル";
    l.feature_confirm_reset = "破壊的：ホストのペアリングを削除し工場出荷値に戻します。";
    l.features_footer = [
        [
            ("上", "移動"),
            ("下", "移動"),
            ("Tab", "カテゴリ"),
            ("Enter", "開く"),
            ("Esc", "戻る"),
            ("", ""),
        ],
        [
            ("文字", "検索"),
            ("Backspace", "削除"),
            ("Ctrl+U", "クリア"),
            ("", ""),
            ("", ""),
            ("", ""),
        ],
        [
            ("Esc", "閉じる"),
            ("?", "ショートカット"),
            ("", ""),
            ("", ""),
            ("", ""),
            ("", ""),
        ],
    ];
    l.status_feature_running = "機能操作を実行中...";
    l.status_edit_in_settings = "{} は検証済みの設定です。設定画面で編集してください (s)。";
    l.codec_unverified = "照会は未検証";
    l.arg_enabled = "有効";
    l.arg_side = "側";
    l.arg_duration = "時間（秒）";
    l.arg_target = "対象";
    l.arg_value = "値（0-255）";
    l.arg_mode = "モード";
    l.arg_peer = "ペアリング済み";
    l.arg_actions = "アクション";
    l.arg_address = "ペアリング済みの MAC";
    l.arg_hint_toggle = "on または off";
    l.arg_hint_side = "左または右";
    l.arg_hint_seconds = "1..30 秒";
    l.arg_hint_byte = "0..255";
    l.arg_hint_mac = "コロン区切りの 6 つの 16 進オクテット";
    l.arg_hint_peer = "1 または 2";
    l.arg_hint_actions = "{} 個のコンマ区切りアクション値";
    l.tui_no_tty = "この画面は対話式の stdin/stdout 端末を必要とします。CLI コマンドは ugreen --help を実行してください。";
    l.tui_bad_channel =
        "画面のチャンネルは 1-30、タイムアウトは 0 より大きく最大 60 秒である必要があります。";
    l.status_worker_busy = "操作は既に実行中です。Esc でキャンセルするか待機してください。";
    l.status_worker_sequence = "リクエストの連番が尽きました。画面を再起動してください。";
    l.status_worker_stopped = "Bluetooth ワーカーが停止しました。画面を再起動してください。";
    l.status_worker_queue = "ワーカーのキューが満杯です。待ってから再試行してください。";
    l.status_reset_unavailable = "リセット対象が利用できません";
    l.status_reset_done =
        "リセットを確認しました。ホストのペアリングと保存済みデバイスを削除しました。";
    l.status_inflight_write =
        "実行中の書き込みが完了した可能性があります。再起動後に更新してください。";
    l
}
