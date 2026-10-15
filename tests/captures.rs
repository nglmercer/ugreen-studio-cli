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

#[test]
fn static_notification_mapping_uses_apk_event_offset() {
    use ugreen_cli::protocol::{
        static_analysis::{interpret_notification, Notification},
        NotificationFrame,
    };
    assert_eq!(
        interpret_notification(&NotificationFrame {
            kind: 2,
            payload: [10, 1]
        }),
        Notification::Spatial(1)
    );
    assert_eq!(
        interpret_notification(&NotificationFrame {
            kind: 2,
            payload: [5, 0]
        }),
        Notification::Eq(0)
    );
    assert_eq!(
        interpret_notification(&NotificationFrame {
            kind: 2,
            payload: [1, 90]
        }),
        Notification::Battery {
            left: 90,
            right: None,
            case: None
        }
    );
}

/// Candidate fixtures are synthetic (docs/reverse-engineering.md),
/// so these checks only prove framing, checksums and offline
/// parsing — never hardware behaviour.
#[test]
fn candidate_request_fixtures_match_the_offline_encoder() {
    for (name, instruction, payload) in [
        ("peer-query-request.hex", 0x0D, vec![0x00]),
        ("codec-query-request.hex", 0x0F, vec![0x00]),
        ("box-mac-query-request.hex", 0x1F, vec![0x00]),
        ("factory-reset-request.hex", 0x0E, vec![0x00]),
    ] {
        assert_eq!(
            bytes(name),
            protocol::request(instruction, &payload).expect("small payload"),
            "{name}"
        );
    }
    assert_eq!(
        bytes("find-start-left-request.hex"),
        protocol::request(0x07, &[0x01, 0x01]).expect("small payload")
    );
    assert_eq!(
        bytes("find-start-right-request.hex"),
        protocol::request(0x07, &[0x01, 0x02]).expect("small payload")
    );
}

#[test]
fn candidate_response_fixtures_decode_cleanly() {
    for name in [
        "peer-response-single.hex",
        "peer-response-second.hex",
        "codec-response.hex",
    ] {
        let mut decoder = Decoder::default();
        let frames = decoder.feed(&bytes(name));
        assert_clean(name, &decoder);
        assert_eq!(frames.len(), 1, "{name}");
        let IncomingFrame::Response(frame) = &frames[0] else {
            panic!("{name}: expected a response, got {:?}", frames[0]);
        };
        assert!(frame.succeeded, "{name}");
    }
}

#[test]
fn candidate_peer_responses_parse_into_peer_updates() {
    use ugreen_cli::protocol::static_analysis::{PeerList, PeerUpdate};
    let mut decoder = Decoder::default();
    let frames = decoder.feed(&bytes("peer-response-single.hex"));
    let IncomingFrame::Response(single) = &frames[0] else {
        panic!("expected a response");
    };
    let mut list = PeerList::default();
    list.apply(PeerUpdate::parse(&single.payload).expect("single-peer payload"))
        .expect("single-peer update");
    let peer = list.slots[0].as_ref().expect("peer 1 present");
    assert_eq!(peer.index, 1);
    assert_eq!(peer.name, "My Phone");
    assert_eq!(peer.address.octets(), [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
    // flag=01 with index=01 marks peer 2 absent and completes the list.
    assert!(list.complete);
    assert!(list.slots[1].is_none());

    let mut decoder = Decoder::default();
    let frames = decoder.feed(&bytes("peer-response-second.hex"));
    let IncomingFrame::Response(second) = &frames[0] else {
        panic!("expected a response");
    };
    let mut list = PeerList::default();
    list.apply(PeerUpdate::parse(&second.payload).expect("second-peer payload"))
        .expect("second-peer update");
    let peer = list.slots[1].as_ref().expect("peer 2 present");
    assert_eq!(peer.index, 2);
    assert_eq!(peer.name, "Work Laptop");
    assert_eq!(peer.address.octets(), [0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);
    // flag=00 with a lone peer 2 does NOT complete the list.
    assert!(!list.complete);
    assert!(list.slots[0].is_none());
}

#[test]
fn candidate_finder_notification_uses_the_event_code_offset() {
    let mut decoder = Decoder::default();
    let frames = decoder.feed(&bytes("notification-finder-state.hex"));
    assert_clean("notification-finder-state", &decoder);
    assert_eq!(frames.len(), 1);
    let IncomingFrame::Notification(frame) = &frames[0] else {
        panic!("expected a notification, got {:?}", frames[0]);
    };
    // Byte 3 is the envelope (0x02 in every capture); the
    // event code is payload[0] (frame byte 5), per the APK.
    assert_eq!(frame.kind, 0x02);
    assert_eq!(frame.payload, [0x09, 0x01]);
    assert_eq!(
        ugreen_cli::protocol::static_analysis::describe_notification_event(frame.payload[0]),
        Some("headphone-finder")
    );
}

#[test]
fn live_studio_pro_fw025_core_capture_remains_crc_valid_and_scoped_to_reads() {
    let mut decoder = Decoder::default();
    let frames = decoder.feed(&bytes("studio-pro-fw025-core-read-20261014.hex"));
    assert_clean("studio-pro-fw025-core-read-20261014", &decoder);
    assert_eq!(frames.len(), 2);
    let IncomingFrame::Response(info) = &frames[0] else {
        panic!("expected device info");
    };
    let IncomingFrame::Response(firmware) = &frames[1] else {
        panic!("expected firmware");
    };
    assert_eq!(info.instruction, protocol::INFO);
    assert_eq!(firmware.instruction, protocol::FIRMWARE);
    assert!(info.succeeded && firmware.succeeded);
    assert_eq!(
        ugreen_cli::settings::firmware(&firmware.payload).as_deref(),
        Some("0.2.5")
    );
    let observed = StudioProState::new(info.payload.clone()).unwrap();
    assert_eq!(observed.battery(), Some(50));
    assert_eq!(observed.value("dual").as_deref(), Some("on"));
    assert!(
        ugreen_cli::multipoint::MultipointState::from_device_info(&observed)
            .peers
            .is_none()
    );
}
