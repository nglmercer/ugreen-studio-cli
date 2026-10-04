//! Entirely offline: state, rendering and worker tests use fake sessions only.
use super::{
    state::{choices, App, Intent},
    view,
    worker::{Action, Backend, Engine, Output, Reply, Request, Session, Snapshot, Worker},
    Config,
};
use crate::{
    settings::{self, DeviceInfo, Setting},
    transport::Device,
};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{backend::TestBackend, Terminal};
use std::{
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    time::{Duration, Instant},
};

fn info() -> DeviceInfo {
    let mut raw = vec![0; 30];
    raw[0] = 76;
    raw[3] = 0xa0;
    DeviceInfo::new(raw).unwrap()
}
fn config() -> Config {
    Config {
        address: Some("AA:BB:CC:DD:EE:FF".into()),
        model_confirmed: true,
        ..Config::default()
    }
}
fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}
fn ready() -> App {
    let mut app = App::new(&config());
    app.accepted(
        1,
        Action::Connect {
            address: app.address.clone(),
            model_confirmed: true,
        },
    );
    app.receive(Reply {
        id: 1,
        result: Ok(Output::Snapshot(Snapshot {
            info: info(),
            firmware: Some("1.2.3".into()),
            note: None,
        })),
        connected: true,
        writes_blocked: false,
    });
    app
}
fn render(app: &App, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| view::render(frame, app)).unwrap();
    terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>()
}
#[test]
fn startup_is_disconnected_without_implicit_actions() {
    let app = App::new(&config());
    assert!(!app.connected);
    assert!(app.info.is_none());
    assert!(app.busy.is_none());
    assert!(app.paired.is_none());
    assert!(app.status.contains("No Bluetooth access"));
}
#[test]
fn connect_requires_address_and_model() {
    let mut app = App::new(&Config::default());
    assert!(matches!(app.key(key(KeyCode::Char('c'))), Intent::None));
    assert!(app.status.contains("six colon-separated"));
    app.address = "AA:BB:CC:DD:EE:FF".into();
    assert!(matches!(app.key(key(KeyCode::Char('c'))), Intent::None));
    assert!(app.model_confirmation);
    // Confirming the protocol with a valid target connects at once.
    assert!(matches!(
        app.key(key(KeyCode::Char('y'))),
        Intent::Request(Action::Connect {
            model_confirmed: true,
            ..
        })
    ));
    assert!(app.model_confirmed);
}
#[test]
fn model_cancel_does_not_select_or_connect() {
    let mut app = App::new(&Config::default());
    app.key(key(KeyCode::Char('m')));
    app.key(key(KeyCode::Esc));
    assert!(!app.model_confirmed);
    assert!(!app.connected);
}
#[test]
fn address_editor_is_local_bounded_and_validated() {
    let mut app = App::new(&Config::default());
    app.key(key(KeyCode::Char('a')));
    for ch in "ab:cd:ef:01:23:45xyz".chars() {
        app.key(key(KeyCode::Char(ch)));
    }
    assert_eq!(app.address_edit.as_deref(), Some("AB:CD:EF:01:23:45"));
    assert!(matches!(app.key(key(KeyCode::Enter)), Intent::None));
    assert_eq!(app.address, "AB:CD:EF:01:23:45");
    assert!(!app.connected);
    app.key(key(KeyCode::Char('a')));
    app.key(key(KeyCode::Backspace));
    app.key(key(KeyCode::Enter));
    assert!(app.address_edit.is_some());
    app.key(key(KeyCode::Esc));
    assert_eq!(app.address, "AB:CD:EF:01:23:45");
}
#[test]
fn paired_cache_only_on_explicit_request_and_selection_does_not_connect() {
    let mut app = App::new(&Config::default());
    let action = app.key(key(KeyCode::Char('p')));
    assert!(matches!(action, Intent::Request(Action::Paired)));
    app.accepted(1, Action::Paired);
    app.receive(Reply {
        id: 1,
        result: Ok(Output::Paired(vec![Device {
            address: "AA:BB:CC:DD:EE:FF".into(),
            name: "Studio Pro".into(),
        }])),
        connected: false,
        writes_blocked: false,
    });
    assert!(matches!(app.key(key(KeyCode::Enter)), Intent::None));
    assert_eq!(app.address, "AA:BB:CC:DD:EE:FF");
    assert!(!app.connected);
}
#[test]
fn first_device_pick_opens_protocol_modal_then_connects() {
    let mut app = App::new(&Config {
        address: None,
        model_confirmed: false,
        ..Config::default()
    });
    app.paired = Some(vec![Device {
        address: "AA:BB:CC:DD:EE:FF".into(),
        name: "Studio Pro".into(),
    }]);
    assert!(matches!(app.key(key(KeyCode::Enter)), Intent::None));
    assert_eq!(app.address, "AA:BB:CC:DD:EE:FF");
    assert!(app.paired.is_none());
    assert!(app.model_confirmation);
    assert!(!app.model_confirmed);
    // y or Enter in the modal connects at once.
    assert!(matches!(
        app.key(key(KeyCode::Char('y'))),
        Intent::Request(Action::Connect {
            model_confirmed: true,
            ..
        })
    ));
    assert!(app.model_confirmed);
}
#[test]
fn device_pick_with_confirmed_protocol_connects_at_once() {
    let mut app = App::new(&config());
    app.paired = Some(vec![Device {
        address: "AA:BB:CC:DD:EE:FF".into(),
        name: "Studio Pro".into(),
    }]);
    assert!(matches!(
        app.key(key(KeyCode::Enter)),
        Intent::Request(Action::Connect {
            model_confirmed: true,
            ..
        })
    ));
    assert!(app.paired.is_none());
}
#[test]
fn model_modal_enter_confirms_and_connects() {
    let mut app = App::new(&config());
    app.model_confirmation = true;
    assert!(matches!(
        app.key(key(KeyCode::Enter)),
        Intent::Request(Action::Connect {
            model_confirmed: true,
            ..
        })
    ));
    assert!(app.model_confirmed);
    assert!(!app.model_confirmation);
}
#[test]
fn empty_paired_list_navigation_is_safe() {
    let mut app = App::new(&config());
    app.paired = Some(vec![]);
    app.key(key(KeyCode::Down));
    app.key(key(KeyCode::Up));
    app.key(key(KeyCode::Enter));
    assert_eq!(app.address, "AA:BB:CC:DD:EE:FF");
    assert!(app.paired.is_none());
}
#[test]
fn unknown_cached_address_cannot_replace_target() {
    let mut app = App::new(&config());
    app.paired = Some(vec![Device {
        address: "bad".into(),
        name: "bad".into(),
    }]);
    app.key(key(KeyCode::Enter));
    assert_eq!(app.address, "AA:BB:CC:DD:EE:FF");
}
#[test]
fn connected_target_cannot_be_edited_or_replaced() {
    let mut app = ready();
    for code in ['a', 'p', 'm', 'c'] {
        assert!(matches!(app.key(key(KeyCode::Char(code))), Intent::None));
    }
    assert!(app.address_edit.is_none());
    assert!(app.paired.is_none());
}
#[test]
fn proposing_then_enter_applies_directly() {
    let mut app = ready();
    assert!(matches!(app.key(key(KeyCode::Right)), Intent::None));
    assert_eq!(app.proposals[0].as_deref(), Some("ultra"));
    assert_eq!(
        app.info.as_ref().unwrap().value("anc").as_deref(),
        Some("off")
    );
    assert!(
        matches!(app.key(key(KeyCode::Enter)), Intent::Request(Action::Set(setting)) if setting.key == "anc" && setting.value == "ultra")
    );
    assert!(app.proposals[0].is_none());
}
#[test]
fn escape_discards_proposal_without_writing() {
    let mut app = ready();
    app.key(key(KeyCode::Right));
    assert!(matches!(app.key(key(KeyCode::Esc)), Intent::None));
    assert!(app.proposals[0].is_none());
}
#[test]
fn same_value_and_unavailable_value_do_not_write() {
    let mut app = ready();
    app.key(key(KeyCode::Right));
    app.key(key(KeyCode::Left));
    app.key(key(KeyCode::Enter));
    app.info = Some(DeviceInfo::new(vec![255; 8]).unwrap());
    app.selected = 8;
    app.key(key(KeyCode::Right));
    assert!(app.proposals[8].is_none());
}
#[test]
fn all_proposed_values_are_real_supported_settings() {
    for (index, key) in settings::KEYS.iter().enumerate() {
        for value in choices(index) {
            assert!(Setting::parse(key, value).is_ok(), "{key} {value}");
        }
    }
}
#[test]
fn repeats_and_releases_cannot_apply_or_quit() {
    let mut app = ready();
    for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
        let mut event = key(KeyCode::Char('q'));
        event.kind = kind;
        assert!(matches!(app.key(event), Intent::None));
    }
}
#[test]
fn busy_rejects_new_requests_and_editing() {
    let mut app = ready();
    app.accepted(2, Action::Refresh);
    for code in ['p', 'r', 'c', 'a', 'm'] {
        assert!(matches!(app.key(key(KeyCode::Char(code))), Intent::None));
    }
    assert!(matches!(app.key(key(KeyCode::Right)), Intent::None));
    assert!(app.proposals[0].is_none());
}
#[test]
fn cancel_keeps_busy_until_matching_completion() {
    let mut app = ready();
    app.accepted(2, Action::Set(Setting::parse("game", "on").unwrap()));
    assert!(matches!(app.key(key(KeyCode::Esc)), Intent::Cancel));
    assert!(app.busy.is_some());
    assert!(app.status.contains("may complete"));
    app.receive(Reply {
        id: 999,
        result: Ok(Output::Disconnected),
        connected: false,
        writes_blocked: false,
    });
    assert!(app.busy.is_some());
    assert!(app.connected);
}
#[test]
fn disconnect_after_cancel_is_scheduled_only_after_completion() {
    let mut app = ready();
    app.accepted(2, Action::Refresh);
    assert!(matches!(app.key(key(KeyCode::Char('d'))), Intent::Cancel));
    assert!(app.busy.is_some());
    let next = app.receive(Reply {
        id: 2,
        result: Err("Cancelled".into()),
        connected: true,
        writes_blocked: false,
    });
    assert!(matches!(next, Some(Action::Disconnect)));
}
#[test]
fn uncertain_write_blocks_proposals_until_explicit_refresh() {
    let mut app = ready();
    app.accepted(2, Action::Set(Setting::parse("game", "on").unwrap()));
    app.receive(Reply {
        id: 2,
        result: Err("No acknowledgement".into()),
        connected: true,
        writes_blocked: true,
    });
    assert!(app.stale);
    assert!(app.writes_blocked);
    app.key(key(KeyCode::Right));
    assert!(app.proposals[0].is_none());
    assert!(matches!(
        app.key(key(KeyCode::Char('r'))),
        Intent::Request(Action::Refresh)
    ));
}
#[test]
fn quit_during_work_requests_cancel_and_requires_second_action() {
    let mut app = ready();
    app.accepted(2, Action::Refresh);
    assert!(matches!(app.key(key(KeyCode::Char('q'))), Intent::Cancel));
    assert!(app.quit_confirmation);
    assert!(matches!(app.key(key(KeyCode::Enter)), Intent::Quit));
}
#[test]
fn control_c_quits_from_address_editor() {
    let mut app = App::new(&config());
    app.key(key(KeyCode::Char('a')));
    assert!(matches!(
        app.key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
        Intent::Quit
    ));
}
#[test]
fn stale_replies_never_overwrite_ui() {
    let mut app = ready();
    let old = app.info.clone();
    app.receive(Reply {
        id: 1,
        result: Ok(Output::Disconnected),
        connected: false,
        writes_blocked: true,
    });
    assert_eq!(app.info, old);
    assert!(app.connected);
    assert!(!app.writes_blocked);
}
#[test]
fn verified_write_replaces_readback_and_clears_proposals() {
    let mut app = ready();
    app.proposals[0] = Some("ultra".into());
    let setting = Setting::parse("anc", "ultra").unwrap();
    let mut data = info();
    data.raw[3] = 0xa1;
    app.accepted(2, Action::Set(setting.clone()));
    app.receive(Reply {
        id: 2,
        result: Ok(Output::Verified {
            info: data,
            setting,
        }),
        connected: true,
        writes_blocked: false,
    });
    assert!(!app.stale);
    assert!(app.proposals.iter().all(Option::is_none));
    assert!(app.status.contains("Verified"));
}
#[test]
fn worker_failure_disables_future_work() {
    let mut app = ready();
    app.accepted(2, Action::Set(Setting::parse("game", "on").unwrap()));
    app.worker_failed("Worker stopped".into());
    assert!(!app.connected);
    assert!(app.writes_blocked);
    assert!(app.status.contains("may have completed"));
    assert!(matches!(app.key(key(KeyCode::Char('c'))), Intent::None));
}
#[test]
fn default_render_truthfully_shows_missing_data() {
    let text = render(&App::new(&Config::default()), 100, 30);
    assert!(text.contains("DISCONNECTED"));
    assert!(text.contains("Battery: unavailable"));
    assert!(text.contains("Codec: unavailable"));
    assert!(text.contains("fw 0.2.5 checked"));
}
#[test]
fn connected_render_only_shows_deviceinfo_battery() {
    let mut app = ready();
    assert!(render(&app, 100, 30).contains("Battery: 76%"));
    app.info.as_mut().unwrap().raw[0] = 255;
    assert!(render(&app, 100, 30).contains("Battery: unavailable"));
}
#[test]
fn rendering_handles_all_small_dimensions_and_resize() {
    let mut app = ready();
    app.selected = 8;
    for (w, h) in [
        (0, 0),
        (1, 1),
        (20, 3),
        (55, 19),
        (56, 20),
        (80, 24),
        (160, 50),
    ] {
        let _ = render(&app, w, h);
    }
    assert!(render(&app, 56, 20).contains("Volume-down action"));
    assert!(render(&app, 30, 6).contains("Resize"));
}
#[test]
fn every_modal_renders_and_quit_warning_has_priority() {
    let mut app = ready();
    app.help = true;
    assert!(render(&app, 100, 32).contains("Shortcuts"));
    app.help = false;
    app.address_edit = Some("AA:BB".into());
    assert!(render(&app, 80, 24).contains("Edit target"));
    app.address_edit = None;
    app.model_confirmation = true;
    assert!(render(&app, 80, 24).contains("Max5c"));
    app.model_confirmation = false;
    app.quit_confirmation = true;
    assert!(render(&app, 80, 24).contains("Quit while work"));
}
#[test]
fn language_toggle_rewrites_status_and_render() {
    let mut app = App::new(&Config::default());
    assert!(app.status.contains("No Bluetooth access"));
    app.key(key(KeyCode::Char('L')));
    assert!(app.status.contains("español"));
    assert!(render(&app, 100, 30).contains("DESCONECTADO"));
    assert!(render(&app, 100, 30).contains("Destino:"));
    app.key(key(KeyCode::Char('L')));
    assert!(app.status.contains("English"));
    assert!(render(&app, 100, 30).contains("DISCONNECTED"));
}
#[test]
fn spanish_render_shows_translated_panels() {
    let mut cfg = config();
    cfg.lang = crate::i18n::Lang::Es;
    let app = App::new(&cfg);
    let text = render(&app, 100, 30);
    assert!(text.contains("Batería:"));
    assert!(text.contains("Ajustes"));
    assert!(text.contains("Estado"));
}
#[test]
fn footer_and_shortcuts_screen_list_new_bindings() {
    let app = App::new(&Config::default());
    let text = render(&app, 110, 30);
    assert!(text.contains("shortcuts"));
    assert!(text.contains("language"));
    let mut app = ready();
    app.help = true;
    let help = render(&app, 110, 40);
    assert!(help.contains("Mouse"));
    assert!(help.contains("Click"));
    assert!(help.contains("L "));
}
#[test]
fn mouse_click_selects_setting_and_second_click_proposes() {
    use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
    let mut app = ready();
    let click = |row| MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 5,
        row,
        modifiers: KeyModifiers::NONE,
    };
    assert!(matches!(app.mouse(click(9), 110, 30), Intent::None));
    assert_eq!(app.selected, 2);
    let intent = app.mouse(click(9), 110, 30);
    assert!(matches!(intent, Intent::None));
    assert_eq!(app.proposals[2].as_deref(), Some("on"));
}
#[test]
fn mouse_wheel_scrolls_and_right_click_cancels() {
    use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
    let mut app = ready();
    let wheel = |down: bool| MouseEvent {
        kind: if down {
            MouseEventKind::ScrollDown
        } else {
            MouseEventKind::ScrollUp
        },
        column: 5,
        row: 10,
        modifiers: KeyModifiers::NONE,
    };
    app.mouse(wheel(true), 110, 30);
    assert_eq!(app.selected, 1);
    app.mouse(wheel(false), 110, 30);
    assert_eq!(app.selected, 0);
    app.key(key(KeyCode::Char('l')));
    assert!(app.log_open);
    let right = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Right),
        column: 5,
        row: 10,
        modifiers: KeyModifiers::NONE,
    };
    app.mouse(right, 110, 30);
    assert!(!app.log_open);
}
#[test]
fn mouse_footer_shortcuts_dispatch() {
    use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
    let mut app = ready();
    let click = |col: u16| MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: col,
        row: 29,
        modifiers: KeyModifiers::NONE,
    };
    app.mouse(click(2), 120, 30);
    assert!(app.log_open);
}
#[test]
fn autoload_flag_defaults_skip_for_tests() {
    let app = App::new(&Config::default());
    assert!(app.config_skip_autoload());
}
#[test]
fn quit_through_paired_load_needs_no_confirmation() {
    let mut app = App::new(&Config::default());
    app.accepted(1, Action::Paired);
    assert!(matches!(app.key(key(KeyCode::Char('q'))), Intent::Quit));
    let mut app = ready();
    app.accepted(2, Action::Refresh);
    assert!(matches!(app.key(key(KeyCode::Char('q'))), Intent::Cancel));
    assert!(app.quit_confirmation);
}
#[test]
fn untrusted_names_and_errors_cannot_inject_terminal_control_sequences() {
    let cleaned = view::clean("a\x1b[2J\n\u{202e}b");
    assert!(!cleaned.contains('\x1b'));
    assert!(!cleaned.contains('\n'));
    assert!(!cleaned.contains('\u{202e}'));
    let mut app = ready();
    app.status = "error\x1b[2Jevil".into();
    assert!(!render(&app, 80, 24).contains('\x1b'));
    assert_eq!(view::clean(&"x".repeat(2000)).len(), 1024);
}

#[derive(Default)]
struct MockState {
    calls: Vec<String>,
    raw: Vec<u8>,
    info_count: usize,
    fail_info: Option<usize>,
    fail_ack: bool,
    fail_firmware: bool,
    wrong_readback: bool,
    cancel: Option<Arc<AtomicBool>>,
    cancel_info: Option<usize>,
    cancel_ack: bool,
    cancel_connect: bool,
}
struct FakeBackend(Arc<Mutex<MockState>>);
struct FakeSession(Arc<Mutex<MockState>>);
impl Backend for FakeBackend {
    fn paired(&mut self) -> io::Result<Vec<Device>> {
        self.0.lock().unwrap().calls.push("paired".into());
        Ok(vec![])
    }
    fn connect(&mut self, _: &Config, _: &str) -> io::Result<Box<dyn Session>> {
        let mut s = self.0.lock().unwrap();
        s.calls.push("connect".into());
        if s.cancel_connect {
            s.cancel.as_ref().unwrap().store(true, Ordering::SeqCst);
        }
        Ok(Box::new(FakeSession(self.0.clone())))
    }
}
impl Session for FakeSession {
    fn info(&mut self) -> io::Result<DeviceInfo> {
        let mut s = self.0.lock().unwrap();
        s.calls.push("info".into());
        s.info_count += 1;
        if s.cancel_info == Some(s.info_count) {
            s.cancel.as_ref().unwrap().store(true, Ordering::SeqCst);
        }
        if s.fail_info == Some(s.info_count) {
            return Err(io::Error::other("mock status failure"));
        }
        DeviceInfo::new(s.raw.clone()).map_err(io::Error::other)
    }
    fn firmware(&mut self) -> io::Result<String> {
        let mut s = self.0.lock().unwrap();
        s.calls.push("firmware".into());
        if s.fail_firmware {
            Err(io::Error::other("mock firmware failure"))
        } else {
            Ok("1.2.3".into())
        }
    }
    fn acknowledge(&mut self, setting: &Setting) -> io::Result<()> {
        let mut s = self.0.lock().unwrap();
        s.calls
            .push(format!("ack:{}={}", setting.key, setting.value));
        if s.cancel_ack {
            s.cancel.as_ref().unwrap().store(true, Ordering::SeqCst);
        }
        if s.fail_ack {
            return Err(io::Error::other("mock ACK failure"));
        }
        if !s.wrong_readback {
            assert_eq!(setting.key, "game");
            s.raw[6] = setting.payload[0];
        }
        Ok(())
    }
}
fn engine() -> (Engine<FakeBackend>, Arc<Mutex<MockState>>) {
    let state = Arc::new(Mutex::new(MockState {
        raw: info().raw,
        ..Default::default()
    }));
    (
        Engine::new(
            FakeBackend(state.clone()),
            config(),
            Arc::new(AtomicBool::new(false)),
        ),
        state,
    )
}
fn request(action: Action) -> Request {
    Request {
        id: 1,
        action,
        cancelled: Arc::new(AtomicBool::new(false)),
    }
}
fn connected_engine() -> (Engine<FakeBackend>, Arc<Mutex<MockState>>) {
    let (mut engine, state) = engine();
    assert!(engine
        .execute(request(Action::Connect {
            address: "AA:BB:CC:DD:EE:FF".into(),
            model_confirmed: true
        }))
        .result
        .is_ok());
    {
        let mut s = state.lock().unwrap();
        s.calls.clear();
        s.info_count = 0;
    }
    (engine, state)
}
fn set() -> Action {
    Action::Set(Setting::parse("game", "on").unwrap())
}
#[test]
fn constructing_engine_does_no_io() {
    let (_, state) = engine();
    assert!(state.lock().unwrap().calls.is_empty());
}
#[test]
fn engine_rejects_missing_model_and_bad_address_before_io() {
    let (mut engine, state) = engine();
    for (address, model_confirmed) in [("AA:BB:CC:DD:EE:FF", false), ("bad", true)] {
        assert!(engine
            .execute(request(Action::Connect {
                address: address.into(),
                model_confirmed
            }))
            .result
            .is_err());
    }
    assert!(state.lock().unwrap().calls.is_empty());
}
#[test]
fn connect_reads_info_and_firmware_but_never_writes_settings() {
    let (mut engine, state) = engine();
    let reply = engine.execute(request(Action::Connect {
        address: "AA:BB:CC:DD:EE:FF".into(),
        model_confirmed: true,
    }));
    assert!(reply.connected);
    assert!(reply.result.is_ok());
    assert_eq!(state.lock().unwrap().calls, ["connect", "info", "firmware"]);
}
#[test]
fn connect_failure_preflight_drops_session() {
    let (mut engine, state) = engine();
    state.lock().unwrap().fail_info = Some(1);
    let reply = engine.execute(request(Action::Connect {
        address: "AA:BB:CC:DD:EE:FF".into(),
        model_confirmed: true,
    }));
    assert!(!reply.connected);
    assert!(reply.result.is_err());
    assert_eq!(state.lock().unwrap().calls, ["connect", "info"]);
}
#[test]
fn firmware_failure_preserves_real_deviceinfo() {
    let (mut engine, state) = connected_engine();
    state.lock().unwrap().fail_firmware = true;
    let reply = engine.execute(request(Action::Refresh));
    assert!(matches!(
        reply.result,
        Ok(Output::Snapshot(Snapshot {
            firmware: None,
            note: Some(_),
            ..
        }))
    ));
    assert!(reply.connected);
}
#[test]
fn successful_write_has_preflight_ack_and_matching_readback_in_order() {
    let (mut engine, state) = connected_engine();
    let reply = engine.execute(request(set()));
    assert!(matches!(reply.result, Ok(Output::Verified { .. })));
    assert!(!reply.writes_blocked);
    assert_eq!(state.lock().unwrap().calls, ["info", "ack:game=on", "info"]);
}
#[test]
fn preflight_failure_and_unknown_setting_never_send_ack() {
    let (mut engine, state) = connected_engine();
    state.lock().unwrap().fail_info = Some(1);
    let reply = engine.execute(request(set()));
    assert!(reply.result.is_err());
    assert!(!reply.writes_blocked);
    assert_eq!(state.lock().unwrap().calls, ["info"]);
    state.lock().unwrap().raw = vec![255; 8];
    let reply = engine.execute(request(set()));
    assert!(reply.result.is_err());
    assert_eq!(state.lock().unwrap().calls, ["info", "info"]);
}
#[test]
fn acknowledgement_failure_blocks_until_explicit_successful_refresh() {
    let (mut engine, state) = connected_engine();
    state.lock().unwrap().fail_ack = true;
    let reply = engine.execute(request(set()));
    assert!(reply.writes_blocked);
    assert!(reply.result.is_err());
    let calls = state.lock().unwrap().calls.clone();
    assert!(engine.execute(request(set())).result.is_err());
    assert_eq!(state.lock().unwrap().calls, calls);
    state.lock().unwrap().fail_info = Some(2);
    assert!(engine.execute(request(Action::Refresh)).writes_blocked);
    state.lock().unwrap().fail_info = None;
    assert!(!engine.execute(request(Action::Refresh)).writes_blocked);
}
#[test]
fn readback_failure_and_mismatch_are_not_success() {
    for mismatch in [false, true] {
        let (mut engine, state) = connected_engine();
        if mismatch {
            state.lock().unwrap().wrong_readback = true;
        } else {
            state.lock().unwrap().fail_info = Some(2);
        }
        let reply = engine.execute(request(set()));
        assert!(reply.result.is_err());
        assert!(reply.writes_blocked);
        assert_eq!(state.lock().unwrap().calls, ["info", "ack:game=on", "info"]);
    }
}
#[test]
fn disconnect_and_reconnect_do_not_silently_clear_uncertain_write() {
    let (mut engine, state) = connected_engine();
    state.lock().unwrap().fail_ack = true;
    assert!(engine.execute(request(set())).writes_blocked);
    assert!(engine.execute(request(Action::Disconnect)).writes_blocked);
    assert!(
        engine
            .execute(request(Action::Connect {
                address: "AA:BB:CC:DD:EE:FF".into(),
                model_confirmed: true
            }))
            .writes_blocked
    );
    assert!(!engine.execute(request(Action::Refresh)).writes_blocked);
}
#[test]
fn cancellation_before_request_performs_no_io() {
    let (mut engine, state) = connected_engine();
    let r = request(set());
    r.cancelled.store(true, Ordering::SeqCst);
    assert!(engine.execute(r).result.is_err());
    assert!(state.lock().unwrap().calls.is_empty());
}
#[test]
fn cancellation_during_connect_skips_status_and_drops_new_session() {
    let (mut engine, state) = engine();
    let r = request(Action::Connect {
        address: "AA:BB:CC:DD:EE:FF".into(),
        model_confirmed: true,
    });
    {
        let mut s = state.lock().unwrap();
        s.cancel = Some(r.cancelled.clone());
        s.cancel_connect = true;
    }
    let reply = engine.execute(r);
    assert!(reply.result.is_err());
    assert!(!reply.connected);
    assert_eq!(state.lock().unwrap().calls, ["connect"]);
}
#[test]
fn cancellation_during_preflight_skips_setting_write() {
    let (mut engine, state) = connected_engine();
    let r = request(set());
    {
        let mut s = state.lock().unwrap();
        s.cancel = Some(r.cancelled.clone());
        s.cancel_info = Some(1);
    }
    let reply = engine.execute(r);
    assert!(reply.result.is_err());
    assert!(!reply.writes_blocked);
    assert_eq!(state.lock().unwrap().calls, ["info"]);
}
#[test]
fn cancellation_during_ack_skips_readback_and_blocks_more_writes() {
    let (mut engine, state) = connected_engine();
    let r = request(set());
    {
        let mut s = state.lock().unwrap();
        s.cancel = Some(r.cancelled.clone());
        s.cancel_ack = true;
    }
    let reply = engine.execute(r);
    assert!(reply.writes_blocked);
    assert!(reply.result.unwrap_err().contains("may have completed"));
    assert_eq!(state.lock().unwrap().calls, ["info", "ack:game=on"]);
}
#[test]
fn cancellation_during_readback_does_not_report_verified_or_unlock() {
    let (mut engine, state) = connected_engine();
    let r = request(set());
    {
        let mut s = state.lock().unwrap();
        s.cancel = Some(r.cancelled.clone());
        s.cancel_info = Some(2);
    }
    let reply = engine.execute(r);
    assert!(reply.writes_blocked);
    assert!(reply.result.is_err());
}
#[test]
fn shutdown_barrier_skips_all_later_stages() {
    let state = Arc::new(Mutex::new(MockState {
        raw: info().raw,
        ..Default::default()
    }));
    let shutdown = Arc::new(AtomicBool::new(true));
    let mut engine = Engine::new(FakeBackend(state.clone()), config(), shutdown);
    assert!(engine.execute(request(Action::Paired)).result.is_err());
    assert!(state.lock().unwrap().calls.is_empty());
}
struct GateBackend {
    started: mpsc::SyncSender<()>,
    release: mpsc::Receiver<()>,
}
impl Backend for GateBackend {
    fn paired(&mut self) -> io::Result<Vec<Device>> {
        self.started.send(()).unwrap();
        self.release.recv_timeout(Duration::from_secs(5)).unwrap();
        Ok(vec![])
    }
    fn connect(&mut self, _: &Config, _: &str) -> io::Result<Box<dyn Session>> {
        panic!("test must not connect")
    }
}
#[test]
fn worker_admits_only_one_request_and_drop_never_waits_for_io() {
    let (started_tx, started_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let mut worker = Worker::spawn(
        GateBackend {
            started: started_tx,
            release: release_rx,
        },
        config(),
    )
    .unwrap();
    worker.submit(Action::Paired).unwrap();
    started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    assert!(worker.submit(Action::Paired).is_err());
    assert!(worker.poll().unwrap().is_none());
    let now = Instant::now();
    worker.cancel();
    drop(worker);
    assert!(now.elapsed() < Duration::from_secs(1));
    release_tx.send(()).unwrap();
}
#[test]
fn worker_cancellation_reply_is_bounded_and_allows_next_request() {
    let (started_tx, started_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let mut worker = Worker::spawn(
        GateBackend {
            started: started_tx,
            release: release_rx,
        },
        config(),
    )
    .unwrap();
    worker.submit(Action::Paired).unwrap();
    started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    worker.cancel();
    release_tx.send(()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Some(reply) = worker.poll().unwrap() {
            assert!(reply.result.is_err());
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    worker.submit(Action::Paired).unwrap();
    started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    release_tx.send(()).unwrap();
}

#[test]
fn status_log_is_bounded_deduplicated_and_keyboard_accessible() {
    let mut app = ready();
    app.record_status();
    app.record_status();
    assert_eq!(app.history.len(), 1);
    for index in 0..150 {
        app.status = format!("event {index}");
        app.record_status();
    }
    assert_eq!(app.history.len(), 100);
    assert!(app.history.front().unwrap().ends_with("event 50"));
    app.key(key(KeyCode::Char('l')));
    assert!(app.log_open);
    app.key(key(KeyCode::Home));
    assert_eq!(app.log_index, 0);
    app.key(key(KeyCode::End));
    assert_eq!(app.log_index, 99);
    assert!(render(&app, 80, 24).contains("event 149"));
    app.key(key(KeyCode::Esc));
    assert!(!app.log_open);
}
#[test]
fn supplied_address_is_terminal_safe_and_dark_background_is_explicit() {
    let mut app = App::new(&config());
    app.address = "bad\x1b[2J\u{202e}target".into();
    let text = render(&app, 100, 30);
    assert!(!text.contains('\x1b'));
    assert!(!text.contains('\u{202e}'));
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    terminal.draw(|frame| view::render(frame, &app)).unwrap();
    assert_eq!(
        terminal.backend().buffer()[(0, 29)].bg,
        ratatui::style::Color::Rgb(15, 20, 30)
    );
}
#[test]
fn help_scrolls_on_minimum_screen() {
    let mut app = ready();
    app.key(key(KeyCode::Char('?')));
    for _ in 0..18 {
        app.key(key(KeyCode::Down));
    }
    assert!(app.help_scroll > 0);
    let text = render(&app, 56, 20);
    assert!(text.contains("in-flight") || text.contains("Uncertain"));
    app.key(key(KeyCode::Home));
    assert_eq!(app.help_scroll, 0);
}
#[test]
fn address_control_u_clears_prefilled_value_without_io() {
    let mut app = App::new(&config());
    app.key(key(KeyCode::Char('a')));
    app.key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL));
    assert_eq!(app.address_edit.as_deref(), Some(""));
    assert!(!app.connected);
}

#[test]
fn every_popup_restores_explicit_dark_background_after_clear() {
    for modal in 0..5 {
        let mut app = ready();
        match modal {
            0 => app.help = true,
            1 => {
                app.log_open = true;
                app.record_status();
            }
            2 => {
                app.paired = Some(vec![Device {
                    address: "AA:BB:CC:DD:EE:FF".into(),
                    name: "Headphones".into(),
                }])
            }
            3 => app.quit_confirmation = true,
            _ => app.model_confirmation = true,
        }
        let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
        terminal.draw(|frame| view::render(frame, &app)).unwrap();
        assert_eq!(
            terminal.backend().buffer()[(50, 15)].bg,
            ratatui::style::Color::Rgb(15, 20, 30),
            "modal {modal}"
        );
    }
}
#[test]
fn cancelled_refresh_does_not_clear_uncertainty_or_start_firmware() {
    let (mut engine, state) = connected_engine();
    state.lock().unwrap().fail_ack = true;
    assert!(engine.execute(request(set())).writes_blocked);
    let r = request(Action::Refresh);
    {
        let mut s = state.lock().unwrap();
        s.calls.clear();
        s.info_count = 0;
        s.cancel = Some(r.cancelled.clone());
        s.cancel_info = Some(1);
    }
    let reply = engine.execute(r);
    assert!(reply.result.is_err());
    assert!(reply.writes_blocked);
    assert_eq!(state.lock().unwrap().calls, ["info"]);
}
#[test]
fn closure_shutdown_guard_cancels_worker_without_waiting() {
    let (started_tx, started_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let mut worker = Worker::spawn(
        GateBackend {
            started: started_tx,
            release: release_rx,
        },
        config(),
    )
    .unwrap();
    let guard = worker.shutdown_guard();
    worker.submit(Action::Paired).unwrap();
    started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    let now = Instant::now();
    drop(guard);
    assert!(now.elapsed() < Duration::from_secs(1));
    release_tx.send(()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match worker.poll() {
            Err(_) => break,
            Ok(reply) => assert!(reply.is_none()),
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
}
