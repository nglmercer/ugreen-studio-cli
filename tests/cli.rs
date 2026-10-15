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

#[test]
fn static_analysis_is_offline_and_explicitly_unverified() {
    let packet = ugreen_cli::protocol::NotificationFrame {
        kind: 2,
        payload: [9, 1],
    };
    assert!(matches!(
        ugreen_cli::protocol::static_analysis::interpret_notification(&packet),
        ugreen_cli::protocol::static_analysis::Notification::Finder {
            left: 1,
            right: None
        }
    ));
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_ugreen"))
        .args(["analyze", "85 86 87 02 09 01"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("unverified on hardware"));
    assert!(stdout.contains("Finder"));
    for hex in [
        "DD EE FF 0F 01 04 01 FF 02 00 00 00",
        "85 86 87 09 01",
        "00 85 86 87 02 09 01",
    ] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_ugreen"))
            .args(["analyze", hex])
            .output()
            .unwrap();
        assert!(!output.status.success());
    }
}

#[test]
fn candidate_commands_cover_static_instruction_map_without_hardware() {
    let mac = "01:02:03:04:05:06";
    let cases: Vec<(Vec<&str>, u8, Vec<u8>)> = vec![
        (vec!["firmware"], 1, vec![0]),
        (vec!["id"], 2, vec![0]),
        (vec!["color"], 3, vec![0]),
        (vec!["peers"], 13, vec![0]),
        (vec!["codec"], 15, vec![0]),
        (vec!["box-mac"], 31, vec![0]),
        (vec!["factory-reset"], 14, vec![0]),
        (vec!["find", "left", "on"], 7, vec![1, 1]),
        (vec!["find", "right", "off"], 7, vec![0, 2]),
        (vec!["ldac", "on"], 11, vec![1]),
        (vec!["ldac", "on", mac], 11, vec![1, 1, 2, 3, 4, 5, 6]),
        (vec!["ldac", "off", mac], 11, vec![0; 7]),
        (vec!["high-quality", "off"], 30, vec![0]),
        (vec!["volume", "8"], 16, vec![8]),
        (vec!["channel", "mono"], 25, vec![0]),
        (vec!["channel", "stereo"], 25, vec![1]),
        (vec!["ai-control", "on"], 27, vec![1]),
        (
            vec!["buttons", "0,1,2,3,4,5,9,11"],
            10,
            vec![0, 1, 2, 3, 4, 5, 9, 11],
        ),
        (vec!["ai-actions", "6,7,8,0"], 24, vec![6, 7, 8, 0]),
        (vec!["sharing", "off"], 22, vec![0]),
        (
            vec!["sharing", "single"],
            22,
            vec![1, 255, 255, 255, 255, 255, 255, 255],
        ),
        (vec!["sharing", "2", mac], 22, vec![1, 2, 1, 2, 3, 4, 5, 6]),
    ];
    for (args, instruction, payload) in cases {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_ugreen"))
            .arg("candidate")
            .args(&args)
            .output()
            .unwrap();
        assert!(output.status.success(), "{args:?}: {:?}", output.stderr);
        let bytes =
            ugreen_cli::protocol::parse_hex(&String::from_utf8(output.stdout).unwrap()).unwrap();
        assert_eq!(
            bytes,
            ugreen_cli::protocol::request(instruction, &payload).unwrap()
        );
        assert!(output.stderr.is_empty());
    }
    for args in [
        vec![],
        vec!["peers", "extra"],
        vec!["volume", "256"],
        vec!["volume", "-1"],
        vec!["buttons", "0,0,0,0,0,0,0,10"],
        vec!["ai-actions", "0,0,0"],
        vec!["find", "both", "on"],
        vec!["ldac", "on", "bad"],
        vec!["channel", "surround"],
        vec!["sharing", "0", mac],
        vec!["19", "on"],
    ] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_ugreen"))
            .arg("candidate")
            .args(&args)
            .output()
            .unwrap();
        assert!(!output.status.success(), "{args:?}");
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn unverified_queries_refuse_before_connecting() {
    for command in ["peers", "codec", "id", "color", "box-mac"] {
        let output = run(&[
            "--address",
            "01:02:03:04:05:06",
            "--model",
            "studio-pro",
            command,
        ]);
        assert!(!output.status.success());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            error.contains("official-app capture required"),
            "{command}: {error}"
        );
        assert!(!error.contains("Connecting"));
        assert!(output.stdout.is_empty());
        assert!(!run(&[command, "extra"]).status.success());
    }
}

#[test]
fn analyze_peer_capture_reports_completion_and_escapes_names() {
    fn frame(payload: &[u8]) -> Vec<u8> {
        let mut bytes = vec![0xdd, 0xee, 0xff, 13, 1, payload.len() as u8];
        bytes.extend_from_slice(payload);
        bytes.extend_from_slice(&ugreen_cli::protocol::crc_ccitt(&bytes[3..]).to_le_bytes());
        bytes
    }
    let first = frame(&[0, 1, b'A', 0x1b, 1, 2, 3, 4, 5, 6]);
    let second = frame(&[0, 2, b'B', 6, 5, 4, 3, 2, 1]);
    for (bytes, complete) in [
        (first.clone(), false),
        ([first, second].concat(), true),
        (frame(&[0, 0]), true),
    ] {
        let output = run(&["analyze", &ugreen_cli::protocol::hex(&bytes)]);
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains(&format!("static_peer_list complete={complete}")));
        assert!(!stdout.contains('\u{1b}'));
    }
}

#[test]
fn controls_refuse_before_connect_and_dry_run_remains_offline() {
    for args in [
        vec!["ldac", "on"],
        vec!["volume", "8"],
        vec!["buttons", "0,0,0,0,0,0,0,0"],
        vec!["ai-actions", "0,0,0,0"],
        vec!["channel", "mono"],
        vec!["sharing", "single"],
        vec!["ai-control", "on"],
        vec!["factory-reset"],
        vec!["find", "left", "on"],
        vec!["high-quality", "off"],
    ] {
        let mut live = vec![
            "--address",
            "01:02:03:04:05:06",
            "--model",
            "studio-pro",
            "control",
        ];
        live.extend_from_slice(&args);
        let output = run(&live);
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("official-app capture required"));
        assert!(output.stdout.is_empty());
        let mut offline = vec!["--dry-run", "control"];
        offline.extend_from_slice(&args);
        let output = run(&offline);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        assert!(String::from_utf8_lossy(&output.stdout).contains("no Bluetooth access"));
    }
    for args in [
        vec!["control", "codec"],
        vec!["--dry-run", "control", "peers"],
        vec!["control", "volume", "256"],
    ] {
        let output = run(&args);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn finder_refuses_unverified_packets_before_audio_or_connection() {
    let target = if cfg!(target_os = "windows") {
        "0"
    } else {
        "sink"
    };
    let output = run(&["find", "left", "1", target]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("official-app capture required"));
    for args in [
        vec!["find", "both", "1", "sink"],
        vec!["find", "right", "0", "sink"],
        vec!["find", "right", "31", "sink"],
        vec!["find", "left", "1"],
    ] {
        assert!(!run(&args).status.success());
    }
}

#[test]
fn capability_json_contract_is_offline_and_does_not_authorize_writes() {
    let output = run(&["--json", "--model", "studio-pro", "capabilities"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["schema_version"], 1);
    assert_eq!(report["kind"], "capabilities");
    assert!(report["observed_firmware"].is_null());
    let features = report["features"].as_array().unwrap();
    assert_eq!(features.len(), ugreen_cli::features::FEATURES.len());
    for feature in features {
        assert_eq!(feature["operation_authorized"], false);
        assert!(["verified", "static_candidate", "unsupported"]
            .contains(&feature["evidence_status"].as_str().unwrap()));
    }
    assert!(!String::from_utf8_lossy(&output.stdout).contains("Connecting"));
}

#[test]
fn json_rejects_mutations_and_audio_validates_address_before_host_access() {
    let output = run(&["--json", "set", "game", "on"]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let output = run(&["--address", "invalid", "--json", "audio", "status"]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Bluetooth address"));
    let output = run(&["--address", "invalid", "host", "disconnect"]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
}

#[test]
fn new_command_paths_are_registered_without_removing_old_names() {
    for args in [
        &["audio", "--help"][..],
        &["host", "--help"][..],
        &["devices", "--help"][..],
        &["doctor", "--help"][..],
    ] {
        let output = run(args);
        assert!(output.status.success(), "{args:?}");
    }
}

#[cfg(target_os = "linux")]
#[test]
fn audio_output_json_is_stable_across_aliases_and_languages() {
    use std::os::unix::fs::PermissionsExt;
    let directory = std::env::temp_dir().join(format!("ugreen-audio-json-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let executable = directory.join("pw-dump");
    let dump = r#"[{"type":"PipeWire:Interface:Node","info":{"props":{"media.class":"Audio/Sink","node.name":"fixture_sink","node.description":"Fixture output"}}}]"#;
    std::fs::write(&executable, format!("#!/bin/sh\nprintf '%s\\n' '{dump}'\n")).unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();
    for lang in ["en", "es", "ja"] {
        for command in [
            vec!["--json", "audio", "devices"],
            vec!["--json", "audio-devices"],
        ] {
            let output = Command::new(env!("CARGO_BIN_EXE_ugreen"))
                .args(command)
                .env("UGREEN_LANG", lang)
                .env("PATH", &directory)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(output.stderr.is_empty());
            let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(
                value,
                serde_json::json!({"schema_version":1,"kind":"audio_devices","devices":[{"id":"fixture_sink","name":"Fixture output"}]})
            );
        }
    }
    std::fs::remove_dir_all(directory).unwrap();
}
