//! The worker is the only owner of the Bluetooth client. A single request is
//! admitted at a time; neither rendering nor key handling performs I/O.
use super::Config;
use crate::features::{Operation, ResultData};
use crate::{
    bluetooth::{self, BluetoothAddress, HostHeadset},
    client::Client,
    i18n::{Lang, L},
    settings::{Setting, StudioProState},
};
use std::{
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError},
        Arc,
    },
    thread,
    time::Duration,
};

pub(super) trait Session: Send {
    fn info(&mut self) -> io::Result<StudioProState>;
    fn firmware(&mut self) -> io::Result<String>;
    fn acknowledge(&mut self, setting: &Setting) -> io::Result<()>;
    fn authorize_setting(&mut self, setting: &Setting) -> io::Result<()> {
        crate::application::validate_setting(setting)
    }
    fn feature(&mut self, _: &Operation, _: &dyn Fn() -> bool) -> io::Result<ResultData> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "session does not implement feature operations",
        ))
    }
}
impl Session for Client<bluetooth::Connection> {
    fn info(&mut self) -> io::Result<StudioProState> {
        Client::info(self)
    }
    fn firmware(&mut self) -> io::Result<String> {
        Client::firmware(self)
    }
    fn feature(
        &mut self,
        operation: &Operation,
        cancelled: &dyn Fn() -> bool,
    ) -> io::Result<ResultData> {
        operation.execute(self, cancelled)
    }
    fn authorize_setting(&mut self, setting: &Setting) -> io::Result<()> {
        crate::application::validate_setting(setting)?;
        let firmware = Client::firmware(self)?;
        crate::application::authorize_setting(
            Some(crate::models::Model::StudioPro),
            Some(&firmware),
            setting,
        )
    }
    fn acknowledge(&mut self, setting: &Setting) -> io::Result<()> {
        if setting.expects_ack() {
            self.request(setting.instruction, &setting.payload)
                .map(|_| ())
        } else {
            self.send(setting)
        }
    }
}
pub(super) trait Backend: Send + 'static {
    fn capabilities(&self) -> crate::models::Capabilities {
        crate::models::Model::StudioPro.capabilities()
    }
    fn reset_cleanup(&mut self, _: &BluetoothAddress) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "backend does not implement reset cleanup",
        ))
    }
    fn host_connection(&mut self, _: &BluetoothAddress, _: bool) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "backend does not implement host link actions",
        ))
    }
    fn audio(&mut self, address: &BluetoothAddress) -> io::Result<Vec<crate::audio::AudioStatus>> {
        use crate::audio::AudioStatusProvider;
        crate::audio::OsAudio.observe(address)
    }
    fn doctor(
        &mut self,
        address: Option<&BluetoothAddress>,
        channel: u8,
        model: Option<crate::models::Model>,
    ) -> crate::diagnostics::DoctorReport {
        crate::diagnostics::collect(address, channel, model)
    }
    fn paired(&mut self) -> io::Result<Vec<HostHeadset>>;
    fn connect(&mut self, config: &Config, address: &str) -> io::Result<Box<dyn Session>>;
}
struct Native;
impl Backend for Native {
    fn capabilities(&self) -> crate::models::Capabilities {
        crate::models::supported_capabilities()
    }
    fn host_connection(&mut self, address: &BluetoothAddress, connected: bool) -> io::Result<()> {
        bluetooth::set_host_connection(address, connected)
    }
    fn reset_cleanup(&mut self, address: &BluetoothAddress) -> io::Result<()> {
        crate::reset::cleanup(address)
    }
    fn paired(&mut self) -> io::Result<Vec<HostHeadset>> {
        bluetooth::list_devices()
    }
    fn connect(&mut self, config: &Config, address: &str) -> io::Result<Box<dyn Session>> {
        let address = BluetoothAddress::parse(address)?;
        bluetooth::Connection::connect(&address, config.channel, config.timeout)
            .map(|io| Box::new(Client::new(io, config.timeout)) as Box<dyn Session>)
    }
}
#[derive(Clone, Debug)]
pub(super) enum Action {
    Paired,
    Doctor {
        address: Option<String>,
        model: Option<crate::models::Model>,
    },
    Audio {
        address: String,
    },
    HostConnection {
        address: String,
        connected: bool,
    },
    HostRefresh {
        address: String,
    },
    Connect {
        address: String,
        model_confirmed: bool,
    },
    Refresh,
    Disconnect,
    Set(Setting),
    Feature(Operation),
    /// App-settings change; the new channel and timeout
    /// apply to the next connection, never to a live one.
    Configure {
        lang: Lang,
        channel: u8,
        timeout_seconds: u64,
    },
}
impl Action {
    pub fn is_write(&self) -> bool {
        matches!(self, Self::Set(_) | Self::HostConnection { .. })
            || matches!(self,Self::Feature(op) if op.is_write())
    }
}
#[derive(Debug)]
pub(super) struct Snapshot {
    pub info: StudioProState,
    pub firmware: Option<String>,
    pub note: Option<String>,
}
#[derive(Debug)]
pub(super) enum Output {
    Paired(Vec<HostHeadset>),
    Doctor(crate::diagnostics::DoctorReport),
    Audio {
        address: BluetoothAddress,
        observations: Vec<crate::audio::AudioStatus>,
    },
    HostChanged {
        address: BluetoothAddress,
        connected: bool,
    },
    HostSnapshot {
        address: BluetoothAddress,
        devices: Vec<HostHeadset>,
    },
    Snapshot(Snapshot),
    Disconnected,
    Verified {
        info: StudioProState,
        setting: Setting,
    },
    Configured,
    Feature(ResultData),
}
pub(super) struct Request {
    pub id: u64,
    pub action: Action,
    pub cancelled: Arc<AtomicBool>,
}
pub(super) struct Reply {
    pub id: u64,
    pub result: Result<Output, String>,
    pub connected: bool,
    pub writes_blocked: bool,
}

/// Cancellation cannot recall a socket write that has already started. Check
/// immediately before each subsequent stage, and keep uncertain state in this
/// worker (not only in the UI) so a stale/duplicate input cannot bypass it.
pub(super) struct Engine<B> {
    backend: B,
    config: Config,
    session: Option<Box<dyn Session>>,
    uncertain: bool,
    shutdown: Arc<AtomicBool>,
}
impl<B: Backend> Engine<B> {
    pub fn new(backend: B, config: Config, shutdown: Arc<AtomicBool>) -> Self {
        Self {
            backend,
            config,
            session: None,
            uncertain: false,
            shutdown,
        }
    }
    fn cancelled(&self, request: &Request) -> bool {
        request.cancelled.load(Ordering::SeqCst) || self.shutdown.load(Ordering::SeqCst)
    }
    fn t(&self) -> &'static L {
        crate::i18n::txt(self.config.lang)
    }
    fn check(&self, request: &Request) -> Result<(), String> {
        if self.cancelled(request) {
            Err(if self.uncertain {
                self.t().err_cancel_uncertain.into()
            } else {
                self.t().err_cancel.into()
            })
        } else {
            Ok(())
        }
    }
    /// Transport loss closes the control session. It never retries a write;
    /// uncertainty survives reconnect until an explicit completed refresh.
    fn with_session<T>(
        &mut self,
        operation: impl FnOnce(&mut dyn Session) -> io::Result<T>,
    ) -> io::Result<T> {
        let missing = self.t().err_gone;
        let result = operation(
            self.session
                .as_mut()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, missing))?
                .as_mut(),
        );
        if result.as_ref().is_err_and(|error| {
            matches!(
                error.kind(),
                io::ErrorKind::UnexpectedEof
                    | io::ErrorKind::BrokenPipe
                    | io::ErrorKind::NotConnected
                    | io::ErrorKind::ConnectionReset
                    | io::ErrorKind::ConnectionAborted
                    | io::ErrorKind::HostUnreachable
                    | io::ErrorKind::NetworkUnreachable
            )
        }) {
            self.session = None;
        }
        result
    }
    fn snapshot(&mut self, request: &Request) -> Result<Snapshot, String> {
        self.check(request)?;
        let info = self
            .with_session(|session| session.info())
            .map_err(|e| self.t().err_status_query.replacen("{}", &e.to_string(), 1))?;
        self.check(request)?;
        let (firmware, note) = match self.with_session(|session| session.firmware()) {
            Ok(value) => (Some(value), None),
            Err(e) if self.session.is_none() => return Err(e.to_string()),
            Err(e) => (
                None,
                Some(
                    self.t()
                        .status_firmware_note
                        .replacen("{}", &e.to_string(), 1),
                ),
            ),
        };
        self.check(request)?;
        Ok(Snapshot {
            info,
            firmware,
            note,
        })
    }
    pub fn execute(&mut self, request: Request) -> Reply {
        let result = self.execute_inner(&request);
        Reply {
            id: request.id,
            result,
            connected: self.session.is_some(),
            writes_blocked: self.uncertain,
        }
    }
    fn execute_inner(&mut self, request: &Request) -> Result<Output, String> {
        let t = self.t();
        self.check(request)?;
        match &request.action {
            Action::Doctor { address, model } => {
                let address = address
                    .as_deref()
                    .map(BluetoothAddress::parse)
                    .transpose()
                    .map_err(|e| e.to_string())?;
                let report = self
                    .backend
                    .doctor(address.as_ref(), self.config.channel, *model);
                self.check(request)?;
                Ok(Output::Doctor(report))
            }
            Action::HostConnection { address, connected } => {
                let address = BluetoothAddress::parse(address).map_err(|e| e.to_string())?;
                self.backend
                    .host_connection(&address, *connected)
                    .map_err(|e| e.to_string())?;
                if !connected
                    && self
                        .config
                        .address
                        .as_deref()
                        .is_some_and(|a| a.eq_ignore_ascii_case(address.as_str()))
                {
                    self.session = None;
                }
                Ok(Output::HostChanged {
                    address,
                    connected: *connected,
                })
            }
            Action::HostRefresh { address } => {
                let address = BluetoothAddress::parse(address).map_err(|e| e.to_string())?;
                let mut devices = self.backend.paired().map_err(|e| e.to_string())?;
                self.check(request)?;
                crate::device::DeviceRegistry::load().annotate(&mut devices);
                Ok(Output::HostSnapshot { address, devices })
            }
            Action::Audio { address } => {
                let address = BluetoothAddress::parse(address).map_err(|e| e.to_string())?;
                let observations = self.backend.audio(&address).map_err(|e| e.to_string())?;
                self.check(request)?;
                Ok(Output::Audio {
                    address,
                    observations,
                })
            }
            Action::Paired => {
                let mut devices = self
                    .backend
                    .paired()
                    .map_err(|e| t.err_paired_list.replacen("{}", &e.to_string(), 1))?;
                self.check(request)?;
                // The OS list never knows the model; the registry does.
                // A fresh load keeps a mid-session confirmation visible.
                crate::device::DeviceRegistry::load().annotate(&mut devices);
                Ok(Output::Paired(devices))
            }
            Action::Connect {
                address,
                model_confirmed,
            } => {
                if !model_confirmed {
                    return Err(t.err_need_model.into());
                }
                if self.session.is_some() {
                    return Err(t.err_connected_already.into());
                }
                validate_address_in(self.config.lang, address)?;
                let session = self
                    .backend
                    .connect(&self.config, address)
                    .map_err(|e| t.err_connect.replacen("{}", &e.to_string(), 1))?;
                self.check(request)?;
                self.session = Some(session);
                self.config.address = Some(address.clone());
                match self.snapshot(request) {
                    Ok(snapshot) => Ok(Output::Snapshot(snapshot)),
                    Err(error) => {
                        self.session = None;
                        Err(error)
                    }
                }
            }
            Action::Refresh => {
                let snapshot = self.snapshot(request)?;
                // Only an explicitly requested, completed refresh clears this.
                self.uncertain = false;
                Ok(Output::Snapshot(snapshot))
            }
            Action::Disconnect => {
                self.session = None;
                Ok(Output::Disconnected)
            }
            Action::Configure {
                lang,
                channel,
                timeout_seconds,
            } => {
                self.config.lang = *lang;
                self.config.channel = *channel;
                self.config.timeout = Duration::from_secs(*timeout_seconds);
                Ok(Output::Configured)
            }
            Action::Feature(operation) => {
                // Both the UI and worker gate operations; queued input cannot
                // bypass the model's capture evidence or uncertain-write lock.
                operation
                    .check(self.backend.capabilities())
                    .map_err(|e| e.to_string())?;
                if operation.is_write() && self.uncertain {
                    return Err(t.err_uncertain_blocked.into());
                }
                if operation.is_write() {
                    self.with_session(|session| session.info())
                        .map_err(|e| t.err_preflight.replacen("{}", &e.to_string(), 1))?;
                }
                self.check(request)?;
                if operation.is_write() {
                    self.uncertain = true;
                }
                let cancelled = request.cancelled.clone();
                let shutdown = self.shutdown.clone();
                let mut result = self
                    .with_session(|session| {
                        session.feature(operation, &|| {
                            cancelled.load(Ordering::SeqCst) || shutdown.load(Ordering::SeqCst)
                        })
                    })
                    .map_err(|e| e.to_string())?;
                if result.reset {
                    // Reset has already happened; close first and complete
                    // host cleanup even if the request was cancelled meanwhile.
                    self.session = None;
                    let address = BluetoothAddress::parse(
                        self.config
                            .address
                            .as_deref()
                            .ok_or(t.status_reset_unavailable)?,
                    )
                    .map_err(|e| e.to_string())?;
                    self.backend
                        .reset_cleanup(&address)
                        .map_err(|e| e.to_string())?;
                    result.text = t.status_reset_done.into();
                    self.uncertain = false;
                } else {
                    self.check(request)?;
                    if operation.is_write() && result.certain {
                        self.uncertain = false;
                    }
                }
                Ok(Output::Feature(result))
            }
            Action::Set(setting) => {
                if self.uncertain {
                    return Err(t.err_uncertain_blocked.into());
                }
                self.check(request)?;
                self.with_session(|session| session.authorize_setting(setting))
                    .map_err(|e| e.to_string())?;
                self.check(request)?;
                let before = self
                    .with_session(|session| session.info())
                    .map_err(|e| t.err_preflight.replacen("{}", &e.to_string(), 1))?;
                self.check(request)?;
                if crate::application::validate_preflight(&before, setting).is_err() {
                    return Err(t.err_preflight_unreadable.replacen("{}", &setting.key, 1));
                }
                // Includes the unavoidable check-to-I/O race: once we pass this
                // barrier cancellation may not stop the already-starting write.
                self.check(request)?;
                self.uncertain = true;
                let noack = !setting.expects_ack();
                self.with_session(|session| session.acknowledge(setting))
                    .map_err(|e| t.err_ack.replacen("{}", &e.to_string(), 1))?;
                self.check(request)?;
                let info = self.with_session(|session| session.info()).map_err(|e| {
                    if noack {
                        t.err_readback_noack
                    } else {
                        t.err_readback
                    }
                    .replacen("{}", &e.to_string(), 1)
                })?;
                self.check(request)?;
                if !setting.matches(&info) {
                    return Err(if noack {
                        t.err_mismatch_noack
                    } else {
                        t.err_mismatch
                    }
                    .replacen("{}", &setting.key, 1)
                    .replacen("{}", &setting.value, 1));
                }
                self.uncertain = false;
                Ok(Output::Verified {
                    info,
                    setting: setting.clone(),
                })
            }
        }
    }
}
pub(super) fn validate_address(address: &str) -> Result<(), String> {
    check_address(address, Lang::En)
}
pub(super) fn validate_address_in(lang: Lang, address: &str) -> Result<(), String> {
    check_address(address, lang)
}
fn check_address(address: &str, lang: Lang) -> Result<(), String> {
    let t = crate::i18n::txt(lang);
    let parts: Vec<_> = address.split(':').collect();
    if parts.len() == 6
        && parts
            .iter()
            .all(|s| s.len() == 2 && s.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        Ok(())
    } else {
        Err(t.status_bad_address.into())
    }
}

pub(super) struct ShutdownGuard(Arc<AtomicBool>);
impl Drop for ShutdownGuard {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

pub(super) struct Worker {
    lang: Lang,
    requests: Option<SyncSender<Request>>,
    replies: Receiver<Reply>,
    shutdown: Arc<AtomicBool>,
    active: Option<(u64, Arc<AtomicBool>)>,
    next_id: u64,
}
impl Worker {
    pub fn native(config: Config) -> io::Result<Self> {
        Self::spawn(Native, config)
    }
    pub fn spawn<B: Backend>(backend: B, config: Config) -> io::Result<Self> {
        let lang = config.lang;
        let (tx, rx) = mpsc::sync_channel::<Request>(1);
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        let shutdown = Arc::new(AtomicBool::new(false));
        let worker_shutdown = shutdown.clone();
        thread::Builder::new()
            .name("ugreen-bluetooth".into())
            .spawn(move || {
                let mut engine = Engine::new(backend, config, worker_shutdown.clone());
                while let Ok(request) = rx.recv() {
                    if worker_shutdown.load(Ordering::SeqCst) {
                        break;
                    }
                    let reply = engine.execute(request);
                    if worker_shutdown.load(Ordering::SeqCst) {
                        break;
                    }
                    // With one outstanding request this queue always has room.
                    // Never block shutdown on a UI which has gone away.
                    if reply_tx.try_send(reply).is_err() {
                        break;
                    }
                }
            })?;
        Ok(Self {
            lang,
            requests: Some(tx),
            replies: reply_rx,
            shutdown,
            active: None,
            next_id: 0,
        })
    }
    pub fn submit(&mut self, action: Action) -> Result<u64, String> {
        let t = crate::i18n::txt(self.lang);
        if self.active.is_some() {
            return Err(t.status_worker_busy.into());
        }
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or(t.status_worker_sequence)?;
        let id = self.next_id;
        let cancelled = Arc::new(AtomicBool::new(false));
        let request = Request {
            id,
            action,
            cancelled: cancelled.clone(),
        };
        let language = if let Action::Configure { lang, .. } = &request.action {
            Some(*lang)
        } else {
            None
        };
        match self
            .requests
            .as_ref()
            .ok_or(t.status_worker_stopped)?
            .try_send(request)
        {
            Ok(()) => {
                if let Some(lang) = language {
                    self.lang = lang;
                }
                self.active = Some((id, cancelled));
                Ok(id)
            }
            Err(TrySendError::Full(_)) => Err(t.status_worker_queue.into()),
            Err(TrySendError::Disconnected(_)) => {
                Err(crate::i18n::txt(self.lang).status_worker_stopped.into())
            }
        }
    }
    pub fn poll(&mut self) -> Result<Option<Reply>, String> {
        match self.replies.try_recv() {
            Ok(reply) => {
                if self.active.as_ref().map(|(id, _)| *id) == Some(reply.id) {
                    self.active = None;
                }
                Ok(Some(reply))
            }
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => {
                Err(crate::i18n::txt(self.lang).status_worker_stopped.into())
            }
        }
    }
    pub fn shutdown_guard(&self) -> ShutdownGuard {
        ShutdownGuard(self.shutdown.clone())
    }
    pub fn cancel(&self) {
        if let Some((_, cancelled)) = &self.active {
            cancelled.store(true, Ordering::SeqCst);
        }
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        self.cancel();
        self.requests.take();
        // Deliberately do not join: native I/O may still be in flight. Its
        // deadline bounds the current stage; shutdown blocks all later stages.
    }
}
