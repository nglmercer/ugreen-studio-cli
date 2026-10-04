use std::process::Command;
fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ugreen"))
        .args(args)
        .env("UGREEN_LANG", "en")
        .output()
        .unwrap()
}
fn run_lang(lang: &str, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ugreen"))
        .args(args)
        .env("UGREEN_LANG", lang)
        .output()
        .unwrap()
}
#[test]
fn help_is_offline() {
    let o = run(&["--help"]);
    assert!(o.status.success());
    assert!(String::from_utf8_lossy(&o.stdout).contains("Studio Pro"));
}
#[test]
fn help_in_spanish() {
    let o = run_lang("es", &["--help"]);
    assert!(o.status.success());
    assert!(String::from_utf8_lossy(&o.stdout).contains("USO"));
    assert!(!String::from_utf8_lossy(&o.stdout).contains("USAGE"));
}
#[test]
fn lang_flag_beats_environment() {
    let o = run_lang("es", &["--lang", "en", "--help"]);
    assert!(o.status.success());
    assert!(String::from_utf8_lossy(&o.stdout).contains("USAGE"));
}
#[test]
fn dry_run_packet() {
    let o = run(&["--dry-run", "set", "game", "on"]);
    assert!(o.status.success());
    assert!(String::from_utf8_lossy(&o.stdout).contains("AA BB CC 08 01 01"));
}
#[test]
fn unknown_setting_fails_before_connect() {
    let o = run(&["set", "firmware", "update"]);
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("unknown setting"));
}
#[test]
fn unknown_setting_error_in_spanish() {
    let o = run_lang("es", &["set", "firmware", "update"]);
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("desconocido"));
}
#[test]
fn model_required_before_hardware() {
    let o = run(&["status"]);
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("--model studio-pro"));
}
#[test]
fn extra_args_rejected() {
    assert!(!run(&["commands", "surprise"]).status.success());
}
#[test]
fn decode_real_capture() {
    // The captured response lives as a fixture so the CLI contract and
    // the decoder tests share one set of bytes.
    let fixture = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/protocol/info-response.hex"),
    )
    .unwrap();
    let hex = fixture
        .lines()
        .find(|line| !line.trim_start().starts_with('#'))
        .unwrap()
        .trim();
    let o = run(&["decode", hex]);
    assert!(o.status.success());
    assert!(String::from_utf8_lossy(&o.stdout).contains("crc=valid"));
}
#[test]
fn decode_bad_crc_fails() {
    assert!(!run(&["decode", "DD EE FF 04 01 00 00 00"]).status.success());
}

#[test]
fn no_arguments_with_piped_output_prints_help() {
    let o = run(&[]);
    assert!(o.status.success());
    assert!(String::from_utf8_lossy(&o.stdout).contains("USAGE"));
    assert!(!o.stdout.contains(&0x1b));
}

#[test]
fn help_lists_the_capture_command() {
    for lang in ["en", "es", "ja"] {
        let o = run_lang(lang, &["--help"]);
        let text = String::from_utf8_lossy(&o.stdout).to_string();
        assert!(text.contains("capture"), "{lang}: capture missing");
        assert!(text.contains("discover"), "{lang}: discover missing");
    }
}

#[test]
fn capture_requires_model_before_connecting() {
    let o = run(&["capture"]);
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("--model studio-pro"));
}

#[test]
fn explicit_tui_without_a_terminal_fails_cleanly() {
    let o = run(&["tui"]);
    assert!(!o.status.success());
    assert!(!o.stdout.contains(&0x1b));
    assert!(!o.stderr.is_empty());
}

#[test]
fn unsupported_codec_fails_offline() {
    let o = run(&["--dry-run", "set", "codec", "ldac"]);
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("unknown setting"));
}

#[test]
fn dry_run_tui_is_rejected_without_terminal_setup() {
    let o = run(&["--dry-run", "tui"]);
    assert!(!o.status.success());
    assert!(!o.stdout.contains(&0x1b));
}
