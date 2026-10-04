//! User-facing text, one catalog per language. Each language
//! module starts from the English catalog and overrides only
//! the strings it translates; anything left in English is a
//! deliberate fallback, not a missing entry. Protocol values,
//! profile file syntax, `key=value` output labels and wire
//! bytes are never translated: only the prose around them.
//! `Lang::En` must reproduce the historical strings exactly;
//! several tests pin them.
mod de;
mod en;
mod es;
mod fr;
mod it;
mod ja;
mod ko;
mod nl;
mod pt;
mod ru;
mod zh;

use std::{env, sync::OnceLock};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Lang {
    #[default]
    En,
    Es,
    Pt,
    De,
    Fr,
    It,
    Nl,
    Ru,
    Zh,
    Ja,
    Ko,
}

impl Lang {
    /// Every supported language, in cycle order for the `L` key.
    pub const ALL: [Lang; 11] = [
        Lang::En,
        Lang::Es,
        Lang::Pt,
        Lang::De,
        Lang::Fr,
        Lang::It,
        Lang::Nl,
        Lang::Ru,
        Lang::Zh,
        Lang::Ja,
        Lang::Ko,
    ];

    pub fn from_code(code: &str) -> Result<Self, String> {
        match code.trim().to_ascii_lowercase().as_str() {
            "en" => Ok(Self::En),
            "es" => Ok(Self::Es),
            "pt" => Ok(Self::Pt),
            "de" => Ok(Self::De),
            "fr" => Ok(Self::Fr),
            "it" => Ok(Self::It),
            "nl" => Ok(Self::Nl),
            "ru" => Ok(Self::Ru),
            "zh" | "zh-cn" | "zh_cn" | "zh-hans" | "zh_hans" => Ok(Self::Zh),
            "ja" => Ok(Self::Ja),
            "ko" => Ok(Self::Ko),
            _ => Err(format!(
                "invalid language '{code}'; expected en, es, pt, de, fr, it, nl, ru, zh, ja or ko"
            )),
        }
    }
    pub fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Es => "es",
            Self::Pt => "pt",
            Self::De => "de",
            Self::Fr => "fr",
            Self::It => "it",
            Self::Nl => "nl",
            Self::Ru => "ru",
            Self::Zh => "zh",
            Self::Ja => "ja",
            Self::Ko => "ko",
        }
    }
    /// Native name, shown in the language status line.
    pub fn name(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Es => "Español",
            Self::Pt => "Português",
            Self::De => "Deutsch",
            Self::Fr => "Français",
            Self::It => "Italiano",
            Self::Nl => "Nederlands",
            Self::Ru => "Русский",
            Self::Zh => "中文",
            Self::Ja => "日本語",
            Self::Ko => "한국어",
        }
    }
    /// OS locale guess. `Config::default()` stays English so
    /// tests never depend on the environment; the CLI applies
    /// this only when `--lang` is absent. `UGREEN_LANG` wins
    /// when set, so tests and scripts can pin the language
    /// regardless of locale.
    pub fn detect() -> Self {
        if let Ok(value) = env::var("UGREEN_LANG") {
            if let Ok(lang) = Self::from_code(&value) {
                return lang;
            }
        }
        for key in ["LANGUAGE", "LC_ALL", "LC_MESSAGES", "LANG"] {
            if let Ok(value) = env::var(key) {
                let lower = value.to_ascii_lowercase();
                let tag = lower.split(['.', '@', ':']).next().unwrap_or("");
                let base = tag.split(['-', '_']).next().unwrap_or("");
                if let Ok(lang) = Self::from_code(base) {
                    return lang;
                }
            }
        }
        Self::En
    }
}

/// Every user-facing prose string, in every language. `{}`
/// placeholders keep the same argument order in all translations.
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

/// Catalog for a language, built once and shared.
pub fn txt(lang: Lang) -> &'static L {
    static EN: OnceLock<L> = OnceLock::new();
    static ES: OnceLock<L> = OnceLock::new();
    static PT: OnceLock<L> = OnceLock::new();
    static DE: OnceLock<L> = OnceLock::new();
    static FR: OnceLock<L> = OnceLock::new();
    static IT: OnceLock<L> = OnceLock::new();
    static NL: OnceLock<L> = OnceLock::new();
    static RU: OnceLock<L> = OnceLock::new();
    static ZH: OnceLock<L> = OnceLock::new();
    static JA: OnceLock<L> = OnceLock::new();
    static KO: OnceLock<L> = OnceLock::new();
    match lang {
        Lang::En => EN.get_or_init(en::catalog),
        Lang::Es => ES.get_or_init(es::catalog),
        Lang::Pt => PT.get_or_init(pt::catalog),
        Lang::De => DE.get_or_init(de::catalog),
        Lang::Fr => FR.get_or_init(fr::catalog),
        Lang::It => IT.get_or_init(it::catalog),
        Lang::Nl => NL.get_or_init(nl::catalog),
        Lang::Ru => RU.get_or_init(ru::catalog),
        Lang::Zh => ZH.get_or_init(zh::catalog),
        Lang::Ja => JA.get_or_init(ja::catalog),
        Lang::Ko => KO.get_or_init(ko::catalog),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn placeholders(text: &str) -> usize {
        text.matches("{}").count()
    }

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
    fn every_language_keeps_placeholder_counts() {
        // A translation that drops or duplicates a `{}` would
        // panic or misformat at runtime; catch it here for all
        // scalar fields that carry arguments.
        type Field<'a> = (fn(&'a L) -> &'static str, &'static str);
        let checks: &[Field] = &[
            (|l| l.battery_pct, "battery_pct"),
            (|l| l.snapshot_ago, "snapshot_ago"),
            (|l| l.rfcomm_line, "rfcomm_line"),
            (|l| l.proposed, "proposed"),
            (|l| l.resize_to, "resize_to"),
            (|l| l.status_applying, "status_applying"),
            (|l| l.status_firmware_note, "status_firmware_note"),
            (|l| l.status_verified, "status_verified"),
            (|l| l.status_lang, "status_lang"),
            (|l| l.err_status_query, "err_status_query"),
            (|l| l.err_paired_list, "err_paired_list"),
            (|l| l.err_connect, "err_connect"),
            (|l| l.err_preflight, "err_preflight"),
            (|l| l.err_preflight_unreadable, "err_preflight_unreadable"),
            (|l| l.err_ack, "err_ack"),
            (|l| l.err_readback, "err_readback"),
            (|l| l.err_readback_noack, "err_readback_noack"),
            (|l| l.err_mismatch, "err_mismatch"),
            (|l| l.err_mismatch_noack, "err_mismatch_noack"),
            (|l| l.cli_banner, "cli_banner"),
            (|l| l.cli_connect_fail, "cli_connect_fail"),
            (|l| l.cli_preflight_fail, "cli_preflight_fail"),
            (|l| l.cli_verified, "cli_verified"),
            (|l| l.cli_apply_fail, "cli_apply_fail"),
            (|l| l.cli_need_value, "cli_need_value"),
            (|l| l.cli_bad_option, "cli_bad_option"),
            (|l| l.cli_bad_decode, "cli_bad_decode"),
            (|l| l.cli_unknown_command, "cli_unknown_command"),
            (|l| l.set_unknown, "set_unknown"),
            (|l| l.profile_line, "profile_line"),
            (|l| l.profile_dup, "profile_dup"),
            (|l| l.req_rejected, "req_rejected"),
            (|l| l.info_short, "info_short"),
            (|l| l.set_ackfail, "set_ackfail"),
            (|l| l.set_rbfail, "set_rbfail"),
            (|l| l.set_rbfail_noack, "set_rbfail_noack"),
            (|l| l.set_mismatch, "set_mismatch"),
            (|l| l.set_mismatch_noack, "set_mismatch_noack"),
        ];
        let en = txt(Lang::En);
        for lang in Lang::ALL {
            let t = txt(lang);
            for (get, name) in checks {
                assert_eq!(
                    placeholders(get(t)),
                    placeholders(get(en)),
                    "{:?} {}",
                    lang,
                    name
                );
            }
        }
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
        for lang in Lang::ALL {
            assert_eq!(Lang::from_code(lang.code()).unwrap(), lang);
            assert!(Lang::from_code(lang.name()).is_err());
        }
        assert_eq!(Lang::En.code(), "en");
        assert!(Lang::from_code("xx").is_err());
        // Chinese accepts the common locale spellings.
        assert_eq!(Lang::from_code("zh-CN").unwrap(), Lang::Zh);
        assert_eq!(Lang::from_code("zh_Hans").unwrap(), Lang::Zh);
    }
    #[test]
    fn new_languages_translate_their_titles() {
        assert!(txt(Lang::Pt).status_startup.contains("Desconectado"));
        assert!(txt(Lang::De).status_startup.contains("Getrennt"));
        assert!(txt(Lang::Fr).status_startup.contains("Déconnecté"));
        assert!(txt(Lang::It).status_startup.contains("Scollegato"));
        assert!(txt(Lang::Nl).status_startup.contains("Niet verbonden"));
        assert!(txt(Lang::Ru).status_startup.contains("Не подключено"));
        assert!(txt(Lang::Zh).status_startup.contains("未连接"));
        assert!(txt(Lang::Ja).status_startup.contains("未接続"));
        assert!(txt(Lang::Ko).status_startup.contains("연결 안 됨"));
    }
}
