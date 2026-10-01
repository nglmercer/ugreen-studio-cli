//! The worker is the only owner of the Bluetooth client. A single request is
//! admitted at a time; neither rendering nor key handling performs I/O.
use super::Config;
use crate::{
    client::Client,
    settings::{DeviceInfo, Setting},
    transport::{self, Device},
};
use std::{
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError},
        Arc,
    },
    thread,
};

pub(super) trait Session: Send {
    fn info(&mut self) -> io::Result<DeviceInfo>;
    fn firmware(&mut self) -> io::Result<String>;
    fn acknowledge(&mut self, setting: &Setting) -> io::Result<()>;
}
impl Session for Client<transport::Connection> {
    fn info(&mut self) -> io::Result<DeviceInfo> {
        Client::info(self)
    }
    fn firmware(&mut self) -> io::Result<String> {
        Client::firmware(self)
    }
    fn acknowledge(&mut self, setting: &Setting) -> io::Result<()> {
        self.request(setting.instruction, &setting.payload)
            .map(|_| ())
    }
}
pub(super) trait Backend: Send + 'static {
    fn paired(&mut self) -> io::Result<Vec<Device>>;
    fn connect(&mut self, config: &Config, address: &str) -> io::Result<Box<dyn Session>>;
}
struct Native;
impl Backend for Native {
    fn paired(&mut self) -> io::Result<Vec<Device>> {
        transport::list_paired()
    }
    fn connect(&mut self, config: &Config, address: &str) -> io::Result<Box<dyn Session>> {
        transport::Connection::connect(address, config.channel, config.timeout)
            .map(|io| Box::new(Client::new(io, config.timeout)) as Box<dyn Session>)
    }
}
#[derive(Clone, Debug)]
pub(super) enum Action {
    Paired,
    Connect {
        address: String,
        model_confirmed: bool,
    },
    Refresh,
    Disconnect,
    Set(Setting),
}
impl Action {
    pub fn is_write(&self) -> bool {
        matches!(self, Self::Set(_))
    }
}
#[derive(Debug)]
pub(super) struct Snapshot {
    pub info: DeviceInfo,
    pub firmware: Option<String>,
    pub note: Option<String>,
}
#[derive(Debug)]
pub(super) enum Output {
    Paired(Vec<Device>),
    Snapshot(Snapshot),
    Disconnected,
    Verified { info: DeviceInfo, setting: Setting },
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
    fn check(&self, request: &Request) -> Result<(), String> {
        if self.cancelled(request) {
            Err(if self.uncertain {
                "Cancelled later stages. An in-flight write may have completed; explicitly refresh before another write."
            } else { "Cancelled; no further stages will start." }.into())
        } else {
            Ok(())
        }
    }
    fn session(&mut self) -> Result<&mut Box<dyn Session>, String> {
        self.session
            .as_mut()
            .ok_or_else(|| "Disconnected; connect first.".into())
    }
    fn snapshot(&mut self, request: &Request) -> Result<Snapshot, String> {
        self.check(request)?;
        let info = self
            .session()?
            .info()
            .map_err(|e| format!("Status query failed: {e}"))?;
        self.check(request)?;
        let (firmware, note) = match self.session()?.firmware() {
            Ok(value) => (Some(value), None),
            Err(e) => (
                None,
                Some(format!("Settings read; firmware unavailable: {e}")),
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
        self.check(request)?;
        match &request.action {
            Action::Paired => {
                let devices = self
                    .backend
                    .paired()
                    .map_err(|e| format!("Cached paired-device list failed: {e}"))?;
                self.check(request)?;
                Ok(Output::Paired(devices))
            }
            Action::Connect {
                address,
                model_confirmed,
            } => {
                if !model_confirmed {
                    return Err("Confirm Studio Pro HP206 protocol before connecting.".into());
                }
                if self.session.is_some() {
                    return Err("Disconnect before connecting another target.".into());
                }
                validate_address(address)?;
                let session = self
                    .backend
                    .connect(&self.config, address)
                    .map_err(|e| format!("Connection failed: {e}"))?;
                self.check(request)?;
                self.session = Some(session);
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
            Action::Set(setting) => {
                if self.uncertain {
                    return Err(
                        "Writes blocked after an uncertain operation. Explicitly refresh first."
                            .into(),
                    );
                }
                self.check(request)?;
                let before = self
                    .session()?
                    .info()
                    .map_err(|e| format!("Preflight status failed; no setting sent: {e}"))?;
                self.check(request)?;
                if before.value(&setting.key).is_none() {
                    return Err(format!(
                        "Preflight cannot read {}; no setting sent.",
                        setting.key
                    ));
                }
                // Includes the unavoidable check-to-I/O race: once we pass this
                // barrier cancellation may not stop the already-starting write.
                self.check(request)?;
                self.uncertain = true;
                self.session()?.acknowledge(setting).map_err(|e| format!("Setting may have been sent; acknowledgement failed: {e}. Explicitly refresh before another write."))?;
                self.check(request)?;
                let info = self.session()?.info().map_err(|e| format!("Write acknowledged but readback failed: {e}. Explicitly refresh before another write."))?;
                self.check(request)?;
                if !setting.matches(&info) {
                    return Err(format!("Write acknowledged but {} did not match {}. Explicitly refresh before another write.", setting.key, setting.value));
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
    let parts: Vec<_> = address.split(':').collect();
    if parts.len() == 6
        && parts
            .iter()
            .all(|s| s.len() == 2 && s.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        Ok(())
    } else {
        Err("Enter six colon-separated hexadecimal octets, e.g. AA:BB:CC:DD:EE:FF.".into())
    }
}

pub(super) struct ShutdownGuard(Arc<AtomicBool>);
impl Drop for ShutdownGuard {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

pub(super) struct Worker {
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
            requests: Some(tx),
            replies: reply_rx,
            shutdown,
            active: None,
            next_id: 0,
        })
    }
    pub fn submit(&mut self, action: Action) -> Result<u64, String> {
        if self.active.is_some() {
            return Err("An operation is already running; wait or press Esc to cancel.".into());
        }
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or("Request sequence exhausted; restart the interface.")?;
        let id = self.next_id;
        let cancelled = Arc::new(AtomicBool::new(false));
        let request = Request {
            id,
            action,
            cancelled: cancelled.clone(),
        };
        match self
            .requests
            .as_ref()
            .ok_or("Worker stopped")?
            .try_send(request)
        {
            Ok(()) => {
                self.active = Some((id, cancelled));
                Ok(id)
            }
            Err(TrySendError::Full(_)) => {
                Err("Worker queue is full; wait before trying again.".into())
            }
            Err(TrySendError::Disconnected(_)) => {
                Err("Bluetooth worker stopped; restart the interface.".into())
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
                Err("Bluetooth worker stopped; restart the interface.".into())
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
