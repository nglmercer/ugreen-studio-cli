use std::process::Command;
fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ugreen"))
        .args(args)
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
    let o=run(&["decode","DD EE FF 04 01 1E 14 FF FF A0 00 01 00 00 08 0B 00 07 00 00 00 00 02 00 00 09 00 00 04 05 00 00 00 0C 0D 0E D0 E3"]);
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
