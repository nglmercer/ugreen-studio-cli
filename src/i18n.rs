//! English/Spanish user-facing text. Protocol values, profile file syntax,
//! `key=value` output labels and wire bytes are never translated: only the
//! prose around them is. `Lang::En` must reproduce the historical strings
//! exactly; several tests pin them.
use std::env;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Lang {
    #[default]
    En,
    Es,
}

impl Lang {
    pub fn from_code(code: &str) -> Result<Self, String> {
        match code {
            "en" => Ok(Self::En),
            "es" => Ok(Self::Es),
            _ => Err(format!("invalid language '{code}'; expected en or es")),
        }
    }
    pub fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Es => "es",
        }
    }
    /// OS locale guess. `Config::default()` stays English so tests never
    /// depend on the environment; the CLI applies this only when `--lang`
    /// is absent. `UGREEN_LANG` wins when set, so tests and scripts can pin
    /// the language regardless of locale.
    pub fn detect() -> Self {
        if let Ok(value) = env::var("UGREEN_LANG") {
            if let Ok(lang) = Self::from_code(value.trim()) {
                return lang;
            }
        }
        for key in ["LANGUAGE", "LC_ALL", "LC_MESSAGES", "LANG"] {
            if let Ok(value) = env::var(key) {
                let lower = value.to_ascii_lowercase();
                let tag = lower
                    .split(['.', '@', ':'])
                    .next()
                    .unwrap_or("")
                    .replace('_', "-");
                if tag == "es" || tag.starts_with("es-") {
                    return Self::Es;
                }
                if !tag.is_empty() && tag != "c" && tag != "posix" {
                    return Self::En;
                }
            }
        }
        Self::En
    }
}

/// Every user-facing prose string, in both languages. `{}` placeholders keep
/// the same argument order in both translations.
pub struct L {
    pub app_title: &'static str,
    pub connected: &'static str,
    pub disconnected: &'static str,
    pub target: &'static str,
    pub target_none: &'static str,
    pub protocol: &'static str,
    pub protocol_on: &'static str,
    pub protocol_off: &'static str,
    pub battery: &'static str,
    pub battery_pct: &'static str,
    pub unavailable: &'static str,
    pub codec_line: &'static str,
    pub firmware: &'static str,
    pub snapshot: &'static str,
    pub snapshot_stale: &'static str,
    pub snapshot_ago: &'static str,
    pub snapshot_never: &'static str,
    pub rfcomm_line: &'static str,
    pub labels: [&'static str; 9],
    pub proposed: &'static str,
    pub settings_title: &'static str,
    pub settings_blocked: &'static str,
    pub settings_stale: &'static str,
    pub status_title: &'static str,
    pub status_working: &'static str,
    pub log_title: &'static str,
    pub shortcuts_title: &'static str,
    pub shortcuts: &'static [&'static str],
    pub edit_title: &'static str,
    pub edit_lines: &'static [&'static str],
    pub model_title: &'static str,
    pub model_lines: &'static [&'static str],
    pub confirm_title: &'static str,
    pub confirm_target: &'static str,
    pub confirm_lines: &'static [&'static str],
    pub paired_title: &'static str,
    pub paired_empty: &'static str,
    pub quit_title: &'static str,
    pub quit_lines: &'static [&'static str],
    pub resize_to: &'static str,
    pub resize_quit: &'static str,
    pub resize_inflight: &'static str,
    pub resize_disabled: &'static str,
    pub footer: [[(&'static str, &'static str); 6]; 3],
    pub status_startup: &'static str,
    pub status_loading: &'static str,
    pub status_connecting: &'static str,
    pub status_refreshing: &'static str,
    pub status_disconnecting: &'static str,
    pub status_applying: &'static str,
    pub status_paired_empty: &'static str,
    pub status_paired_pick: &'static str,
    pub status_read_ok: &'static str,
    pub status_firmware_note: &'static str,
    pub status_blocked_note: &'static str,
    pub status_verified: &'static str,
    pub status_disconnected: &'static str,
    pub status_quit_confirm: &'static str,
    pub status_cancel_busy: &'static str,
    pub status_address_saved: &'static str,
    pub status_bad_address: &'static str,
    pub status_model_on: &'static str,
    pub status_device_picked: &'static str,
    pub status_device_bad: &'static str,
    pub status_cancel_then_disconnect: &'static str,
    pub status_must_disconnect: &'static str,
    pub status_must_connect: &'static str,
    pub status_proposed: &'static str,
    pub status_unavailable_setting: &'static str,
    pub status_already_matches: &'static str,
    pub status_pick_first: &'static str,
    pub status_disabled: &'static str,
    pub status_lang: &'static str,
    pub err_gone: &'static str,
    pub err_status_query: &'static str,
    pub err_paired_list: &'static str,
    pub err_need_model: &'static str,
    pub err_connected_already: &'static str,
    pub err_connect: &'static str,
    pub err_uncertain_blocked: &'static str,
    pub err_preflight: &'static str,
    pub err_preflight_unreadable: &'static str,
    pub err_ack: &'static str,
    pub err_readback: &'static str,
    pub err_readback_noack: &'static str,
    pub err_mismatch: &'static str,
    pub err_mismatch_noack: &'static str,
    pub err_cancel_uncertain: &'static str,
    pub err_cancel: &'static str,
    pub cli_help: &'static str,
    pub cli_models: &'static str,
    pub cli_commands: &'static str,
    pub cli_banner: &'static str,
    pub cli_need_model: &'static str,
    pub cli_need_address: &'static str,
    pub cli_connect_fail: &'static str,
    pub cli_preflight_fail: &'static str,
    pub cli_verified: &'static str,
    pub cli_apply_fail: &'static str,
    pub cli_model_note: &'static str,
    pub cli_unknown_value: &'static str,
    pub cli_firmware_fail: &'static str,
    pub cli_no_paired: &'static str,
    pub cli_paired_fail: &'static str,
    pub cli_dry_run_writes: &'static str,
    pub cli_bad_model: &'static str,
    pub cli_bad_option: &'static str,
    pub cli_bad_channel: &'static str,
    pub cli_bad_timeout: &'static str,
    pub cli_need_value: &'static str,
    pub cli_unknown_command: &'static str,
    pub cli_bad_usage: &'static str,
    pub cli_bad_decode: &'static str,
    pub cli_no_tty: &'static str,
    pub cli_no_tui_feature: &'static str,
    pub cli_tui_dry: &'static str,
    pub cli_bad_profile: &'static str,
    pub cli_inflight_after_quit: &'static str,
    pub set_bad_anc: &'static str,
    pub set_bad_eq: &'static str,
    pub set_bad_onoff: &'static str,
    pub set_bad_prompts: &'static str,
    pub set_bad_button: &'static str,
    pub set_unknown: &'static str,
    pub profile_big: &'static str,
    pub profile_line: &'static str,
    pub profile_dup: &'static str,
    pub profile_model: &'static str,
    pub profile_need_model: &'static str,
    pub profile_empty: &'static str,
    pub req_timeout: &'static str,
    pub req_closed: &'static str,
    pub req_rejected: &'static str,
    pub info_short: &'static str,
    pub fw_short: &'static str,
    pub set_ackfail: &'static str,
    pub set_rbfail: &'static str,
    pub set_rbfail_noack: &'static str,
    pub set_mismatch: &'static str,
    pub set_mismatch_noack: &'static str,
}

const EN: L = L {
    app_title: " UGREEN / Studio Pro control ",
    connected: "CONNECTED",
    disconnected: "DISCONNECTED",
    target: "Target: ",
    target_none: "not selected",
    protocol: "Protocol: ",
    protocol_on: "Studio Pro HP206 (user-selected)",
    protocol_off: "not selected; m to confirm Studio Pro HP206",
    battery: "Battery: ",
    battery_pct: "{}%",
    unavailable: "unavailable",
    codec_line: "Codec: unavailable (not exposed)",
    firmware: "Firmware: ",
    snapshot: "Snapshot: ",
    snapshot_stale: "STALE / not current",
    snapshot_ago: "read {}s ago",
    snapshot_never: "not read",
    rfcomm_line: "RFCOMM {} | per-operation timeout {}s | retail fw 0.2.5 checked",
    labels: [
        "ANC",
        "Equalizer",
        "Game mode",
        "Spatial audio",
        "Dual connection",
        "Wind reduction",
        "Prompts",
        "Volume-up action",
        "Volume-down action",
    ],
    proposed: "  -> {} (proposed)",
    settings_title: " Settings / device readback, no automatic writes ",
    settings_blocked: " Settings / WRITES BLOCKED: r to refresh ",
    settings_stale: " Settings / last snapshot is stale ",
    status_title: " Status ",
    status_working: " Working / Esc cancels later stages ",
    log_title: " Session log / Up Down Home End / l Esc close ",
    shortcuts_title: " Shortcuts / Up Down scroll / ? Esc close ",
    shortcuts: &[
        "Connection",
        "  a ......... Edit target address (while disconnected)",
        "  p ......... Load paired-device cache (auto-loads at startup)",
        "  m then y .. Confirm Studio Pro HP206 protocol",
        "  c / r / d . Connect / refresh status / disconnect",
        "Settings",
        "  Up/Down ... Select setting",
        "  Left/Right  Propose a value (local only, no write)",
        "  Enter ..... Review the proposed change",
        "  y / n ..... Confirm once / cancel the review",
        "View",
        "  l ......... Session log (latest 100 events)",
        "  L ......... Switch language (English/Español)",
        "  ? ......... This shortcuts screen",
        "  q ......... Quit (asks again while work is in flight)",
        "Mouse",
        "  Click ..... Select a setting, device or shortcut button",
        "  Click again Advance the proposed value",
        "  Wheel ..... Move selection or scroll",
        "  Right-click Cancel / go back (Esc)",
        "Safety",
        "  Writes need preflight, acknowledgement (spatial audio:",
        "  readback only) and matching device-info readback.",
        "  Uncertain writes stay blocked until an explicit refresh.",
        "  Model selection does not verify hardware identity.",
    ],
    edit_title: " Edit target address ",
    edit_lines: &[
        "Type hexadecimal digits and colons. Backspace deletes; Ctrl-U clears.",
        "Enter saves locally; Esc cancels. No connection is started.",
    ],
    model_title: " Confirm model-specific protocol ",
    model_lines: &[
        "Use this only with UGREEN Studio Pro HP206.",
        "Hardware compatibility has not been verified on this build.",
        "HiTune Max5c uses conflicting command IDs and is unsupported.",
        "This selects a protocol; it does not prove device identity.",
        "y: select Studio Pro HP206    n / Esc: cancel",
    ],
    confirm_title: " Confirm one setting write ",
    confirm_target: "Target: {} (user-selected Studio Pro HP206)",
    confirm_lines: &[
        "A preflight query, acknowledgement and readback are required.",
        "No automatic retry or rollback. A partial write may take effect.",
        "y: apply once    n / Esc: cancel",
    ],
    paired_title: " Paired cache / Up Down Enter select / Esc close ",
    paired_empty: "No cached paired devices. Esc closes.",
    quit_title: " Quit while work is in flight? ",
    quit_lines: &[
        "Cancellation requested; later stages will be skipped.",
        "An in-flight setting write may still complete.",
        "Quitting never waits for the worker.",
        "q / Enter: quit now    Esc: stay and wait for the result",
    ],
    resize_to: "Resize to at least {}x{}",
    resize_quit: "Esc: cancel  q: quit",
    resize_inflight: "In-flight write may complete. q/Enter: quit; Esc: stay",
    resize_disabled: "Actions disabled at this size.",
    footer: [
        [
            ("a", "address"),
            ("p", "devices"),
            ("m", "model"),
            ("c", "connect"),
            ("r", "refresh"),
            ("d", "disconnect"),
        ],
        [
            ("<", "less"),
            (">", "more"),
            ("Enter", "review"),
            ("y", "apply"),
            ("Esc", "back"),
            ("", ""),
        ],
        [
            ("l", "log"),
            ("?", "shortcuts"),
            ("L", "language"),
            ("q", "quit"),
            ("", ""),
            ("", ""),
        ],
    ],
    status_startup:
        "Disconnected. No Bluetooth access yet. Edit address (a) or read paired cache (p).",
    status_loading: "Loading cached paired devices (no scan)...",
    status_connecting: "Connecting, then reading status and firmware...",
    status_refreshing: "Refreshing status and firmware...",
    status_disconnecting: "Disconnecting...",
    status_applying:
        "Applying {}={} after preflight, then verifying acknowledgement and readback...",
    status_paired_empty: "No cached paired devices. Pair in OS Bluetooth settings first.",
    status_paired_pick: "Select the intended device and press Enter. Selection does not connect.",
    status_read_ok: "Status read successfully. Settings shown are device readback.",
    status_firmware_note: "Settings read; firmware unavailable: {}",
    status_blocked_note: " Explicit refresh (r) is still required to unlock writes.",
    status_verified: "Verified {}={} by acknowledgement and matching device readback.",
    status_disconnected: "Disconnected. No automatic reconnect.",
    status_quit_confirm: "Cancellation requested. An in-flight write may complete. Press q or Enter to quit without waiting; Esc to stay.",
    status_cancel_busy: "Cancellation requested; waiting for current stage. An in-flight write may complete. Further stages will be skipped.",
    status_address_saved: "Address saved locally. Press c to connect explicitly.",
    status_bad_address:
        "Enter six colon-separated hexadecimal octets, e.g. AA:BB:CC:DD:EE:FF.",
    status_model_on:
        "Studio Pro HP206 protocol selected by you, not device-identity verified. Press c to connect.",
    status_device_picked:
        "Address selected locally. Confirm the intended device, then press c to connect.",
    status_device_bad: "Cached device has an invalid address; edit it manually.",
    status_cancel_then_disconnect:
        "Cancellation requested; disconnect will follow the current stage. An in-flight write may complete.",
    status_must_disconnect: "Disconnect (d) before changing the target or protocol.",
    status_must_connect: "Disconnected; connect (c) before refreshing.",
    status_proposed:
        "Proposed value only. Enter reviews the change; y confirms one write. Esc discards.",
    status_unavailable_setting:
        "This setting is unavailable in device readback; writing is disabled.",
    status_already_matches: "That value already matches the latest readback; no write needed.",
    status_pick_first: "Use Left/Right to choose a proposed value before applying.",
    status_disabled:
        "Changes disabled. Connect and explicitly refresh after any uncertain operation.",
    status_lang: "Language: English. Pulsa L para español.",
    err_gone: "Disconnected; connect first.",
    err_status_query: "Status query failed: {}",
    err_paired_list: "Cached paired-device list failed: {}",
    err_need_model: "Confirm Studio Pro HP206 protocol before connecting.",
    err_connected_already: "Disconnect before connecting another target.",
    err_connect: "Connection failed: {}",
    err_uncertain_blocked:
        "Writes blocked after an uncertain operation. Explicitly refresh first.",
    err_preflight: "Preflight status failed; no setting sent: {}",
    err_preflight_unreadable: "Preflight cannot read {}; no setting sent.",
    err_ack: "Setting may have been sent; acknowledgement failed: {}. Explicitly refresh before another write.",
    err_readback: "Write acknowledged but readback failed: {}. Explicitly refresh before another write.",
    err_readback_noack: "Write sent but readback failed: {}. Explicitly refresh before another write.",
    err_mismatch: "Write acknowledged but {} did not match {}. Explicitly refresh before another write.",
    err_mismatch_noack: "Write sent but {} did not match {}. Explicitly refresh before another write.",
    err_cancel_uncertain: "Cancelled later stages. An in-flight write may have completed; explicitly refresh before another write.",
    err_cancel: "Cancelled; no further stages will start.",
    cli_help: "ugreen 0.1.0 — unofficial UGREEN Studio Pro Bluetooth CLI\n\nUSAGE\n  ugreen [OPTIONS] COMMAND\n\nOFFLINE COMMANDS\n  help                          Show this help\n  tui                           Open the optional interactive terminal UI\n  models                        Show protocol compatibility and limits\n  commands                      List supported settings\n  decode HEX                    Validate/decode captured response bytes offline\n  profile example               Print an example settings profile\n\nBLUETOOTH COMMANDS (pair in OS settings first)\n  discover                      List cached paired devices; no radio scan\n  status                        Read battery, firmware and settings\n  set KEY VALUE                 Change one setting, then verify readback\n  profile export                Read settings and print a reusable profile\n  profile apply FILE            Validate profile, apply and verify each setting\n\nOPTIONS (before COMMAND)\n  --address XX:XX:XX:XX:XX:XX    Explicit target Bluetooth address\n  --model studio-pro            Required for hardware requests; never auto-detected\n  --channel 1                   RFCOMM channel, 1–30 (default 1)\n  --timeout 3                   Per-operation timeout in seconds, 1–60\n  --lang en                     Interface language: en or es (default en)\n  --dry-run                     Print setting packets without Bluetooth access\n  --help, -h                    Show help\n  --version, -V                 Show version\n\nEXAMPLES\n  ugreen discover\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro status\n  ugreen --dry-run set anc ultra\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro set eq bass\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro profile export > my-profile.conf\n  ugreen --dry-run profile apply my-profile.conf\n\nStudio Pro HP206 protocol verified on retail firmware 0.2.5 (one unit);\nHiTune Max5c uses conflicting command IDs. No firmware, reset, raw-write or\nfind-headphones actions are provided. Profiles may apply partially on failure.\n",
    cli_models: "studio-pro: protocol based on UGREEN Studio Pro HP206 reference code and capture\nHardware test status: VERIFIED on retail firmware 0.2.5 (Linux, channel 1; see docs/verification.md)\nHiTune Max5c: NOT SUPPORTED (conflicting command IDs)\nOther UGREEN models/firmware: NOT VERIFIED",
    cli_commands: "anc: off|ultra|general|gentle|adaptive|ambient\neq: classic|jazz|electronic|pop|classical|rock|bass|treble\ngame, spatial, dual, wind: on|off\nprompts: voice|beeps\nvolume-up-action, volume-down-action: none|next|previous",
    cli_banner: "Using Studio Pro HP206 protocol at {}, RFCOMM channel {}",
    cli_need_model:
        "pass --model studio-pro to explicitly select this model-specific protocol",
    cli_need_address: "--address is required; use discover to list paired devices",
    cli_connect_fail: "Bluetooth connection failed: {}. Check pairing, power, adapter support, and whether another app owns the control channel",
    cli_preflight_fail: "preflight status failed; no settings sent: {}",
    cli_verified: "Verified {}={}",
    cli_apply_fail: "{} of {} settings verified before failure at {}={}: {}. Earlier settings are not rolled back",
    cli_model_note: "model=studio-pro (user-selected; not device identity verification)",
    cli_unknown_value: "unknown",
    cli_firmware_fail: "settings read, but firmware query failed: {}",
    cli_no_paired:
        "No paired devices returned. Pair the headphones in OS Bluetooth settings first.",
    cli_paired_fail: "paired-device listing failed: {}",
    cli_dry_run_writes: "Dry run: no Bluetooth connection or device writes",
    cli_bad_model: "only --model studio-pro is implemented; Max5c IDs are incompatible",
    cli_bad_option: "unknown option '{}'",
    cli_bad_channel: "channel must be 1–30",
    cli_bad_timeout: "timeout must be 1–60 seconds",
    cli_need_value: "{} requires a value",
    cli_unknown_command: "unknown command '{}'; run ugreen --help",
    cli_bad_usage: "usage: {}",
    cli_bad_decode: "capture is not a clean complete response stream: {} frames, {} rejected CRCs, {} discarded bytes, {} pending bytes",
    cli_no_tty: "The TUI requires interactive stdin and stdout terminals; run ugreen --help for CLI commands.",
    cli_no_tui_feature:
        "TUI feature is disabled in this CLI-only build; rebuild without --no-default-features",
    cli_tui_dry:
        "tui cannot be combined with --dry-run; use --dry-run set KEY VALUE for offline packets",
    cli_bad_profile: "profile exceeds 16 KiB",
    cli_inflight_after_quit: "An in-flight setting write may have completed. Explicitly refresh device status before making further changes.",
    set_bad_anc: "invalid ANC value",
    set_bad_eq: "invalid EQ value",
    set_bad_onoff: "expected on or off",
    set_bad_prompts: "expected voice or beeps",
    set_bad_button: "expected none, next or previous",
    set_unknown: "unknown setting '{}'\n{}",
    profile_big: "profile exceeds 16 KiB",
    profile_line: "profile line {} must be key=value",
    profile_dup: "duplicate profile key '{}'",
    profile_model: "profile model must be studio-pro",
    profile_need_model: "profile requires model=studio-pro",
    profile_empty: "profile has no settings",
    req_timeout: "no matching valid response before deadline",
    req_closed: "Bluetooth connection closed",
    req_rejected: "headphones rejected instruction 0x{:02X}",
    info_short: "device-info payload too short: {} bytes (need at least 8)",
    fw_short: "firmware response lacks version bytes",
    set_ackfail: "setting may have been sent but acknowledgement failed: {}; query status before retrying",
    set_rbfail: "write acknowledged but readback failed: {}",
    set_rbfail_noack: "write sent but readback failed: {}",
    set_mismatch: "write acknowledged but {} readback does not match {}; query status",
    set_mismatch_noack: "write sent but {} readback does not match {}; query status",
};

const ES: L = L {
    app_title: " UGREEN / control Studio Pro ",
    connected: "CONECTADO",
    disconnected: "DESCONECTADO",
    target: "Destino: ",
    target_none: "sin elegir",
    protocol: "Protocolo: ",
    protocol_on: "Studio Pro HP206 (elegido por ti)",
    protocol_off: "sin elegir; m para confirmar Studio Pro HP206",
    battery: "Batería: ",
    battery_pct: "{}\u{a0}%",
    unavailable: "no disponible",
    codec_line: "Códec: no disponible (no expuesto)",
    firmware: "Firmware: ",
    snapshot: "Lectura: ",
    snapshot_stale: "OBSOLETA / no actual",
    snapshot_ago: "leída hace {}\u{a0}s",
    snapshot_never: "sin leer",
    rfcomm_line: "RFCOMM {} | tiempo por operación {}\u{a0}s | fw 0.2.5 verificado",
    labels: [
        "ANC",
        "Ecualizador",
        "Modo juego",
        "Audio espacial",
        "Conexión dual",
        "Reducción de viento",
        "Avisos",
        "Acción subir volumen",
        "Acción bajar volumen",
    ],
    proposed: "  -> {} (propuesto)",
    settings_title: " Ajustes / lectura del dispositivo, sin escritura automática ",
    settings_blocked: " Ajustes / ESCRITURA BLOQUEADA: r para actualizar ",
    settings_stale: " Ajustes / última lectura obsoleta ",
    status_title: " Estado ",
    status_working: " Trabajando / Esc cancela las siguientes etapas ",
    log_title: " Registro / Arriba Abajo Inicio Fin / l Esc cerrar ",
    shortcuts_title: " Atajos / Arriba Abajo para ver / ? Esc cerrar ",
    shortcuts: &[
        "Conexión",
        "  a ......... Editar dirección de destino (desconectado)",
        "  p ......... Cargar dispositivos emparejados (auto al iniciar)",
        "  m luego y . Confirmar protocolo Studio Pro HP206",
        "  c / r / d . Conectar / actualizar estado / desconectar",
        "Ajustes",
        "  Arriba/Abajo Seleccionar ajuste",
        "  Izq/Der .... Proponer valor (solo local, no escribe)",
        "  Enter ...... Revisar el cambio propuesto",
        "  y / n ...... Confirmar una vez / cancelar la revisión",
        "Vista",
        "  l ......... Registro de sesión (últimos 100 eventos)",
        "  L ......... Cambiar idioma (English/Español)",
        "  ? ......... Esta pantalla de atajos",
        "  q ......... Salir (pregunta si hay trabajo en curso)",
        "Ratón",
        "  Clic ...... Seleccionar ajuste, dispositivo o botón",
        "  Otro clic . Avanzar el valor propuesto",
        "  Rueda ..... Mover selección o desplazar",
        "  Clic der .. Cancelar / volver (Esc)",
        "Seguridad",
        "  Escritura con consulta previa, confirmación (audio",
        "  espacial: solo lectura) y lectura coincidente.",
        "  Tras un resultado incierto, actualizar antes de escribir.",
        "  Elegir modelo no verifica la identidad del hardware.",
    ],
    edit_title: " Editar dirección de destino ",
    edit_lines: &[
        "Escribe dígitos hexadecimales y dos puntos. Retroceso borra; Ctrl-U limpia.",
        "Enter guarda localmente; Esc cancela. No se inicia conexión.",
    ],
    model_title: " Confirmar protocolo específico del modelo ",
    model_lines: &[
        "Úsalo solo con UGREEN Studio Pro HP206.",
        "La compatibilidad no se ha verificado en esta compilación.",
        "HiTune Max5c usa otros identificadores y no es compatible.",
        "Esto elige un protocolo; no prueba la identidad del dispositivo.",
        "y: elegir Studio Pro HP206    n / Esc: cancelar",
    ],
    confirm_title: " Confirmar una escritura ",
    confirm_target: "Destino: {} (Studio Pro HP206 elegido por ti)",
    confirm_lines: &[
        "Se exigen consulta previa, confirmación y lectura posterior.",
        "Sin reintento ni reversión. Un cambio parcial puede aplicarse.",
        "y: aplicar una vez    n / Esc: cancelar",
    ],
    paired_title: " Emparejados / Arriba Abajo Enter elegir / Esc cerrar ",
    paired_empty: "Sin dispositivos emparejados. Esc cierra.",
    quit_title: " ¿Salir con trabajo en curso? ",
    quit_lines: &[
        "Cancelación pedida; se omitirán las etapas siguientes.",
        "Una escritura en curso puede haberse aplicado.",
        "Al salir nunca se espera al trabajador.",
        "q / Enter: salir ya    Esc: quedarse y esperar el resultado",
    ],
    resize_to: "Agranda a al menos {}x{}",
    resize_quit: "Esc: cancelar  q: salir",
    resize_inflight: "Una escritura puede haberse aplicado. q/Enter: salir; Esc: quedarse",
    resize_disabled: "Acciones desactivadas con este tamaño.",
    footer: [
        [
            ("a", "dirección"),
            ("p", "dispositivos"),
            ("m", "modelo"),
            ("c", "conectar"),
            ("r", "actualizar"),
            ("d", "desconectar"),
        ],
        [
            ("<", "menos"),
            (">", "más"),
            ("Enter", "revisar"),
            ("y", "aplicar"),
            ("Esc", "atrás"),
            ("", ""),
        ],
        [
            ("l", "registro"),
            ("?", "atajos"),
            ("L", "idioma"),
            ("q", "salir"),
            ("", ""),
            ("", ""),
        ],
    ],
    status_startup:
        "Desconectado. Sin Bluetooth aún. Edita la dirección (a) o lee emparejados (p).",
    status_loading: "Cargando dispositivos emparejados (sin buscar)...",
    status_connecting: "Conectando y leyendo estado y firmware...",
    status_refreshing: "Actualizando estado y firmware...",
    status_disconnecting: "Desconectando...",
    status_applying:
        "Aplicando {}={} tras consulta previa, verificando confirmación y lectura...",
    status_paired_empty:
        "Sin dispositivos emparejados. Empareja en los ajustes Bluetooth primero.",
    status_paired_pick:
        "Elige el dispositivo y pulsa Enter. Elegir no conecta.",
    status_read_ok: "Estado leído. Los ajustes mostrados vienen del dispositivo.",
    status_firmware_note: "Ajustes leídos; firmware no disponible: {}",
    status_blocked_note: " Aún se exige actualizar (r) para desbloquear escrituras.",
    status_verified: "Verificado {}={} con confirmación y lectura coincidente.",
    status_disconnected: "Desconectado. Sin reconexión automática.",
    status_quit_confirm: "Cancelación pedida. Una escritura puede haberse aplicado. Pulsa q o Enter para salir sin esperar; Esc para quedarse.",
    status_cancel_busy: "Cancelación pedida; esperando la etapa actual. Una escritura puede haberse aplicado. Se omitirán las etapas siguientes.",
    status_address_saved: "Dirección guardada. Pulsa c para conectar.",
    status_bad_address:
        "Escribe seis octetos hexadecimales con dos puntos, p. ej. AA:BB:CC:DD:EE:FF.",
    status_model_on:
        "Protocolo Studio Pro HP206 elegido por ti, sin verificar identidad. Pulsa c para conectar.",
    status_device_picked:
        "Dirección elegida. Confirma el dispositivo y pulsa c para conectar.",
    status_device_bad: "El dispositivo tiene una dirección inválida; edítala a mano.",
    status_cancel_then_disconnect:
        "Cancelación pedida; se desconectará tras la etapa actual. Una escritura puede haberse aplicado.",
    status_must_disconnect: "Desconecta (d) antes de cambiar destino o protocolo.",
    status_must_connect: "Desconectado; conecta (c) antes de actualizar.",
    status_proposed:
        "Solo propuesta. Enter revisa el cambio; y confirma una escritura. Esc descarta.",
    status_unavailable_setting:
        "Ajuste ausente en la lectura; la escritura está desactivada.",
    status_already_matches: "Ese valor ya coincide con la lectura; no hace falta escribir.",
    status_pick_first: "Usa Izquierda/Derecha para proponer un valor antes de aplicar.",
    status_disabled:
        "Cambios desactivados. Conecta y actualiza tras cualquier operación incierta.",
    status_lang: "Idioma: español. Press L for English.",
    err_gone: "Desconectado; conecta primero.",
    err_status_query: "Falló la consulta de estado: {}",
    err_paired_list: "Falló la lista de emparejados: {}",
    err_need_model: "Confirma el protocolo Studio Pro HP206 antes de conectar.",
    err_connected_already: "Desconecta antes de conectar otro destino.",
    err_connect: "Falló la conexión: {}",
    err_uncertain_blocked:
        "Escritura bloqueada tras una operación incierta. Actualiza primero.",
    err_preflight: "Falló la consulta previa; nada enviado: {}",
    err_preflight_unreadable: "La consulta no lee {}; nada enviado.",
    err_ack: "Quizá se envió; falló la confirmación: {}. Actualiza antes de escribir.",
    err_readback: "Confirmado, pero falló la lectura: {}. Actualiza antes de escribir.",
    err_readback_noack: "Enviado, pero falló la lectura: {}. Actualiza antes de escribir.",
    err_mismatch: "Confirmado, pero {} no coincide con {}. Actualiza antes de escribir.",
    err_mismatch_noack: "Enviado, pero {} no coincide con {}. Actualiza antes de escribir.",
    err_cancel_uncertain: "Etapas siguientes canceladas. Una escritura puede haberse aplicado; actualiza antes de escribir.",
    err_cancel: "Cancelado; no empezarán más etapas.",
    cli_help: "ugreen 0.1.0 — CLI Bluetooth no oficial para UGREEN Studio Pro\n\nUSO\n  ugreen [OPCIONES] COMANDO\n\nCOMANDOS SIN CONEXIÓN\n  help                          Mostrar esta ayuda\n  tui                           Abrir la interfaz de terminal opcional\n  models                        Mostrar compatibilidad y límites\n  commands                      Listar ajustes disponibles\n  decode HEX                    Validar/decodificar bytes capturados sin conexión\n  profile example               Mostrar un perfil de ejemplo\n\nCOMANDOS BLUETOOTH (empareja antes en el sistema)\n  discover                      Listar emparejados; sin búsqueda por radio\n  status                        Leer batería, firmware y ajustes\n  set CLAVE VALOR               Cambiar un ajuste y verificar lectura\n  profile export                Leer ajustes e imprimir un perfil reutilizable\n  profile apply ARCHIVO         Validar perfil y verificar cada ajuste\n\nOPCIONES (antes del COMANDO)\n  --address XX:XX:XX:XX:XX:XX    Dirección Bluetooth de destino explícita\n  --model studio-pro            Obligatorio para hardware; nunca se detecta solo\n  --channel 1                   Canal RFCOMM, 1–30 (1 por defecto)\n  --timeout 3                   Tiempo por operación en segundos, 1–60\n  --lang es                     Idioma de la interfaz: en o es (en por defecto)\n  --dry-run                     Mostrar paquetes sin acceder a Bluetooth\n  --help, -h                    Mostrar ayuda\n  --version, -V                 Mostrar versión\n\nEJEMPLOS\n  ugreen discover\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro status\n  ugreen --dry-run set anc ultra\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro set eq bass\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro profile export > mi-perfil.conf\n  ugreen --dry-run profile apply mi-perfil.conf\n\nProtocolo Studio Pro HP206 verificado en firmware 0.2.5 (una unidad);\nHiTune Max5c usa otros identificadores de comando. Sin firmware, restablecimiento,\nescritura libre ni localización. Los perfiles pueden aplicarse en parte.\n",
    cli_models: "studio-pro: protocolo basado en el código de referencia UGREEN Studio Pro HP206 y su captura\nEstado de prueba de hardware: VERIFICADO en firmware 0.2.5 (Linux, canal 1; ver docs/verification.md)\nHiTune Max5c: NO COMPATIBLE (otros identificadores de comando)\nOtros modelos/firmware UGREEN: SIN VERIFICAR",
    cli_commands: "anc: off|ultra|general|gentle|adaptive|ambient\neq: classic|jazz|electronic|pop|classical|rock|bass|treble\ngame, spatial, dual, wind: on|off\nprompts: voice|beeps\nvolume-up-action, volume-down-action: none|next|previous",
    cli_banner: "Usando protocolo Studio Pro HP206 en {}, canal RFCOMM {}",
    cli_need_model:
        "pasa --model studio-pro para elegir explícitamente este protocolo específico",
    cli_need_address: "falta --address; usa discover para listar emparejados",
    cli_connect_fail: "Falló la conexión Bluetooth: {}. Revisa emparejamiento, encendido, adaptador y si otra app usa el canal de control",
    cli_preflight_fail: "falló la consulta previa; ajustes no enviados: {}",
    cli_verified: "Verificado {}={}",
    cli_apply_fail: "{} de {} ajustes verificados antes de fallar en {}={}: {}. Los anteriores no se revierten",
    cli_model_note: "model=studio-pro (elegido por ti; sin verificar identidad)",
    cli_unknown_value: "desconocido",
    cli_firmware_fail: "ajustes leídos, pero falló el firmware: {}",
    cli_no_paired:
        "Sin dispositivos emparejados. Empareja los auriculares en Bluetooth primero.",
    cli_paired_fail: "falló la lista de emparejados: {}",
    cli_dry_run_writes: "Simulación: sin conexión Bluetooth ni escrituras",
    cli_bad_model: "solo --model studio-pro está implementado; Max5c es incompatible",
    cli_bad_option: "opción desconocida '{}'",
    cli_bad_channel: "el canal debe ser 1–30",
    cli_bad_timeout: "el tiempo debe ser 1–60 segundos",
    cli_need_value: "{} necesita un valor",
    cli_unknown_command: "comando desconocido '{}'; ejecuta ugreen --help",
    cli_bad_usage: "uso: {}",
    cli_bad_decode: "la captura no es una secuencia limpia: {} tramas, {} CRC rechazados, {} bytes descartados, {} pendientes",
    cli_no_tty: "La TUI exige terminales interactivos en entrada y salida; ejecuta ugreen --help para comandos CLI.",
    cli_no_tui_feature:
        "La TUI está desactivada en esta compilación solo-CLI; recompila sin --no-default-features",
    cli_tui_dry:
        "tui no admite --dry-run; usa --dry-run set CLAVE VALOR para ver paquetes",
    cli_bad_profile: "el perfil supera 16 KiB",
    cli_inflight_after_quit: "Una escritura en curso puede haberse aplicado. Actualiza el estado antes de más cambios.",
    set_bad_anc: "valor ANC inválido",
    set_bad_eq: "valor EQ inválido",
    set_bad_onoff: "se esperaba on u off",
    set_bad_prompts: "se esperaba voice o beeps",
    set_bad_button: "se esperaba none, next o previous",
    set_unknown: "ajuste desconocido '{}'\n{}",
    profile_big: "el perfil supera 16 KiB",
    profile_line: "la línea {} del perfil debe ser clave=valor",
    profile_dup: "clave duplicada '{}'",
    profile_model: "el modelo del perfil debe ser studio-pro",
    profile_need_model: "el perfil exige model=studio-pro",
    profile_empty: "el perfil no tiene ajustes",
    req_timeout: "sin respuesta válida antes del límite",
    req_closed: "conexión Bluetooth cerrada",
    req_rejected: "los auriculares rechazaron la instrucción 0x{:02X}",
    info_short: "respuesta de información muy corta: {} bytes (mínimo 8)",
    fw_short: "la respuesta de firmware no trae versión",
    set_ackfail: "quizá se envió, pero falló la confirmación: {}; consulta el estado antes de reintentar",
    set_rbfail: "confirmada la escritura, pero falló la lectura: {}",
    set_rbfail_noack: "escritura enviada, pero falló la lectura: {}",
    set_mismatch: "confirmada, pero {} no coincide con {}; consulta el estado",
    set_mismatch_noack: "enviada, pero {} no coincide con {}; consulta el estado",
};

pub fn txt(lang: Lang) -> &'static L {
    match lang {
        Lang::En => &EN,
        Lang::Es => &ES,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn english_pins_historical_strings() {
        let t = txt(Lang::En);
        assert!(t.status_startup.contains("No Bluetooth access"));
        assert_eq!(
            t.status_bad_address,
            "Enter six colon-separated hexadecimal octets, e.g. AA:BB:CC:DD:EE:FF."
        );
        assert!(t.status_verified.starts_with("Verified "));
        assert!(t.cli_help.contains("USAGE"));
        assert!(t.cli_models.contains("VERIFIED on retail firmware 0.2.5"));
    }
    #[test]
    fn spanish_differs_and_formats() {
        let t = txt(Lang::Es);
        assert!(t.status_startup.contains("Desconectado"));
        assert!(t.status_verified.starts_with("Verificado "));
        assert!(t.cli_help.contains("USO"));
        assert!(!t.status_verified.contains("acknowledgement"));
    }
    #[test]
    fn detect_reads_spanish_locale() {
        unsafe { env::set_var("UGREEN_TEST_LANG", "es_ES.UTF-8") };
        unsafe { env::set_var("LANGUAGE", "") };
        unsafe { env::set_var("LC_ALL", "") };
        unsafe { env::set_var("LC_MESSAGES", "") };
        unsafe { env::set_var("LANG", env::var("UGREEN_TEST_LANG").unwrap()) };
        assert_eq!(Lang::detect(), Lang::Es);
        unsafe { env::set_var("LANG", "en_US.UTF-8") };
        assert_eq!(Lang::detect(), Lang::En);
        unsafe { env::remove_var("LANG") };
        unsafe { env::remove_var("UGREEN_TEST_LANG") };
    }
    #[test]
    fn language_codes_roundtrip() {
        assert_eq!(Lang::from_code("en").unwrap(), Lang::En);
        assert_eq!(Lang::from_code("es").unwrap(), Lang::Es);
        assert_eq!(Lang::En.code(), "en");
        assert!(Lang::from_code("fr").is_err());
    }
}
