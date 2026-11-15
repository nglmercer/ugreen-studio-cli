//! Worker engine tests: preflight, acknowledgement, readback, cancellation
//! and uncertain-write lockout, all against fake sessions.
use super::*;
use crate::models::Model;
#[derive(Default)]
struct MockState {
    calls: Vec<String>,
    raw: Vec<u8>,
    info_count: usize,
    fail_info: Option<usize>,
    fail_info_kind: Option<io::ErrorKind>,
    fail_ack: bool,
    fail_firmware: bool,
    wrong_readback: bool,
    cancel: Option<Arc<AtomicBool>>,
    cancel_info: Option<usize>,
    cancel_ack: bool,
    cancel_connect: bool,
    feature_caps: Option<crate::models::Capabilities>,
    feature_certain: Option<bool>,
    fail_feature: bool,
    cancel_feature: bool,
    record_drop: bool,
    fail_cleanup: bool,
}
struct FakeBackend(Arc<Mutex<MockState>>);
struct FakeSession(Arc<Mutex<MockState>>);
impl Backend for FakeBackend {
    fn capabilities(&self) -> crate::models::Capabilities {
        self.0
            .lock()
            .unwrap()
            .feature_caps
            .unwrap_or_else(crate::models::supported_capabilities)
    }
    fn reset_cleanup(&mut self, address: &BluetoothAddress) -> io::Result<()> {
        let mut s = self.0.lock().unwrap();
        s.calls.push(format!("reset-cleanup:{address}"));
        if s.fail_cleanup {
            Err(io::Error::other("mock cleanup failed"))
        } else {
            Ok(())
        }
    }
    fn paired(&mut self) -> io::Result<Vec<HostHeadset>> {
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
impl Drop for FakeSession {
    fn drop(&mut self) {
        let mut s = self.0.lock().unwrap();
        if s.record_drop {
            s.calls.push("close-session".into());
        }
    }
}
impl Session for FakeSession {
    fn feature(
        &mut self,
        op: &crate::features::Operation,
        cancelled: &dyn Fn() -> bool,
    ) -> io::Result<crate::features::ResultData> {
        let mut s = self.0.lock().unwrap();
        s.calls.push("feature".into());
        if s.cancel_feature {
            s.cancel.as_ref().unwrap().store(true, Ordering::SeqCst);
        }
        if s.fail_feature || cancelled() {
            return Err(io::Error::other("mock feature failed/cancelled"));
        }
        let certain = s.feature_certain.unwrap_or(true);
        Ok(crate::features::ResultData {
            text: "mock result".into(),
            codecs: None,
            info: if certain && op.is_write() && !op.is_reset() {
                Some(StudioProState::new(s.raw.clone()).unwrap())
            } else {
                None
            },
            peers: None,
            reset: op.is_reset(),
            certain,
        })
    }
    fn info(&mut self) -> io::Result<StudioProState> {
        let mut s = self.0.lock().unwrap();
        s.calls.push("info".into());
        s.info_count += 1;
        if s.cancel_info == Some(s.info_count) {
            s.cancel.as_ref().unwrap().store(true, Ordering::SeqCst);
        }
        if s.fail_info == Some(s.info_count) {
            return Err(io::Error::new(
                s.fail_info_kind.unwrap_or(io::ErrorKind::Other),
                "mock status failure",
            ));
        }
        StudioProState::new(s.raw.clone()).map_err(io::Error::other)
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
    fn paired(&mut self) -> io::Result<Vec<HostHeadset>> {
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
#[test]
fn feature_worker_gates_before_preflight_and_blocks_uncertain_outcomes() {
    use crate::features::Operation;
    use crate::models::Capabilities as C;
    let (mut engine, state) = connected_engine();
    state.lock().unwrap().feature_caps = Some(Model::StudioPro.capabilities());
    let op = Operation::parse("control volume 8").unwrap();
    assert!(engine
        .execute(request(Action::Feature(op.clone())))
        .result
        .is_err());
    assert!(state.lock().unwrap().calls.is_empty());
    {
        let mut s = state.lock().unwrap();
        s.feature_caps = Some(C::from_bits(C::VOLUME));
        s.feature_certain = Some(false);
    }
    let reply = engine.execute(request(Action::Feature(op.clone())));
    assert!(reply.result.is_ok());
    assert!(reply.writes_blocked);
    let calls = state.lock().unwrap().calls.clone();
    assert!(engine.execute(request(Action::Feature(op))).result.is_err());
    assert_eq!(state.lock().unwrap().calls, calls);
    assert!(!engine.execute(request(Action::Refresh)).writes_blocked);
}
#[test]
fn feature_worker_respects_cancellation_before_and_after_write() {
    use crate::features::Operation;
    use crate::models::Capabilities as C;
    for during_write in [false, true] {
        let (mut engine, state) = connected_engine();
        let req = request(Action::Feature(
            Operation::parse("control volume 8").unwrap(),
        ));
        {
            let mut s = state.lock().unwrap();
            s.feature_caps = Some(C::from_bits(C::VOLUME));
            s.cancel = Some(req.cancelled.clone());
            if during_write {
                s.cancel_feature = true;
            } else {
                s.cancel_info = Some(1);
            }
        }
        let reply = engine.execute(req);
        assert!(reply.result.is_err());
        assert_eq!(reply.writes_blocked, during_write);
        assert_eq!(
            state.lock().unwrap().calls.contains(&"feature".into()),
            during_write
        );
    }
}
#[test]
fn feature_reset_closes_session_before_cleanup_for_actual_connected_address() {
    use crate::features::Operation;
    use crate::models::Capabilities as C;
    for fail_cleanup in [false, true] {
        let (mut engine, state) = engine();
        engine.execute(request(Action::Connect {
            address: "11:22:33:44:55:66".into(),
            model_confirmed: true,
        }));
        {
            let mut s = state.lock().unwrap();
            s.calls.clear();
            s.record_drop = true;
            s.fail_cleanup = fail_cleanup;
            s.feature_caps = Some(C::from_bits(C::FACTORY_RESET));
        }
        let reply = engine.execute(request(Action::Feature(
            Operation::parse("control factory-reset").unwrap(),
        )));
        assert!(!reply.connected);
        assert_eq!(reply.result.is_err(), fail_cleanup);
        let calls = &state.lock().unwrap().calls;
        assert_eq!(
            calls.as_slice(),
            [
                "info",
                "feature",
                "close-session",
                "reset-cleanup:11:22:33:44:55:66"
            ]
        );
    }
}
#[test]
fn feature_parser_rejects_reserved_and_invalid_forms_and_classifies_writes() {
    use crate::features::Operation;
    for input in [
        "",
        "firmware",
        "control codec",
        "control volume 256",
        "control sharing 0 01:02:03:04:05:06",
        "find both 1 sink",
        "find left 31 sink",
        "peers extra",
        "codec\n",
        "é",
    ] {
        assert!(Operation::parse(input).is_err(), "{input}");
    }
    for input in ["peers", "codec", "id", "color", "box-mac"] {
        let op = Operation::parse(input).unwrap();
        assert!(!op.is_write());
        assert!(op.check(Model::StudioPro.capabilities()).is_err());
    }
    for input in [
        "control ldac on",
        "control ai-control off",
        "control factory-reset",
    ] {
        let op = Operation::parse(input).unwrap();
        assert!(op.is_write());
        assert!(op.check(Model::StudioPro.capabilities()).is_err());
    }
}
#[test]
fn terminal_transport_errors_close_session_and_allow_explicit_reconnect() {
    for kind in [
        io::ErrorKind::UnexpectedEof,
        io::ErrorKind::ConnectionReset,
        io::ErrorKind::BrokenPipe,
        io::ErrorKind::NotConnected,
    ] {
        let (mut engine, state) = connected_engine();
        {
            let mut s = state.lock().unwrap();
            s.fail_info = Some(1);
            s.fail_info_kind = Some(kind);
        }
        let reply = engine.execute(request(Action::Refresh));
        assert!(reply.result.is_err());
        assert!(!reply.connected);
        assert!(engine
            .execute(request(Action::Connect {
                address: "AA:BB:CC:DD:EE:FF".into(),
                model_confirmed: true
            }))
            .result
            .is_ok());
    }
}
#[test]
fn reconnect_after_lost_write_readback_never_replays_or_unlocks_the_write() {
    let (mut engine, state) = connected_engine();
    {
        let mut s = state.lock().unwrap();
        s.fail_info = Some(2);
        s.fail_info_kind = Some(io::ErrorKind::UnexpectedEof);
    }
    let reply = engine.execute(request(Action::Set(Setting::parse("game", "on").unwrap())));
    assert!(reply.result.is_err());
    assert!(!reply.connected);
    assert!(reply.writes_blocked);
    assert!(engine
        .execute(request(Action::Connect {
            address: "AA:BB:CC:DD:EE:FF".into(),
            model_confirmed: true
        }))
        .result
        .is_ok());
    assert!(engine
        .execute(request(Action::Set(Setting::parse("game", "off").unwrap())))
        .result
        .is_err());
    assert_eq!(
        state
            .lock()
            .unwrap()
            .calls
            .iter()
            .filter(|call| call.starts_with("ack:"))
            .count(),
        1
    );
    assert!(engine.execute(request(Action::Refresh)).result.is_ok());
    assert!(engine
        .execute(request(Action::Set(Setting::parse("game", "off").unwrap())))
        .result
        .is_ok());
}
#[test]
fn worker_admission_errors_use_the_selected_language() {
    let mut cfg = config();
    cfg.lang = crate::i18n::Lang::Es;
    let state = Arc::new(Mutex::new(MockState::default()));
    let mut worker = Worker::spawn(FakeBackend(state), cfg).unwrap();
    worker.submit(Action::Paired).unwrap();
    assert_eq!(
        worker.submit(Action::Paired).unwrap_err(),
        crate::i18n::txt(crate::i18n::Lang::Es).status_worker_busy
    );
}
