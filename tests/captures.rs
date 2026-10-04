//! Fixture-driven protocol tests. Every file under
//! `tests/fixtures/protocol/` is decoded through the same public
//! decoder the CLI uses, so a framing regression fails here with the
//! fixture's name. `#` lines are provenance comments, not bytes.

use ugreen_cli::{
    protocol::{self, Decoder, DecoderStats, IncomingFrame},
    settings::StudioProState,
};

fn fixture(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/protocol")
        .join(name);
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{name}: {e}"))
}

fn bytes(name: &str) -> Vec<u8> {
    let text: String = fixture(name)
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join(" ");
    protocol::parse_hex(&text).unwrap_or_else(|e| panic!("{name}: {e}"))
}

fn assert_clean(name: &str, decoder: &Decoder) {
    assert_eq!(
        decoder.stats,
        DecoderStats::default(),
        "{name}: decoder counted a defect"
    );
    assert_eq!(decoder.pending_bytes(), 0, "{name}: trailing partial frame");
}

#[test]
fn info_response_fixture_decodes_and_parses_state() {
    let mut decoder = Decoder::default();
    let frames = decoder.feed(&bytes("info-response.hex"));
    assert_clean("info-response", &decoder);
    assert_eq!(frames.len(), 1);
    let IncomingFrame::Response(frame) = &frames[0] else {
        panic!("expected a response, got {:?}", frames[0]);
    };
    assert_eq!(frame.instruction, protocol::INFO);
    assert!(frame.succeeded);
    let state = StudioProState::new(frame.payload.clone()).expect("30-byte payload parses");
    assert_eq!(state.battery(), Some(20));
    assert_eq!(state.value("anc").as_deref(), Some("off"));
    assert_eq!(state.value("eq").as_deref(), Some("classic"));
    assert_eq!(state.value("dual").as_deref(), Some("on"));
    assert_eq!(state.value("prompts").as_deref(), Some("beeps"));
    assert_eq!(state.value("spatial").as_deref(), Some("off"));
    assert_eq!(state.value("volume-up-action").as_deref(), Some("next"));
    assert_eq!(
        state.value("volume-down-action").as_deref(),
        Some("previous")
    );
    assert_eq!(state.value("wind").as_deref(), Some("off"));
}

#[test]
fn device_info_query_fixture_matches_the_verified_encoder() {
    assert_eq!(
        bytes("device-info-query.hex"),
        protocol::request(protocol::INFO, &[0]).expect("small payload")
    );
}

#[test]
fn notification_fixtures_decode_with_unknown_semantics() {
    for name in [
        "notification-spatial-echo.hex",
        "notification-link-change.hex",
    ] {
        let mut decoder = Decoder::default();
        let frames = decoder.feed(&bytes(name));
        assert_clean(name, &decoder);
        assert_eq!(frames.len(), 1);
        match &frames[0] {
            IncomingFrame::Notification(frame) => {
                assert_eq!(frame.kind, 0x02, "{name}");
                assert_eq!(
                    frame.event(),
                    protocol::HeadsetEvent::UnknownNotification(frame.clone()),
                    "{name}"
                );
            }
            other => panic!("{name}: expected a notification, got {other:?}"),
        }
    }
}

#[test]
fn session_fixture_keeps_notifications_from_disturbing_responses() {
    let mut decoder = Decoder::default();
    let frames = decoder.feed(&bytes("session-with-notifications.hex"));
    assert_clean("session-with-notifications", &decoder);
    assert_eq!(frames.len(), 3);
    assert!(matches!(frames[0], IncomingFrame::Response(_)));
    assert!(matches!(frames[1], IncomingFrame::Notification(_)));
    let IncomingFrame::Response(second) = &frames[2] else {
        panic!("expected a response, got {:?}", frames[2]);
    };
    assert_eq!(second.instruction, protocol::INFO);
    assert!(second.succeeded);
}

#[test]
fn request_layout_on_rx_fixture_is_counted_as_unknown() {
    let mut decoder = Decoder::default();
    let frames = decoder.feed(&bytes("request-layout-on-rx.hex"));
    assert_eq!(frames.len(), 1);
    let IncomingFrame::Unknown(frame) = &frames[0] else {
        panic!("expected an unknown frame, got {:?}", frames[0]);
    };
    assert_eq!(frame.instruction, 0x11);
    assert_eq!(frame.payload, [0xAB, 0xCD]);
    assert_eq!(decoder.stats.unknown_frames, 1);
    assert_eq!(decoder.stats.discarded_bytes, 0);
    assert_eq!(decoder.stats.response_crc_failures, 0);
    assert_eq!(decoder.pending_bytes(), 0);
}

#[test]
fn every_fixture_has_provenance_comments() {
    for name in [
        "info-response.hex",
        "device-info-query.hex",
        "notification-spatial-echo.hex",
        "notification-link-change.hex",
        "session-with-notifications.hex",
        "request-layout-on-rx.hex",
    ] {
        let text = fixture(name);
        assert!(
            text.lines().any(|line| line.trim_start().starts_with('#')),
            "{name}: fixtures must document where the bytes came from"
        );
    }
}
