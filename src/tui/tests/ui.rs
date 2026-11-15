//! Interface state, input and rendering. Nothing here touches Bluetooth:
//! requests are inspected as values and replies are constructed by hand.
use super::*;
use crate::{
    features::{FeatureId, Operation, ResultData},
    i18n::Lang,
    protocol::static_analysis::{Peer, PeerList},
};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

fn click(app: &mut App, x: u16, y: u16, width: u16, height: u16) -> Intent {
    app.mouse(
        MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: x,
            row: y,
            modifiers: KeyModifiers::NONE,
        },
        width,
        height,
    )
}
fn is_none(intent: &Intent) -> bool {
    matches!(intent, Intent::None)
}
fn request(intent: Intent) -> Action {
    match intent {
        Intent::Request(action) => action,
        _ => panic!("expected a request"),
    }
}
fn sized(mut app: App, width: u16, height: u16) -> App {
    app.width = width;
    app.height = height;
    app
}
fn peers() -> PeerList {
    PeerList {
        slots: [
            Some(Peer {
                index: 1,
                name: "Phone".into(),
                address: BluetoothAddress::parse("11:22:33:44:55:66").unwrap(),
            }),
            None,
        ],
        observed: true,
        complete: true,
    }
}
fn feature_reply(app: &mut App, id: u64, result: ResultData) {
    complete(app, id, Ok(Output::Feature(result)), true);
}
fn result(text: &str) -> ResultData {
    ResultData {
        text: text.into(),
        info: None,
        codecs: None,
        peers: None,
        reset: false,
        certain: true,
    }
}
/// Move the feature browser to an entry by registry id.
fn select_feature(app: &mut App, id: FeatureId) {
    let position = app
        .feature_list()
        .iter()
        .position(|spec| spec.id == id)
        .expect("feature listed in this browser");
    app.browser_mut().index = position;
}

// ----- startup and target selection ---------------------------------------

#[test]
fn startup_is_offline_unless_a_confirmed_target_autoconnects() {
    let mut app = App::new(&Config::default());
    assert!(is_none(&app.startup()));
    assert_eq!(app.page, Page::Home);

    let mut cfg = config();
    cfg.skip_autoload = false;
    let mut app = App::new(&cfg);
    assert!(matches!(request(app.startup()), Action::Connect { .. }));

    cfg.autoconnect = false;
    let mut app = App::new(&cfg);
    assert!(is_none(&app.startup()));

    let cfg = Config {
        skip_autoload: false,
        ..Config::default()
    };
    let mut app = App::new(&cfg);
    assert!(matches!(request(app.startup()), Action::Paired));
    assert_eq!(app.page, Page::Devices);
}

#[test]
fn connect_requires_address_then_protocol_confirmation() {
    let mut app = App::new(&Config::default());
    assert!(is_none(&app.key(ch('c'))));
    assert_eq!(app.status, app.tt().need_address);
    app.address = "AA:BB:CC:DD:EE:FF".into();
    assert!(is_none(&app.key(ch('c'))));
    assert!(matches!(app.modal, Some(Modal::Model)));
    assert!(is_none(&app.key(ch('n'))));
    assert!(app.modal.is_none() && !app.model_confirmed);
    app.key(ch('m'));
    let action = request(app.key(ch('y')));
    assert!(matches!(
        action,
        Action::Connect {
            model_confirmed: true,
            ..
        }
    ));
}

#[test]
fn address_editor_is_local_bounded_validated_and_resets_confirmation() {
    let mut app = App::new(&config());
    app.key(ch('a'));
    app.key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL));
    for c in "zz11:22:33:44:55:66:77".chars() {
        assert!(is_none(&app.key(ch(c))));
    }
    let Some(Modal::Address { text }) = &app.modal else {
        panic!("editor stays open while typing");
    };
    assert_eq!(text, "11:22:33:44:55:66");
    assert!(is_none(&app.key(key(KeyCode::Enter))));
    assert_eq!(app.address, "11:22:33:44:55:66");
    // A different target is unconfirmed unless the registry knows it.
    assert!(!app.model_confirmed);
    app.key(ch('a'));
    app.key(key(KeyCode::Backspace));
    app.key(key(KeyCode::Enter));
    assert!(matches!(app.modal, Some(Modal::Address { .. })));
    assert_eq!(app.status, app.t().status_bad_address);
    app.key(key(KeyCode::Esc));
    assert_eq!(app.address, "11:22:33:44:55:66");
}

#[test]
fn device_page_picks_confirm_protocol_once_and_lock_while_connected() {
    let mut app = App::new(&Config::default());
    let action = request(app.navigate(Page::Devices));
    assert!(matches!(action, Action::Paired));
    app.accepted(1, action);
    let mut known = device("11:22:33:44:55:66", "Known");
    known.model = Some(crate::models::Model::StudioPro);
    complete(
        &mut app,
        1,
        Ok(Output::Paired(vec![
            device("AA:BB:CC:DD:EE:FF", "New"),
            known,
        ])),
        false,
    );
    let index = |app: &App, addr: &str| {
        app.devices
            .as_ref()
            .unwrap()
            .iter()
            .position(|d| d.address.as_str() == addr)
            .unwrap()
    };
    app.device_index = index(&app, "AA:BB:CC:DD:EE:FF");
    assert!(is_none(&app.key(key(KeyCode::Enter))));
    assert!(matches!(app.modal, Some(Modal::Model)));
    app.key(key(KeyCode::Esc));
    app.device_index = index(&app, "11:22:33:44:55:66");
    assert!(matches!(
        request(app.key(key(KeyCode::Enter))),
        Action::Connect { .. }
    ));
    assert_eq!(app.address, "11:22:33:44:55:66");

    let mut app = ready();
    app.devices = Some(vec![device("11:22:33:44:55:66", "Other")]);
    app.page = Page::Devices;
    app.key(key(KeyCode::Enter));
    assert_eq!(app.address, "AA:BB:CC:DD:EE:FF");
    assert_eq!(app.status, app.tt().devices_locked);
    assert!(is_none(&app.key(ch('a'))));
    assert!(app.modal.is_none());
}

#[test]
fn empty_and_unloaded_device_lists_are_safe() {
    let mut app = App::new(&Config::default());
    app.page = Page::Devices;
    for code in [
        KeyCode::Up,
        KeyCode::Down,
        KeyCode::End,
        KeyCode::Enter,
        KeyCode::Char('h'),
    ] {
        assert!(is_none(&app.key(key(code))));
    }
    app.devices = Some(vec![]);
    for code in [KeyCode::Down, KeyCode::Enter, KeyCode::Char('H')] {
        assert!(is_none(&app.key(key(code))));
    }
    assert!(render(&app, 56, 20).contains(app.t().paired_empty.split('.').next().unwrap()));
}

// ----- settings writes ------------------------------------------------------

#[test]
fn proposals_need_a_review_and_escape_discards_them() {
    let mut app = ready();
    app.navigate(Page::Settings);
    assert!(is_none(&app.key(key(KeyCode::Right))));
    assert!(app.proposals[0].is_some());
    assert!(is_none(&app.key(key(KeyCode::Enter))));
    assert!(matches!(app.modal, Some(Modal::Review(_))));
    let action = request(app.key(key(KeyCode::Enter)));
    assert!(matches!(action, Action::Set(ref s) if s.key == "anc"));

    let mut app = ready();
    app.navigate(Page::Settings);
    app.key(key(KeyCode::Right));
    app.key(key(KeyCode::Esc));
    assert!(app.proposals[0].is_none());
    assert_eq!(app.page, Page::Settings);
    app.key(key(KeyCode::Esc));
    assert_eq!(app.page, Page::Home);
}

#[test]
fn value_list_selects_then_reviews_and_current_value_needs_no_write() {
    let mut app = ready();
    app.navigate(Page::Settings);
    app.key(key(KeyCode::Enter));
    let Some(Modal::Values { index }) = app.modal else {
        panic!("Enter without proposal opens the value list");
    };
    // The cursor starts on the device value: choosing it is not a write.
    assert!(is_none(&app.key(key(KeyCode::Enter))));
    assert!(app.modal.is_none());
    assert_eq!(app.status, app.t().status_already_matches);
    app.key(key(KeyCode::Enter));
    app.modal = Some(Modal::Values { index: index + 1 });
    app.key(key(KeyCode::Enter));
    assert!(matches!(app.modal, Some(Modal::Review(_))));
    app.key(key(KeyCode::Esc));
    assert!(app.modal.is_none());
}

#[test]
fn tabs_digits_and_arrows_select_settings_across_categories() {
    let mut app = ready();
    app.navigate(Page::Settings);
    app.key(key(KeyCode::Tab));
    assert_eq!(app.tab, 1);
    app.key(key(KeyCode::BackTab));
    app.key(ch('3'));
    assert_eq!(app.selected, super::super::app::categories()[0][2]);
    app.key(ch('9'));
    assert_eq!(app.selected, super::super::app::categories()[0][2]);
    for _ in 0..20 {
        app.key(key(KeyCode::Down));
    }
    assert_eq!(app.tab, super::super::app::categories().len() - 1);
    assert_eq!(app.selected, 8);
}

#[test]
fn writes_are_disabled_without_trusted_readback() {
    for setup in [
        |app: &mut App| app.firmware = Some("9.9.9".into()),
        |app: &mut App| app.stale = true,
        |app: &mut App| app.writes_blocked = true,
        |app: &mut App| app.info = None,
    ] {
        let mut app = ready();
        setup(&mut app);
        app.navigate(Page::Settings);
        assert!(is_none(&app.key(key(KeyCode::Right))));
        assert!(is_none(&app.key(key(KeyCode::Enter))));
        assert!(app.proposals.iter().all(Option::is_none));
        assert!(app.modal.is_none());
    }
}

#[test]
fn verified_write_replaces_readback_and_uncertain_write_blocks_until_refresh() {
    let mut app = ready();
    app.navigate(Page::Settings);
    app.key(key(KeyCode::Right));
    app.key(key(KeyCode::Enter));
    let action = {
        let intent = app_key(&mut app, KeyCode::Enter);
        accept(&mut app, 2, intent)
    };
    let Action::Set(setting) = action else {
        panic!()
    };
    assert!(app.stale);
    app.receive(Reply {
        id: 2,
        result: Err("ack failed".into()),
        connected: true,
        writes_blocked: true,
    });
    assert!(app.writes_blocked);
    assert!(is_none(&app.key(key(KeyCode::Right))));
    let refresh = {
        let intent = app.key(ch('r'));
        accept(&mut app, 3, intent)
    };
    assert!(matches!(refresh, Action::Refresh));
    complete(
        &mut app,
        3,
        Ok(Output::Verified {
            info: info(),
            setting,
        }),
        true,
    );
    assert!(!app.writes_blocked && !app.stale);
    assert!(app.status.contains("Verified"));
}
fn app_key(app: &mut App, code: KeyCode) -> Intent {
    app.key(key(code))
}

// ----- busy work, cancellation and quitting -----------------------------------

#[test]
fn busy_work_rejects_requests_but_allows_browsing_and_cancel() {
    let mut app = ready();
    app.accepted(5, Action::Refresh);
    assert!(app.working());
    assert!(is_none(&app.key(ch('c'))));
    assert!(is_none(&app.key(ch('a'))));
    assert!(app.modal.is_none());
    assert_eq!(app.status, app.tt().busy_rejected);
    assert!(is_none(&app.navigate(Page::Diagnostics)));
    assert_eq!(app.page, Page::Diagnostics);
    assert!(matches!(app.key(key(KeyCode::Esc)), Intent::Cancel));
    assert!(app.busy.is_some(), "cancel waits for the matching reply");
    complete(&mut app, 4, Err("old".into()), true);
    assert!(app.busy.is_some(), "stale replies are ignored");
    let cancelled = app.t().err_cancel.to_owned();
    complete(&mut app, 5, Err(cancelled), true);
    assert!(app.idle());
}

#[test]
fn disconnect_during_work_follows_the_completion() {
    let mut app = ready();
    app.accepted(5, Action::Refresh);
    assert!(matches!(app.key(ch('d')), Intent::Cancel));
    let follow = complete(&mut app, 5, Err("cancelled".into()), true);
    assert!(matches!(follow, Some(Action::Disconnect)));
}

#[test]
fn quit_is_immediate_unless_a_write_is_in_flight() {
    let mut app = ready();
    app.accepted(5, Action::Refresh);
    assert!(matches!(app.key(ch('q')), Intent::Quit));

    let mut app = ready();
    app.accepted(5, Action::Set(Setting::parse("game", "on").unwrap()));
    assert!(matches!(app.key(ch('q')), Intent::Cancel));
    assert!(app.quit_confirmation);
    assert!(is_none(&app.key(key(KeyCode::Esc))));
    assert!(!app.quit_confirmation);
    app.key(ch('q'));
    assert!(matches!(app.key(key(KeyCode::Enter)), Intent::Quit));

    let mut app = App::new(&config());
    app.key(ch('a'));
    assert!(is_none(&app.key(ch('q'))), "q is text in the editor");
    assert!(matches!(
        app.key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
        Intent::Quit
    ));
}

#[test]
fn key_releases_never_act() {
    let mut app = ready();
    let mut release = ch('q');
    release.kind = crossterm::event::KeyEventKind::Release;
    assert!(is_none(&app.key(release)));
}

#[test]
fn background_host_refresh_never_locks_input_or_damages_state() {
    let mut app = ready();
    app.accepted(
        9,
        Action::HostRefresh {
            address: app.address.clone(),
        },
    );
    assert!(app.background() && !app.working() && app.idle());
    app.navigate(Page::Settings);
    assert!(matches!(app.key(key(KeyCode::Right)), Intent::None));
    assert!(app.proposals[0].is_some(), "editing continues");
    let status = app.status.clone();
    // A failed refresh leaves status, snapshot and write state alone.
    complete(&mut app, 9, Err("bus error".into()), true);
    assert_eq!(app.status, status);
    assert!(!app.stale && app.info.is_some());
    // A request made during the refresh waits in the queue.
    app.accepted(
        10,
        Action::HostRefresh {
            address: app.address.clone(),
        },
    );
    app.queued = Some(Action::Refresh);
    assert!(app.working());
    assert!(matches!(app.key(ch('q')), Intent::Quit));
}

#[test]
fn worker_failure_disables_further_work() {
    let mut app = ready();
    app.accepted(5, Action::Set(Setting::parse("game", "on").unwrap()));
    app.worker_failed("stopped".into());
    assert!(!app.worker_alive && app.writes_blocked && !app.connected());
    assert!(is_none(&app.key(ch('c'))));
    assert!(is_none(&app.key(ch('r'))));
}

// ----- feature browsers -----------------------------------------------------------

#[test]
fn features_and_research_partition_the_registry() {
    let mut app = ready();
    app.page = Page::Features;
    let everyday: Vec<_> = app.feature_list().iter().map(|s| s.id).collect();
    app.page = Page::Research;
    let research: Vec<_> = app.feature_list().iter().map(|s| s.id).collect();
    assert_eq!(
        everyday.len() + research.len(),
        crate::features::FEATURES.len()
    );
    assert!(everyday.iter().all(|id| !research.contains(id)));
    for id in &research {
        let spec = crate::features::lookup(*id).unwrap();
        assert!(
            !app.enabled(spec),
            "{id:?} is listed as research but enabled"
        );
    }
    assert!(research.contains(&FeatureId::FactoryReset));
    assert!(everyday.contains(&FeatureId::OfflineDecode));
}

#[test]
fn research_entries_never_open_live_forms() {
    let mut app = sized(ready(), 120, 40);
    app.navigate(Page::Research);
    for _ in 0..app.feature_list().len() {
        assert!(is_none(&app.key(key(KeyCode::Enter))));
        assert!(app.modal.is_none());
        app.key(key(KeyCode::Down));
    }
}

#[test]
fn search_is_explicit_and_typed_letters_are_not_shortcuts() {
    let mut app = sized(ready(), 120, 40);
    app.navigate(Page::Features);
    app.key(ch('/'));
    for c in "codec q".chars() {
        assert!(is_none(&app.key(ch(c))));
    }
    assert_eq!(app.features.search, "codec q");
    app.key(key(KeyCode::Backspace));
    app.key(key(KeyCode::Backspace));
    app.key(key(KeyCode::Enter));
    assert!(!app.features.searching);
    assert!(app
        .feature_list()
        .iter()
        .any(|s| s.id == FeatureId::CodecStatus));
    assert!(app.feature_list().len() < 10);
    app.key(key(KeyCode::Esc));
    assert!(app.features.search.is_empty());
    assert_eq!(app.page, Page::Features);
}

#[test]
fn queries_and_controls_require_connection_and_go_through_review() {
    let mut app = sized(App::new(&config()), 120, 40);
    app.navigate(Page::Features);
    select_feature(&mut app, FeatureId::CodecStatus);
    assert!(is_none(&app.key(key(KeyCode::Enter))));
    assert_eq!(app.status, app.tt().need_connection);

    let mut app = sized(ready(), 120, 40);
    app.navigate(Page::Features);
    select_feature(&mut app, FeatureId::CodecStatus);
    let action = {
        let intent = app_key(&mut app, KeyCode::Enter);
        accept(&mut app, 2, intent)
    };
    assert!(matches!(action, Action::Feature(Operation::Codec)));
    let mut codec = result("codec ok");
    codec.codecs = Some([
        (0, crate::protocol::static_analysis::Codec::Ldac),
        (1, crate::protocol::static_analysis::Codec::Aac),
    ]);
    feature_reply(&mut app, 2, codec);
    assert!(app.codecs.is_some());
    assert!(matches!(app.last_result, Some((FeatureId::CodecStatus, _))));
    assert!(render(&app, 120, 40).contains("codec ok"));

    select_feature(&mut app, FeatureId::ChannelMode);
    app.key(key(KeyCode::Enter));
    assert!(matches!(app.modal, Some(Modal::Form(_))));
    app.key(key(KeyCode::Enter));
    assert!(
        matches!(app.modal, Some(Modal::Form(_))),
        "unset fields block review"
    );
    app.key(key(KeyCode::Right));
    app.key(key(KeyCode::Enter));
    assert!(matches!(app.modal, Some(Modal::Review(_))));
    app.key(key(KeyCode::Esc));
    assert!(
        matches!(app.modal, Some(Modal::Form(_))),
        "review returns to its form"
    );
    app.key(key(KeyCode::Enter));
    let action = request(app.key(key(KeyCode::Enter)));
    assert!(matches!(action, Action::Feature(ref op) if op.is_write()));
    // A write invalidates previous query results.
    app.accepted(3, action);
    assert!(app.codecs.is_none());
}

#[test]
fn verified_settings_in_the_browser_hand_over_to_settings() {
    let mut app = sized(ready(), 120, 40);
    app.navigate(Page::Features);
    select_feature(&mut app, FeatureId::DualConnection);
    assert!(is_none(&app.key(key(KeyCode::Enter))));
    assert_eq!(app.page, Page::Settings);
    assert_eq!(crate::settings::KEYS[app.selected], "dual");
}

#[test]
fn narrow_browser_opens_details_before_activating() {
    let mut app = sized(ready(), 60, 24);
    app.navigate(Page::Features);
    select_feature(&mut app, FeatureId::HeadphoneId);
    assert!(is_none(&app.key(key(KeyCode::Enter))));
    assert!(matches!(app.modal, Some(Modal::Details)));
    assert!(matches!(
        request(app.key(key(KeyCode::Enter))),
        Action::Feature(Operation::Id)
    ));
    assert!(app.modal.is_none());
}

#[test]
fn form_fields_cycle_type_and_prefill_peer_identity() {
    let mut app = sized(ready(), 120, 40);
    app.peers = Some(peers());
    app.navigate(Page::Features);
    select_feature(&mut app, FeatureId::AudioSharingPeer);
    app.key(key(KeyCode::Enter));
    app.key(key(KeyCode::Right));
    let Some(Modal::Form(form)) = &app.modal else {
        panic!("sharing with a peer has a form");
    };
    assert_eq!(form.drafts[0].as_deref(), Some("1"));
    assert_eq!(form.drafts[1].as_deref(), Some("11:22:33:44:55:66"));

    select_feature(&mut app, FeatureId::VolumeCommand);
    app.modal = None;
    app.key(key(KeyCode::Enter));
    for c in "2x55".chars() {
        app.key(ch(c));
    }
    let Some(Modal::Form(form)) = &app.modal else {
        panic!()
    };
    assert_eq!(form.drafts[0].as_deref(), Some("255"));
    app.key(key(KeyCode::Backspace));
    let Some(Modal::Form(form)) = &app.modal else {
        panic!()
    };
    assert_eq!(form.drafts[0].as_deref(), Some("25"));
}

#[test]
fn offline_tools_only_explain_their_command() {
    let mut app = sized(App::new(&Config::default()), 120, 40);
    app.navigate(Page::Features);
    select_feature(&mut app, FeatureId::OfflineDecode);
    assert!(is_none(&app.key(key(KeyCode::Enter))));
    assert!(app.status.contains("ugreen decode"));
}

// ----- observation pages ------------------------------------------------------------

#[test]
fn peers_page_queries_on_entry_and_reviews_targeted_close() {
    let mut app = ready();
    let action = {
        let intent = app.navigate(Page::Peers);
        accept(&mut app, 2, intent)
    };
    assert!(matches!(action, Action::Feature(Operation::Peers)));
    let mut reply = result("peers");
    reply.peers = Some(peers());
    feature_reply(&mut app, 2, reply);
    assert!(render(&app, 80, 24).contains("Phone"));
    assert!(is_none(&app.key(ch('2'))), "empty slot");
    assert!(app.modal.is_none());
    app.key(ch('1'));
    let Some(Modal::Review(_)) = &app.modal else {
        panic!("close needs a review")
    };
    assert!(render(&app, 80, 24).contains("11:22:33:44:55:66"));
    let action = request(app.key(key(KeyCode::Enter)));
    assert!(matches!(action, Action::Feature(ref op) if op.is_write()));

    // Re-entering with cached peers does not query again.
    let mut app = ready();
    app.peers = Some(peers());
    assert!(is_none(&app.navigate(Page::Peers)));
    // A failed query never leaves a stale list behind.
    let action = {
        let intent = app.key(ch('r'));
        accept(&mut app, 3, intent)
    };
    assert!(matches!(action, Action::Feature(Operation::Peers)));
    complete(&mut app, 3, Err("timeout".into()), true);
    assert!(app.peers.is_none());
}

#[test]
fn disconnected_peers_page_stays_offline() {
    let mut app = App::new(&config());
    assert!(is_none(&app.navigate(Page::Peers)));
    for c in ['r', '1', '2', 'R', 'e'] {
        assert!(is_none(&app.key(ch(c))));
    }
    assert_eq!(app.page, Page::Settings, "e opens the dual setting");
}

#[test]
fn audio_and_diagnostics_read_on_entry_and_ignore_other_targets() {
    let mut app = ready();
    let action = {
        let intent = app.navigate(Page::Audio);
        accept(&mut app, 2, intent)
    };
    assert!(matches!(action, Action::Audio { .. }));
    complete(
        &mut app,
        2,
        Ok(Output::Audio {
            address: BluetoothAddress::parse("11:22:33:44:55:66").unwrap(),
            observations: vec![],
        }),
        true,
    );
    assert!(
        app.audio.is_none(),
        "another target's observation is ignored"
    );
    assert!(
        app.connected(),
        "observations never change the control session"
    );
    assert!(matches!(
        request(app.key(ch('k'))),
        Action::Feature(Operation::Codec)
    ));
    let action = request(app.navigate(Page::Diagnostics));
    assert!(matches!(
        action,
        Action::Doctor {
            address: Some(_),
            ..
        }
    ));

    let mut app = App::new(&Config::default());
    assert!(is_none(&app.navigate(Page::Audio)));
    assert!(matches!(
        request(app.navigate(Page::Diagnostics)),
        Action::Doctor { address: None, .. }
    ));
}

#[test]
fn host_states_stay_independent_of_the_control_session() {
    let mut app = ready();
    assert_eq!(app.host_connection, HostConnectionState::Unknown);
    let mut listed = device("AA:BB:CC:DD:EE:FF", "Studio");
    listed.connected = HostConnectionState::Connected;
    app.accepted(
        2,
        Action::HostRefresh {
            address: app.address.clone(),
        },
    );
    complete(
        &mut app,
        2,
        Ok(Output::HostSnapshot {
            address: BluetoothAddress::parse("AA:BB:CC:DD:EE:FF").unwrap(),
            devices: vec![listed],
        }),
        true,
    );
    assert_eq!(app.host_connection, HostConnectionState::Connected);
    app.accepted(3, Action::Disconnect);
    complete(&mut app, 3, Ok(Output::Disconnected), false);
    assert!(!app.connected());
    assert_eq!(app.host_connection, HostConnectionState::Connected);
}

#[test]
fn failed_connect_reports_error_without_claiming_a_link() {
    let mut app = App::new(&config());
    let action = {
        let intent = app.key(ch('c'));
        accept(&mut app, 1, intent)
    };
    assert!(matches!(action, Action::Connect { .. }));
    assert_eq!(
        app.control_state,
        crate::bluetooth::ControlSessionState::Connecting
    );
    complete(&mut app, 1, Err("refused".into()), false);
    assert_eq!(
        app.control_state,
        crate::bluetooth::ControlSessionState::Error
    );
    assert_eq!(app.host_connection, HostConnectionState::Unknown);
    assert_eq!(app.status, "refused");
}

// ----- preferences -----------------------------------------------------------------

#[test]
fn options_change_language_channel_and_timeout_through_the_worker() {
    let mut app = ready();
    app.key(ch('s'));
    let action = request(app.key(key(KeyCode::Right)));
    assert!(matches!(action, Action::Configure { lang: Lang::Es, .. }));
    assert_eq!(app.lang, Lang::Es);
    app.accepted(2, action);
    assert!(
        is_none(&app.key(key(KeyCode::Right))),
        "one change at a time"
    );
    assert_eq!(app.lang, Lang::Es);
    complete(&mut app, 2, Ok(Output::Configured), true);
    app.key(key(KeyCode::Down));
    let action = request(app.key(key(KeyCode::Left)));
    assert!(matches!(action, Action::Configure { channel: 30, .. }));
    app.accepted(3, action);
    complete(&mut app, 3, Ok(Output::Configured), true);
    app.key(key(KeyCode::Down));
    let action = request(app.key(key(KeyCode::Right)));
    assert!(matches!(
        action,
        Action::Configure {
            timeout_seconds: 4,
            ..
        }
    ));
    app.accepted(4, action);
    complete(&mut app, 4, Ok(Output::Configured), true);
    app.key(key(KeyCode::Up));
    app.key(key(KeyCode::Up));
    app.key(key(KeyCode::Enter));
    assert!(matches!(app.modal, Some(Modal::Language { .. })));
    app.key(key(KeyCode::Home));
    assert!(matches!(
        request(app.key(key(KeyCode::Enter))),
        Action::Configure { lang: Lang::En, .. }
    ));
    assert!(app.modal.is_none());
}

#[test]
fn status_log_is_bounded_and_deduplicated() {
    let mut app = ready();
    for index in 0..150 {
        app.status = format!("event {index}");
        app.record_status();
        app.record_status();
    }
    assert_eq!(app.history.len(), 100);
    assert!(app.history.back().unwrap().ends_with("event 149"));
    app.key(ch('l'));
    let Some(Modal::Log { index }) = app.modal else {
        panic!()
    };
    assert_eq!(index, 99);
    app.key(key(KeyCode::Home));
    assert!(matches!(app.modal, Some(Modal::Log { index: 0 })));
    app.key(ch('l'));
    assert!(app.modal.is_none());
}

// ----- mouse ------------------------------------------------------------------------

#[test]
fn navigation_and_footer_hitboxes_match_the_rendered_frame() {
    for (width, height) in [(56u16, 20u16), (120, 40)] {
        let mut app = sized(ready(), width, height);
        let area = ratatui::layout::Rect::new(0, 0, width, height);
        let areas = view::areas(&app, area);
        let rects = view::nav_rects(areas.nav, &app);
        let screen = render(&app, width, height);
        let rows: Vec<&str> = screen.lines().collect();
        for (page, rect) in Page::ALL.iter().zip(rects) {
            let label = format!("F{}", page.index() + 1);
            let row: String = rows[usize::from(rect.y)]
                .chars()
                .skip(usize::from(rect.x))
                .take(usize::from(rect.width))
                .collect();
            assert!(row.contains(&label), "{label} drawn in its cell: {row:?}");
        }
        let target = rects[Page::Diagnostics.index()];
        assert!(matches!(
            request(click(&mut app, target.x + 1, target.y, width, height)),
            Action::Doctor { .. }
        ));
        assert_eq!(app.page, Page::Diagnostics);
        // The help hint in the footer opens help.
        let areas = view::areas(&app, area);
        let (_, help) = areas
            .slots
            .iter()
            .find(|(hint, _)| hint.key == "?")
            .expect("help hint visible");
        click(&mut app, help.x, help.y, width, height);
        assert!(matches!(app.modal, Some(Modal::Help { .. })));
    }
}

#[test]
fn settings_rows_select_then_propose_and_value_rows_click() {
    let (width, height) = (120, 40);
    let mut app = sized(ready(), width, height);
    app.navigate(Page::Settings);
    let area = ratatui::layout::Rect::new(0, 0, width, height);
    let (_, list, _) = view::settings_areas(view::areas(&app, area).body);
    click(&mut app, list.x + 3, list.y + 2, width, height);
    assert_eq!(app.selected, super::super::app::categories()[0][1]);
    assert!(app.proposals[app.selected].is_none());
    click(&mut app, list.x + 3, list.y + 2, width, height);
    assert!(app.proposals[app.selected].is_some());
    app.key(key(KeyCode::Esc));
    app.key(key(KeyCode::Enter));
    let count = super::super::app::choices(app.selected).len();
    let rect = view::list_modal_rect(area, count);
    click(&mut app, rect.x + 2, rect.y + count as u16, width, height);
    assert!(matches!(app.modal, Some(Modal::Review(_))));
}

#[test]
fn list_clicks_follow_the_rendered_scroll_offset() {
    let (width, height) = (60, 20);
    let mut app = sized(App::new(&Config::default()), width, height);
    app.devices = Some(
        (0..20)
            .map(|i| device(&format!("AA:BB:CC:DD:EE:{i:02X}"), "x"))
            .collect(),
    );
    app.page = Page::Devices;
    app.device_index = 15;
    let area = ratatui::layout::Rect::new(0, 0, width, height);
    let body = view::areas(&app, area).body;
    render(&app, width, height);
    let offset = app.devices_offset.get();
    assert!(offset > 0);
    click(&mut app, body.x + 3, body.y + 1, width, height);
    assert_eq!(app.device_index, offset);
    let screen = render(&app, width, height);
    assert_eq!(app.devices_offset.get(), offset, "the view does not jump");
    let first_row = screen.lines().nth(usize::from(body.y + 1)).unwrap();
    assert!(
        first_row.contains(&format!("EE:{offset:02X}")),
        "{first_row:?} offset {offset}\n{screen}"
    );
}

#[test]
fn wheel_moves_and_right_click_backs_out() {
    let mut app = ready();
    app.navigate(Page::Settings);
    app.mouse(
        MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        },
        80,
        24,
    );
    assert_eq!(app.selected, 1);
    app.key(ch('?'));
    app.mouse(
        MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Right),
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        },
        80,
        24,
    );
    assert!(app.modal.is_none());
}

#[test]
fn clicks_cannot_bypass_busy_or_modal_state() {
    let mut app = sized(ready(), 120, 40);
    app.accepted(5, Action::Set(Setting::parse("game", "on").unwrap()));
    let rects = view::nav_rects(
        view::areas(&app, ratatui::layout::Rect::new(0, 0, 120, 40)).nav,
        &app,
    );
    let devices = rects[Page::Devices.index()];
    assert!(is_none(&click(&mut app, devices.x + 1, devices.y, 120, 40)));
    assert_eq!(app.page, Page::Devices, "browsing is allowed");
    assert!(matches!(app.busy, Some((5, _))));
    app.key(ch('?'));
    let diagnostics = rects[Page::Diagnostics.index()];
    click(&mut app, diagnostics.x + 1, diagnostics.y, 120, 40);
    assert_eq!(app.page, Page::Devices, "modals capture clicks");
}

#[test]
fn tab_strips_scroll_to_keep_the_active_tab_visible() {
    use ratatui::layout::Rect;
    let tabs = view::tab_rects(Rect::new(0, 0, 56, 1), &["音声", "設定"], 0);
    assert_eq!(
        tabs,
        vec![(0, Rect::new(0, 0, 6, 1)), (1, Rect::new(7, 0, 6, 1))]
    );
    let labels = ["alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta"];
    let tabs = view::tab_rects(Rect::new(0, 0, 20, 1), &labels, 6);
    assert!(tabs.iter().any(|(index, _)| *index == 6));
    assert!(tabs.iter().all(|(_, rect)| rect.right() <= 20));
}

// ----- rendering ----------------------------------------------------------------------

#[test]
fn every_page_and_modal_renders_in_every_locale_and_size() {
    let modals = [
        Modal::Help { scroll: 0 },
        Modal::Log { index: 0 },
        Modal::Options { index: 0 },
        Modal::Language { index: 0 },
        Modal::Address {
            text: "AA:BB".into(),
        },
        Modal::Model,
        Modal::Values { index: 0 },
        Modal::Details,
    ];
    for lang in Lang::ALL {
        for (width, height) in [(56u16, 20u16), (80, 24), (130, 45)] {
            for page in Page::ALL {
                let mut app = sized(ready(), width, height);
                app.lang = lang;
                app.peers = Some(peers());
                app.devices = Some(vec![device("AA:BB:CC:DD:EE:FF", "Studio")]);
                app.page = page;
                let screen = render(&app, width, height);
                assert!(
                    screen.contains(&format!("F{}", page.index() + 1)),
                    "{lang:?} {page:?} {width}x{height}"
                );
                for modal in &modals {
                    app.modal = Some(modal.clone());
                    render(&app, width, height);
                }
                app.modal = None;
                app.quit_confirmation = true;
                let screen = render(&app, width, height);
                // Wide glyphs leave a blank continuation cell; compare without spaces.
                let squeeze = |s: &str| s.split_whitespace().collect::<String>();
                let title: String = squeeze(app.t().quit_title).chars().take(6).collect();
                assert!(
                    squeeze(&screen).contains(&title),
                    "{lang:?} {width}x{height}"
                );
            }
        }
    }
}

#[test]
fn tiny_terminals_show_resize_guidance_only() {
    for (width, height) in [(1, 1), (20, 10), (55, 30), (100, 19)] {
        let app = ready();
        let screen = render(&app, width, height);
        assert!(!screen.contains("F1"), "{width}x{height}");
        let mut app = ready();
        assert!(is_none(&app.mouse(
            MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 0,
                row: 0,
                modifiers: KeyModifiers::NONE,
            },
            width,
            height,
        )));
    }
}

#[test]
fn untrusted_text_cannot_inject_terminal_sequences() {
    let mut app = sized(ready(), 120, 40);
    app.status = "bad\x1b[2Jstatus\u{202e}".into();
    app.devices = Some(vec![device("AA:BB:CC:DD:EE:FF", "evil\x1b]0;x\x07name")]);
    for page in Page::ALL {
        app.page = page;
        let screen = render(&app, 120, 40);
        assert!(!screen.contains('\x1b') && !screen.contains('\u{202e}'));
    }
}

#[test]
fn popups_keep_the_dark_background_after_clearing() {
    let mut app = sized(ready(), 80, 24);
    app.modal = Some(Modal::Model);
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| view::render(frame, &app)).unwrap();
    let buffer = terminal.backend().buffer();
    assert_eq!(buffer[(40, 12)].bg, ratatui::style::Color::Rgb(15, 20, 30));
}

#[test]
fn help_is_generated_from_the_keymap_and_scrolls() {
    let mut app = sized(ready(), 56, 20);
    app.key(ch('?'));
    let screen = render(&app, 56, 20);
    assert!(screen.contains(app.tt().help_global));
    let max = app.max_scroll.get();
    assert!(max > 0);
    for _ in 0..(max + 5) {
        app.key(key(KeyCode::Down));
    }
    assert!(matches!(app.modal, Some(Modal::Help { scroll }) if scroll == max));
    let screen = render(&app, 56, 20);
    assert!(screen.contains("Mouse") || screen.contains("Safety"));
}

#[test]
fn footer_reflects_the_current_context() {
    let mut app = sized(App::new(&config()), 120, 40);
    assert!(render(&app, 120, 40).contains("c connect"));
    app.key(ch('?'));
    let screen = render(&app, 120, 40);
    assert!(!screen.lines().last().unwrap().contains("c connect"));
    app.key(key(KeyCode::Esc));
    app.accepted(1, Action::Refresh);
    assert!(render(&app, 120, 40).contains("Esc cancel"));
}

#[test]
fn spanish_interface_translates_pages_and_hints() {
    let mut app = sized(ready(), 120, 40);
    app.lang = Lang::Es;
    let screen = render(&app, 120, 40);
    assert!(screen.contains("Ajustes") && screen.contains("Conexiones"));
    assert!(screen.contains("desconectar"));
}

#[test]
fn derived_locales_translate_page_names() {
    for lang in Lang::ALL {
        let tt = crate::i18n::tui::text(lang);
        assert!(
            tt.pages.iter().all(|page| !page.trim().is_empty()),
            "{lang:?}"
        );
        let unique: std::collections::HashSet<_> = tt.pages.iter().collect();
        assert_eq!(unique.len(), 8, "{lang:?} page names are distinct");
    }
}

#[test]
fn fill_replaces_placeholders_in_order_without_rescanning_values() {
    assert_eq!(
        super::super::app::fill("{} and {}", &[&"{}", &2]),
        "{} and 2"
    );
    assert_eq!(super::super::app::fill("none", &[&1]), "none");
}
