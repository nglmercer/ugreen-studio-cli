//! Simplified Chinese catalog. Starts from English; untranslated
//! strings stay in English. `{}` placeholders keep the same order.
use super::L;

pub fn catalog() -> L {
    let mut l = super::en::catalog();
    l.app_title = " UGREEN / Studio Pro 控制 ";
    l.connected = "已连接";
    l.disconnected = "未连接";
    l.target = "目标：";
    l.target_none = "未选择";
    l.protocol = "协议：";
    l.protocol_on = "Studio Pro HP206（用户自选）";
    l.protocol_off = "未选择；按 m 确认 Studio Pro HP206";
    l.battery = "电池：";
    l.battery_pct = "{}%";
    l.unavailable = "不可用";
    l.codec_line = "编解码器：不可用（未提供）";
    l.firmware = "固件：";
    l.snapshot = "读取：";
    l.snapshot_stale = "已过期 / 非当前";
    l.snapshot_ago = "{} 秒前读取";
    l.snapshot_never = "未读取";
    l.rfcomm_line = "RFCOMM {} | 每次操作超时 {} 秒 | 已检查零售版 fw 0.2.5";
    l.labels = [
        "降噪",
        "均衡器",
        "游戏模式",
        "空间音频",
        "双设备连接",
        "防风降噪",
        "提示音",
        "音量加操作",
        "音量减操作",
    ];
    l.proposed = "  -> {}（建议）";
    l.settings_title = " 设置 / 设备读回，无自动写入 ";
    l.settings_blocked = " 设置 / 写入已锁定：按 r 刷新 ";
    l.settings_stale = " 设置 / 上次读取已过期 ";
    l.status_title = " 状态 ";
    l.status_working = " 处理中 / Esc 取消后续阶段 ";
    l.log_title = " 会话日志 / 上 下 Home End / l Esc 关闭 ";
    l.shortcuts_title = " 快捷键 / 上下滚动 / ? Esc 关闭 ";
    l.edit_title = " 编辑目标地址 ";
    l.edit_lines = &[
        "输入十六进制数字和冒号。退格删除；Ctrl-U 清空。",
        "Enter 仅保存在本地；Esc 取消。不会发起连接。",
    ];
    l.model_title = " 确认型号专用协议 ";
    l.model_lines = &[
        "仅与 UGREEN Studio Pro HP206 搭配使用。",
        "已在零售固件 0.2.5 上验证（一台设备）；",
        "其他型号与固件未经验证。",
        "HiTune Max5c 使用冲突的命令 ID，不受支持。",
        "这只是选择协议，不能证明设备身份。",
        "y / Enter：选择 Studio Pro HP206    n / Esc：取消",
    ];
    l.paired_title = " 已配对缓存 / 上 下 Enter 选择 / Esc 关闭 ";
    l.paired_empty = "缓存中没有已配对设备。Esc 关闭。";
    l.quit_title = " 任务进行中仍要退出吗？ ";
    l.quit_lines = &[
        "已请求取消；后续阶段将被跳过。",
        "进行中的写入仍可能完成。",
        "退出永远不会等待 worker。",
        "q / Enter：立即退出    Esc：留下并等待结果",
    ];
    l.resize_to = "请调整窗口至至少 {}x{}";
    l.resize_quit = "Esc：取消  q：退出";
    l.resize_inflight = "进行中的写入可能完成。q/Enter：退出；Esc：留下";
    l.resize_disabled = "此尺寸下操作已禁用。";
    l.footer = [
        [
            ("a", "地址"),
            ("p", "设备"),
            ("m", "型号"),
            ("c", "连接"),
            ("r", "刷新"),
            ("d", "断开"),
        ],
        [
            ("<", "减少"),
            (">", "增加"),
            ("Enter", "应用"),
            ("y", "确认"),
            ("Esc", "返回"),
            ("", ""),
        ],
        [
            ("l", "日志"),
            ("?", "快捷键"),
            ("L", "语言"),
            ("q", "退出"),
            ("", ""),
            ("", ""),
        ],
    ];
    l.status_startup = "未连接。尚无蓝牙访问权限。编辑地址（a）或读取已配对缓存（p）。";
    l.status_loading = "正在加载已配对设备缓存（不扫描）...";
    l.status_connecting = "正在连接，随后读取状态与固件...";
    l.status_refreshing = "正在刷新状态与固件...";
    l.status_disconnecting = "正在断开连接...";
    l.status_applying = "预查询后应用 {}={}，随后验证确认与读回...";
    l.status_paired_empty = "缓存中没有已配对设备。请先在系统蓝牙设置中配对。";
    l.status_paired_pick = "选择目标设备并按 Enter。选择本身不会连接。";
    l.status_read_ok = "状态读取成功。显示的设置来自设备读回。";
    l.status_firmware_note = "设置已读取；固件不可用：{}";
    l.status_blocked_note = " 仍需显式刷新（r）才能解锁写入。";
    l.status_verified = "已通过确认与一致的设备读回验证 {}={}。";
    l.status_disconnected = "已断开。不会自动重连。";
    l.status_quit_confirm =
        "已请求取消。进行中的写入可能完成。按 q 或 Enter 不等待直接退出；Esc 留下。";
    l.status_cancel_busy = "已请求取消；正在等待当前阶段。进行中的写入可能完成。后续阶段将被跳过。";
    l.status_address_saved = "地址已保存在本地。按 c 显式连接。";
    l.status_bad_address = "请输入六个以冒号分隔的十六进制字节，例如 AA:BB:CC:DD:EE:FF。";
    l.status_model_on = "Studio Pro HP206 协议由你选择，未验证设备身份。";
    l.status_device_picked = "地址已选择。请确认 Studio Pro 协议以连接（y 或 Enter）。";
    l.status_device_bad = "缓存中的设备地址无效；请手动编辑。";
    l.status_cancel_then_disconnect =
        "已请求取消；断开将在当前阶段之后进行。进行中的写入可能完成。";
    l.status_must_disconnect = "请先断开（d）再更改目标或协议。";
    l.status_must_connect = "未连接；请先连接（c）再刷新。";
    l.status_proposed = "仅为建议值。Enter 应用；Esc 放弃。";
    l.status_unavailable_setting = "此设置不在设备读回中；写入已禁用。";
    l.status_already_matches = "该值已与最新读回一致；无需写入。";
    l.status_pick_first = "请先用左/右选择一个建议值再应用。";
    l.status_disabled = "修改已禁用。请连接并在任何不确定操作后显式刷新。";
    l.status_lang = "语言：{}。按 L 循环切换。";
    l.err_gone = "未连接；请先连接。";
    l.err_status_query = "状态查询失败：{}";
    l.err_paired_list = "读取已配对缓存失败：{}";
    l.err_need_model = "连接前请先确认 Studio Pro HP206 协议。";
    l.err_connected_already = "连接其他目标前请先断开。";
    l.err_connect = "连接失败：{}";
    l.err_uncertain_blocked = "不确定操作后写入被锁定。请先显式刷新。";
    l.err_preflight = "预查询状态失败；未发送任何设置：{}";
    l.err_preflight_unreadable = "预查询无法读取 {}；未发送任何内容。";
    l.err_ack = "设置可能已发送；确认失败：{}。再次写入前请显式刷新。";
    l.err_readback = "写入已确认但读回失败：{}。再次写入前请显式刷新。";
    l.err_readback_noack = "写入已发送但读回失败：{}。再次写入前请显式刷新。";
    l.err_mismatch = "写入已确认，但 {} 与 {} 不一致。再次写入前请显式刷新。";
    l.err_mismatch_noack = "写入已发送，但 {} 与 {} 不一致。再次写入前请显式刷新。";
    l.err_cancel_uncertain = "后续阶段已取消。进行中的写入可能已完成；再次写入前请显式刷新。";
    l.err_cancel = "已取消；不会开始后续阶段。";
    l.cli_help = "ugreen 0.1.0 — 非官方的 UGREEN Studio Pro 蓝牙 CLI\n\n用法\n  ugreen [选项] 命令\n\n离线命令\n  help                          显示本帮助\n  tui                           打开可选的终端界面\n  models                        显示协议兼容性与限制\n  commands                      列出支持的设置\n  decode HEX                    离线校验/解码捕获的响应字节\n  profile example               显示示例设置配置文件\n\n蓝牙命令（请先在系统设置中配对）\n  discover                      列出缓存中的已配对设备；不扫描无线电\n  capture                       将会话的原始 TX/RX 字节记录为基准十六进制\n  status                        读取电池、固件与设置\n  set 键 值                    更改一个设置并验证读回\n  profile export                读取设置并打印可复用的配置文件\n  profile apply 文件            校验配置文件，应用并验证每个设置\n\n选项（位于命令之前）\n  --address XX:XX:XX:XX:XX:XX    显式的目标蓝牙地址\n  --model studio-pro            硬件请求必需；从不自动检测\n  --channel 1                   RFCOMM 通道，1–30（默认 1）\n  --timeout 3                   每次操作超时秒数，1–60\n  --lang zh                     界面语言：en, es, pt, de, fr, it, nl, ru, zh, ja, ko（默认 en）\n  --autoconnect                 TUI 启动时连接缓存中的目标\n  --no-autoconnect              禁用启动时自动连接\n  --dry-run                     仅打印设置数据包，不访问蓝牙\n  --help, -h                    显示帮助\n  --version, -V                 显示版本\n\n示例\n  ugreen discover\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro status\n  ugreen --dry-run set anc ultra\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro set eq bass\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro profile export > 我的配置.conf\n  ugreen --dry-run profile apply 我的配置.conf\n\nStudio Pro HP206 协议已在零售固件 0.2.5 上验证（一台设备）；\nHiTune Max5c 使用冲突的命令 ID。不提供固件、复位、原始写入\n或搜索耳机功能。配置文件失败时可能部分应用。\n";
    l.cli_models = "studio-pro：基于 UGREEN Studio Pro HP206 参考代码及其捕获的协议\n硬件测试状态：已在零售固件 0.2.5 上验证（Linux，通道 1；见 docs/verification.md）\nHiTune Max5c：不支持（命令 ID 冲突）\n其他 UGREEN 型号/固件：未验证";
    l.cli_commands = "anc: off|ultra|general|gentle|adaptive|ambient\neq: classic|jazz|electronic|pop|classical|rock|bass|treble\ngame, spatial, dual, wind: on|off\nprompts: voice|beeps\nvolume-up-action, volume-down-action: none|next|previous";
    l.cli_banner = "在 {} 上使用 Studio Pro HP206 协议，RFCOMM 通道 {}";
    l.cli_need_model = "请传递 --model studio-pro 以显式选择此型号专用协议";
    l.cli_need_address = "缺少 --address；请使用 discover 列出已配对设备";
    l.cli_connect_fail =
        "蓝牙连接失败：{}。请检查配对、电源、适配器支持，以及是否有其他应用占用控制通道";
    l.cli_preflight_fail = "预查询状态失败；未发送任何设置：{}";
    l.cli_verified = "已验证 {}={}";
    l.cli_apply_fail = "在 {}={} 失败前已验证 {} 个（共 {} 个）设置：{}。先前的设置不会回滚";
    l.cli_model_note = "model=studio-pro（用户自选；不是设备身份验证）";
    l.cli_unknown_value = "未知";
    l.cli_firmware_fail = "设置已读取，但固件查询失败：{}";
    l.cli_no_paired = "未返回已配对设备。请先在系统蓝牙设置中配对耳机。";
    l.cli_paired_fail = "列出已配对设备失败：{}";
    l.cli_dry_run_writes = "试运行：无蓝牙连接，也不写入设备";
    l.cli_bad_model = "仅实现了 --model studio-pro；Max5c 的 ID 不兼容";
    l.cli_bad_option = "未知选项 '{}'";
    l.cli_bad_channel = "通道必须为 1–30";
    l.cli_bad_timeout = "超时必须为 1–60 秒";
    l.cli_need_value = "{} 需要一个值";
    l.cli_unknown_command = "未知命令 '{}'；请运行 ugreen --help";
    l.cli_bad_usage = "用法：{}";
    l.cli_bad_decode =
        "捕获不是干净完整的响应流：{} 帧、{} 个被拒 CRC、{} 个被丢弃字节、{} 个未知帧、{} 个待定字节";
    l.cli_no_tty = "TUI 需要交互式的 stdin 与 stdout 终端；请运行 ugreen --help 查看 CLI 命令。";
    l.cli_no_tui_feature = "此仅 CLI 构建中 TUI 已禁用；请不带 --no-default-features 重新构建";
    l.cli_tui_dry = "tui 不能与 --dry-run 同时使用；请用 --dry-run set 键 值 查看离线数据包";
    l.cli_bad_profile = "配置文件超过 16 KiB";
    l.cli_inflight_after_quit = "进行中的写入可能已完成。请在进一步修改前显式刷新设备状态。";
    l.set_bad_anc = "无效的 ANC 值";
    l.set_bad_eq = "无效的 EQ 值";
    l.set_bad_onoff = "应为 on 或 off";
    l.set_bad_prompts = "应为 voice 或 beeps";
    l.set_bad_button = "应为 none、next 或 previous";
    l.set_unknown = "未知设置 '{}'\n{}";
    l.profile_big = "配置文件超过 16 KiB";
    l.profile_line = "配置文件第 {} 行必须是 键=值";
    l.profile_dup = "重复的配置键 '{}'";
    l.profile_model = "配置文件的型号必须为 studio-pro";
    l.profile_need_model = "配置文件需要 model=studio-pro";
    l.profile_empty = "配置文件没有设置";
    l.req_timeout = "截止前没有匹配的有效响应";
    l.req_closed = "蓝牙连接已关闭";
    l.req_rejected = "耳机拒绝了指令 0x{:02X}";
    l.info_short = "设备信息负载过短：{} 字节（至少需要 8）";
    l.fw_short = "固件响应缺少版本字节";
    l.set_ackfail = "设置可能已发送，但确认失败：{}；重试前请先查询状态";
    l.set_rbfail = "写入已确认，但读回失败：{}";
    l.set_rbfail_noack = "写入已发送，但读回失败：{}";
    l.set_mismatch = "写入已确认，但 {} 的读回与 {} 不一致；请查询状态";
    l.set_mismatch_noack = "写入已发送，但 {} 的读回与 {} 不一致；请查询状态";
    l
}
