//! Interface state. Exactly one page is active and at most one modal is
//! open; key handling and rendering both dispatch on those two values, so
//! they cannot disagree about what is on screen. No method performs I/O:
//! device work leaves as an `Intent::Request` for the worker.
use super::{
    worker::{Action, Output, Reply},
    Config,
};
use crate::{
    bluetooth::{BluetoothAddress, ControlSessionState, HostConnectionState, HostHeadset},
    device::registry::DeviceRegistry,
    features::{lookup, FeatureId, Operation},
    i18n::{tui::TuiText, Lang, L},
    models::Model,
    protocol::static_analysis::{Codec, PeerList},
    settings::{self, StudioProState},
};
use std::{cell::Cell, collections::VecDeque, time::Instant};

pub(super) enum Intent {
    None,
    Request(Action),
    Cancel,
    Quit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Page {
    Home,
    Settings,
    Features,
    Devices,
    Peers,
    Audio,
    Diagnostics,
    Research,
}
impl Page {
    pub const ALL: [Self; 8] = [
        Self::Home,
        Self::Settings,
        Self::Features,
        Self::Devices,
        Self::Peers,
        Self::Audio,
        Self::Diagnostics,
        Self::Research,
    ];
    pub const fn index(self) -> usize {
        self as usize
    }
    pub fn label(self, tt: &'static TuiText) -> &'static str {
        tt.pages[self.index()]
    }
}

/// Typed drafts for one registry feature. Nothing is defaulted: every
/// field must be set explicitly before the review can build the operation.
#[derive(Clone, Debug)]
pub(super) struct FeatureForm {
    pub id: FeatureId,
    pub drafts: Vec<Option<String>>,
    pub field: usize,
}
impl FeatureForm {
    pub fn new(id: FeatureId) -> Self {
        let spec = lookup(id).expect("registry entry");
        Self {
            id,
            drafts: vec![None; spec.arguments.len()],
            field: 0,
        }
    }
}

/// A pending write; it is committed only from the review modal.
#[derive(Clone, Debug)]
pub(super) enum Review {
    Setting { index: usize, value: String },
    Feature { form: FeatureForm, from_form: bool },
}

#[derive(Clone, Debug)]
pub(super) enum Modal {
    Help {
        scroll: u16,
    },
    Log {
        index: usize,
    },
    Options {
        index: usize,
    },
    Language {
        index: usize,
    },
    Address {
        text: String,
    },
    Model,
    Values {
        index: usize,
    },
    Form(FeatureForm),
    Review(Review),
    /// Narrow terminals show the selected feature's details here.
    Details,
}
impl Modal {
    /// Modals whose printable keys are text, not shortcuts.
    pub fn takes_text(&self) -> bool {
        matches!(self, Self::Address { .. } | Self::Form(_))
    }
}

/// One registry browser (Features or Research) and its own cursor.
#[derive(Clone, Debug, Default)]
pub(super) struct Browser {
    pub tab: usize,
    pub index: usize,
    pub search: String,
    pub searching: bool,
}

/// App-settings rows: language, channel, timeout and target.
pub(super) const OPTION_ROWS: usize = 4;

pub(super) struct App {
    pub page: Page,
    pub modal: Option<Modal>,
    pub quit_confirmation: bool,
    pub lang: Lang,
    pub address: String,
    pub model_confirmed: bool,
    pub channel: u8,
    pub timeout_seconds: u64,
    /// RFCOMM control session, independent of the OS link.
    pub control_state: ControlSessionState,
    /// OS Bluetooth link of the target, from host enumeration.
    pub host_connection: HostConnectionState,
    pub host_observed_at: Option<Instant>,
    pub info: Option<StudioProState>,
    pub firmware: Option<String>,
    pub observed_at: Option<Instant>,
    pub stale: bool,
    pub writes_blocked: bool,
    pub audio: Option<Vec<crate::audio::AudioStatus>>,
    pub codecs: Option<[(u8, Codec); 2]>,
    pub peers: Option<PeerList>,
    pub diagnostics: Option<crate::diagnostics::DoctorReport>,
    pub devices: Option<Vec<HostHeadset>>,
    pub device_index: usize,
    /// Settings page: absolute index into `settings::KEYS` and its tab.
    pub selected: usize,
    pub tab: usize,
    pub proposals: [Option<String>; 9],
    pub features: Browser,
    pub research: Browser,
    pub last_result: Option<(FeatureId, String)>,
    /// Scroll offset of text pages; the renderer stores the largest
    /// useful offset so keys never scroll into empty space.
    pub scroll: u16,
    pub max_scroll: Cell<u16>,
    /// First visible row of each scrolling list as last rendered; the
    /// mouse handler maps clicks through the same offsets.
    pub devices_offset: Cell<usize>,
    pub features_offset: Cell<usize>,
    pub research_offset: Cell<usize>,
    pub busy: Option<(u64, Action)>,
    /// A request made while a background refresh was in flight; it is
    /// submitted as soon as that refresh is cancelled and answers.
    pub queued: Option<Action>,
    pub status: String,
    pub history: VecDeque<String>,
    pub width: u16,
    pub height: u16,
    pub worker_alive: bool,
    started_at: Instant,
    last_logged_status: String,
    pub(super) disconnect_after: bool,
    skip_autoload: bool,
    autoconnect: bool,
}

pub(super) fn fill(template: &str, args: &[&dyn std::fmt::Display]) -> String {
    let mut out = template.to_owned();
    let mut from = 0;
    for arg in args {
        let Some(pos) = out[from..].find("{}").map(|p| p + from) else {
            break;
        };
        let value = arg.to_string();
        out.replace_range(pos..pos + 2, &value);
        from = pos + value.len();
    }
    out
}

/// Setting categories as absolute indices into `settings::KEYS`.
pub(super) fn categories() -> &'static [Vec<usize>] {
    static CACHED: std::sync::OnceLock<Vec<Vec<usize>>> = std::sync::OnceLock::new();
    CACHED.get_or_init(settings::category_groups)
}
/// Value choices for one setting, from its declarative spec.
pub(super) fn choices(index: usize) -> &'static [(&'static str, u8)] {
    settings::spec(index).map(|spec| spec.values).unwrap_or(&[])
}

impl App {
    pub fn new(config: &Config) -> Self {
        let t = crate::i18n::txt(config.lang);
        Self {
            page: Page::Home,
            modal: None,
            quit_confirmation: false,
            lang: config.lang,
            address: config.address.clone().unwrap_or_default(),
            model_confirmed: config.model_confirmed,
            channel: config.channel,
            timeout_seconds: config.timeout.as_secs(),
            control_state: ControlSessionState::Disconnected,
            host_connection: HostConnectionState::Unknown,
            host_observed_at: None,
            info: None,
            firmware: None,
            observed_at: None,
            stale: false,
            writes_blocked: false,
            audio: None,
            codecs: None,
            peers: None,
            diagnostics: None,
            devices: None,
            device_index: 0,
            selected: 0,
            tab: 0,
            proposals: Default::default(),
            features: Browser::default(),
            research: Browser::default(),
            last_result: None,
            scroll: 0,
            max_scroll: Cell::new(0),
            devices_offset: Cell::new(0),
            features_offset: Cell::new(0),
            research_offset: Cell::new(0),
            busy: None,
            queued: None,
            status: t.status_startup.into(),
            history: VecDeque::new(),
            width: 80,
            height: 24,
            worker_alive: true,
            started_at: Instant::now(),
            last_logged_status: String::new(),
            disconnect_after: false,
            skip_autoload: config.skip_autoload,
            autoconnect: config.autoconnect,
        }
    }
    pub fn t(&self) -> &'static L {
        crate::i18n::txt(self.lang)
    }
    pub fn tt(&self) -> &'static TuiText {
        crate::i18n::tui::text(self.lang)
    }
    pub(super) fn operation_capabilities(&self) -> crate::models::Capabilities {
        crate::models::supported_capabilities()
    }

    /// The first request of a session. Startup never writes settings: a
    /// cached, confirmed target connects when auto-connect is on; without
    /// a target the Devices page opens and reads the paired cache.
    pub fn startup(&mut self) -> Intent {
        if self.skip_autoload {
            return Intent::None;
        }
        if self.autoconnect && !self.address.is_empty() && self.model_confirmed {
            return self.connect();
        }
        if self.address.is_empty() {
            return self.navigate(Page::Devices);
        }
        Intent::None
    }

    // ----- busy state -------------------------------------------------

    /// The periodic host refresh runs in the background: it never locks
    /// the interface and is cancelled when the user asks for something.
    pub fn background(&self) -> bool {
        matches!(self.busy, Some((_, Action::HostRefresh { .. })))
    }
    /// A user-visible operation is running or waiting to run.
    pub fn working(&self) -> bool {
        (self.busy.is_some() && !self.background()) || self.queued.is_some()
    }
    pub fn idle(&self) -> bool {
        self.worker_alive && !self.working()
    }
    pub fn connected(&self) -> bool {
        self.control_state.is_connected()
    }

    pub fn accepted(&mut self, id: u64, action: Action) {
        if matches!(action, Action::Connect { .. }) {
            self.control_state = ControlSessionState::Connecting;
        }
        let t = self.t();
        let status = match &action {
            Action::Doctor { .. } => Some(t.host_commands[3].to_owned()),
            Action::HostConnection { connected, .. } => {
                Some(t.host_commands[if *connected { 4 } else { 5 }].to_owned())
            }
            Action::Audio { .. } => Some(t.host_commands[0].to_owned()),
            Action::Paired => Some(t.status_loading.to_owned()),
            Action::Connect { .. } => Some(t.status_connecting.to_owned()),
            Action::Refresh => Some(t.status_refreshing.to_owned()),
            Action::Disconnect => Some(t.status_disconnecting.to_owned()),
            Action::Feature(_) => Some(t.status_feature_running.to_owned()),
            Action::Set(setting) => Some(fill(
                if setting.expects_ack() {
                    t.status_applying
                } else {
                    t.status_applying_noack
                },
                &[&setting.key, &setting.value],
            )),
            // Preferences set their status before submitting.
            Action::HostRefresh { .. } | Action::Configure { .. } => None,
        };
        if let Some(status) = status {
            self.status = status;
        }
        if matches!(action, Action::Refresh | Action::Set(_)) || action.is_write() {
            self.stale = true;
        }
        if matches!(
            action,
            Action::Feature(Operation::Peers) | Action::Connect { .. }
        ) {
            self.peers = None;
        }
        if action.is_write() {
            self.codecs = None;
            self.peers = None;
        }
        self.busy = Some((id, action));
    }

    /// Apply one worker reply. Replies to an older request are ignored so
    /// they can never overwrite a newer screen. Returns a follow-up action.
    pub fn receive(&mut self, reply: Reply) -> Option<Action> {
        if self.busy.as_ref().map(|(id, _)| *id) != Some(reply.id) {
            return None;
        }
        let (_, action) = self.busy.take().expect("matching request exists");
        let was_connected = self.connected();
        self.control_state = if reply.connected {
            ControlSessionState::Connected
        } else if reply.result.is_err() && matches!(action, Action::Connect { .. }) {
            ControlSessionState::Error
        } else {
            ControlSessionState::Disconnected
        };
        self.writes_blocked = reply.writes_blocked;
        if !was_connected && self.connected() {
            // The device answered: remember it, with its model, for the
            // next start's auto-connect.
            self.save_cache();
        }
        let t = self.t();
        let mut follow_up = None;
        match reply.result {
            Ok(Output::Doctor(report)) => {
                if report
                    .address
                    .as_ref()
                    .is_none_or(|a| self.address.eq_ignore_ascii_case(a.as_str()))
                {
                    self.diagnostics = Some(report);
                }
                self.status = t.host_commands[3].into();
            }
            Ok(Output::HostChanged { address, connected }) => {
                if self.address.eq_ignore_ascii_case(address.as_str()) {
                    self.host_connection = HostConnectionState::Unknown;
                    self.host_observed_at = None;
                    if !connected {
                        self.clear_snapshot();
                    }
                }
                self.status = t.host_commands[if connected { 4 } else { 5 }].into();
                follow_up = Some(Action::HostRefresh {
                    address: address.to_string(),
                });
            }
            Ok(Output::HostSnapshot { address, devices }) => {
                if self.address.eq_ignore_ascii_case(address.as_str()) {
                    self.host_connection = devices
                        .iter()
                        .find(|d| d.address == address)
                        .map_or(HostConnectionState::Unknown, |d| d.connected);
                    self.host_observed_at = Some(Instant::now());
                    if self.devices.is_some() {
                        self.set_devices(devices);
                    }
                }
            }
            Ok(Output::Audio {
                address,
                observations,
            }) => {
                if self.address.eq_ignore_ascii_case(address.as_str()) {
                    self.audio = Some(observations);
                }
                self.status = t.host_commands[0].into();
            }
            Ok(Output::Paired(devices)) => {
                self.status = if devices.is_empty() {
                    t.status_paired_empty.into()
                } else {
                    t.status_paired_pick.into()
                };
                self.set_devices(devices);
                self.host_connection = self
                    .devices
                    .iter()
                    .flatten()
                    .find(|d| d.address.as_str().eq_ignore_ascii_case(&self.address))
                    .map_or(HostConnectionState::Unknown, |d| d.connected);
                self.host_observed_at = Some(Instant::now());
            }
            Ok(Output::Snapshot(snapshot)) => {
                self.info = Some(snapshot.info);
                self.firmware = snapshot.firmware;
                self.observed_at = Some(Instant::now());
                self.stale = false;
                self.proposals = Default::default();
                self.status = snapshot.note.unwrap_or_else(|| t.status_read_ok.into());
                if self.writes_blocked {
                    self.status.push_str(t.status_blocked_note);
                }
            }
            Ok(Output::Verified { info, setting }) => {
                self.info = Some(info);
                self.observed_at = Some(Instant::now());
                self.stale = false;
                self.proposals = Default::default();
                self.status = fill(
                    if setting.expects_ack() {
                        t.status_verified
                    } else {
                        t.status_verified_noack
                    },
                    &[&setting.key, &setting.value],
                );
            }
            Ok(Output::Feature(result)) => {
                if let Some(peers) = result.peers {
                    self.peers = Some(peers);
                }
                if let Some(codecs) = result.codecs {
                    self.codecs = Some(codecs);
                }
                if let Some(info) = result.info {
                    self.info = Some(info);
                    self.observed_at = Some(Instant::now());
                    self.stale = false;
                    self.proposals = Default::default();
                } else if action.is_write() {
                    self.stale = self.info.is_some();
                }
                if result.reset {
                    self.clear_snapshot();
                    self.model_confirmed = false;
                    self.host_connection = HostConnectionState::Unknown;
                }
                if let Action::Feature(operation) = &action {
                    if let Some(id) = operation.id() {
                        self.last_result = Some((id, result.text.clone()));
                    }
                }
                self.status = result.text;
            }
            Ok(Output::Disconnected) => {
                let audio = self.audio.take();
                self.clear_snapshot();
                self.audio = audio;
                self.status = t.status_disconnected.into();
            }
            Ok(Output::Configured) => {}
            // A failed or cancelled background refresh changes nothing the
            // user asked for; it is retried on the next tick.
            Err(_) if matches!(action, Action::HostRefresh { .. }) => {}
            Err(error) => {
                if !matches!(
                    action,
                    Action::Paired | Action::Audio { .. } | Action::Doctor { .. }
                ) {
                    self.stale = self.info.is_some();
                }
                if !self.connected() {
                    self.clear_snapshot();
                }
                if let Action::Feature(Operation::Peers) = action {
                    self.peers = None;
                }
                self.status = error;
            }
        }
        if self.disconnect_after {
            self.disconnect_after = false;
            if self.connected() {
                return Some(Action::Disconnect);
            }
        }
        follow_up
    }

    fn set_devices(&mut self, mut devices: Vec<HostHeadset>) {
        let keep = self
            .devices
            .as_ref()
            .and_then(|list| list.get(self.device_index))
            .map(|device| device.address.clone());
        let selected = BluetoothAddress::parse(&self.address).ok();
        crate::bluetooth::sort_devices(&mut devices, selected.as_ref());
        self.device_index = keep
            .and_then(|address| devices.iter().position(|d| d.address == address))
            .unwrap_or(0);
        self.devices = Some(devices);
    }

    pub(super) fn clear_snapshot(&mut self) {
        self.diagnostics = None;
        self.audio = None;
        self.info = None;
        self.firmware = None;
        self.codecs = None;
        self.peers = None;
        self.observed_at = None;
        self.proposals = Default::default();
        self.stale = false;
    }

    pub fn worker_failed(&mut self, error: String) {
        if !self.worker_alive {
            return;
        }
        self.worker_alive = false;
        let write = self.busy.as_ref().is_some_and(|(_, a)| a.is_write());
        if write {
            self.writes_blocked = true;
            self.status = format!("{error} {}", self.t().status_inflight_write);
        } else {
            self.status = error;
        }
        self.busy = None;
        self.queued = None;
        self.control_state = ControlSessionState::Disconnected;
        self.stale = true;
    }

    /// Append the status line to the bounded session log when it changes.
    pub fn record_status(&mut self) {
        if self.status == self.last_logged_status {
            return;
        }
        self.last_logged_status = self.status.clone();
        if self.history.len() == 100 {
            self.history.pop_front();
        }
        self.history.push_back(format!(
            "[+{}s] {}",
            self.started_at.elapsed().as_secs(),
            super::view::clean(&self.status)
        ));
        if let Some(Modal::Log { index }) = &mut self.modal {
            *index = (*index).min(self.history.len().saturating_sub(1));
        }
    }

    // ----- preferences --------------------------------------------------

    pub(super) fn configuration(&self) -> Action {
        Action::Configure {
            lang: self.lang,
            channel: self.channel,
            timeout_seconds: self.timeout_seconds,
        }
    }
    pub fn apply_lang(&mut self, lang: Lang) -> Intent {
        // The worker must learn each language change before the next one.
        if !self.idle() {
            self.status = self.tt().busy_rejected.into();
            return Intent::None;
        }
        self.lang = lang;
        self.status = fill(self.t().status_lang, &[&lang.name()]);
        self.save_cache();
        Intent::Request(self.configuration())
    }
    /// Remember target, model, channel, timeout and language.
    pub(super) fn save_cache(&self) {
        let mut registry = DeviceRegistry::load();
        if let Ok(address) = BluetoothAddress::parse(&self.address) {
            registry.set_selected(&address);
            registry.set_model(&address, self.model_confirmed.then_some(Model::StudioPro));
        }
        registry.channel = Some(self.channel);
        registry.timeout = Some(self.timeout_seconds);
        registry.lang = Some(self.lang);
        registry.autoconnect = self.autoconnect;
        registry.save();
    }

    /// Change the target. Model confirmation belongs to an address: a new
    /// target is confirmed only if the registry already knows its model.
    pub(super) fn set_target(&mut self, address: String, known_model: Option<Model>) {
        if self.address.eq_ignore_ascii_case(&address) {
            self.model_confirmed |= known_model == Some(Model::StudioPro);
            self.address = address;
            return;
        }
        self.address = address;
        self.model_confirmed = known_model == Some(Model::StudioPro);
        self.clear_snapshot();
        self.last_result = None;
        self.host_connection = HostConnectionState::Unknown;
        self.host_observed_at = None;
        if let Some(devices) = &self.devices {
            if let Some(device) = devices
                .iter()
                .find(|d| d.address.as_str().eq_ignore_ascii_case(&self.address))
            {
                self.host_connection = device.connected;
            }
        }
    }

    // ----- shared actions ------------------------------------------------

    /// Connect to the current target, asking for the protocol first.
    pub(super) fn connect(&mut self) -> Intent {
        if super::worker::validate_address(&self.address).is_err() {
            self.status = if self.address.is_empty() {
                self.tt().need_address.into()
            } else {
                self.t().status_bad_address.into()
            };
            return Intent::None;
        }
        if !self.model_confirmed {
            self.modal = Some(Modal::Model);
            return Intent::None;
        }
        Intent::Request(Action::Connect {
            address: self.address.clone(),
            model_confirmed: true,
        })
    }

    /// Submit a typed feature operation through the same guards the
    /// settings page uses: capability, connection, busy and write lockout.
    pub(super) fn submit_feature(&mut self, operation: Operation) -> Intent {
        if let Err(error) = operation.check(self.operation_capabilities()) {
            self.status = error.to_string();
            return Intent::None;
        }
        if !self.connected() || !self.model_confirmed {
            self.status = self.tt().need_connection.into();
            return Intent::None;
        }
        if !self.idle() {
            self.status = self.tt().busy_rejected.into();
            return Intent::None;
        }
        if operation.is_write() && self.writes_blocked {
            self.status = self.t().status_blocked_note.trim().into();
            return Intent::None;
        }
        Intent::Request(Action::Feature(operation))
    }

    pub(super) fn can_write(&self) -> bool {
        self.connected()
            && self.model_confirmed
            && crate::models::capabilities_for(Some(Model::StudioPro), self.firmware.as_deref())
                .bits()
                != 0
            && self.info.is_some()
            && !self.stale
            && !self.writes_blocked
            && self.idle()
    }

    /// Switch page. Pages that show an observation read it on entry when
    /// the worker is free; rendering itself never performs I/O.
    pub fn navigate(&mut self, page: Page) -> Intent {
        if self.modal.is_some() {
            return Intent::None;
        }
        if self.page != page {
            self.scroll = 0;
        }
        self.page = page;
        if !self.idle() {
            return Intent::None;
        }
        self.page_refresh(true)
    }
    /// The page's own read: on entry (`entering`) only when nothing is
    /// cached yet for the observation pages that can be expensive.
    pub(super) fn page_refresh(&mut self, entering: bool) -> Intent {
        match self.page {
            Page::Devices => Intent::Request(Action::Paired),
            Page::Audio => {
                if super::worker::validate_address(&self.address).is_err() {
                    if !entering {
                        self.status = self.tt().need_address.into();
                    }
                    return Intent::None;
                }
                Intent::Request(Action::Audio {
                    address: self.address.clone(),
                })
            }
            Page::Diagnostics => Intent::Request(Action::Doctor {
                address: BluetoothAddress::parse(&self.address)
                    .ok()
                    .map(|a| a.to_string()),
                model: self.model_confirmed.then_some(Model::StudioPro),
            }),
            Page::Peers => {
                if entering && (!self.connected() || self.peers.is_some()) {
                    return Intent::None;
                }
                self.submit_feature(Operation::Peers)
            }
            Page::Home | Page::Settings | Page::Features | Page::Research => {
                if entering {
                    return Intent::None;
                }
                if self.connected() {
                    Intent::Request(Action::Refresh)
                } else {
                    self.status = self.t().status_must_connect.into();
                    Intent::None
                }
            }
        }
    }

    /// Select the settings row that edits a verified setting key.
    pub(super) fn select_setting_by_key(&mut self, key: &str) {
        if let Some(index) = settings::KEYS.iter().position(|k| *k == key) {
            self.selected = index;
            if let Some(tab) = categories().iter().position(|g| g.contains(&index)) {
                self.tab = tab;
            }
        }
    }
    pub(super) fn browser(&self) -> &Browser {
        if self.page == Page::Research {
            &self.research
        } else {
            &self.features
        }
    }
    pub(super) fn browser_offset(&self) -> &Cell<usize> {
        if self.page == Page::Research {
            &self.research_offset
        } else {
            &self.features_offset
        }
    }
    pub(super) fn browser_mut(&mut self) -> &mut Browser {
        if self.page == Page::Research {
            &mut self.research
        } else {
            &mut self.features
        }
    }
    pub(super) fn current_value(&self, index: usize) -> Option<String> {
        self.info.as_ref()?.value(settings::KEYS[index])
    }
    pub(super) fn dual_value(&self) -> Option<String> {
        self.info.as_ref()?.value("dual")
    }
}
