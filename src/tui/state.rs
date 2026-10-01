use super::{
    worker::{self, Action, Output, Reply},
    Config,
};
use crate::{
    settings::{self, DeviceInfo, Setting},
    transport::Device,
};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::{collections::VecDeque, time::Instant};

pub(super) const LABELS: [&str; 9] = [
    "ANC",
    "Equalizer",
    "Game mode",
    "Spatial audio",
    "Dual connection",
    "Wind reduction",
    "Prompts",
    "Volume-up action",
    "Volume-down action",
];
pub(super) fn choices(index: usize) -> &'static [&'static str] {
    match index {
        0 => &["off", "ultra", "general", "gentle", "adaptive", "ambient"],
        1 => &[
            "classic",
            "jazz",
            "electronic",
            "pop",
            "classical",
            "rock",
            "bass",
            "treble",
        ],
        2..=5 => &["off", "on"],
        6 => &["voice", "beeps"],
        _ => &["none", "next", "previous"],
    }
}
pub(super) enum Intent {
    None,
    Request(Action),
    Cancel,
    Quit,
}
pub(super) struct App {
    pub address: String,
    pub model_confirmed: bool,
    pub channel: u8,
    pub timeout_seconds: u64,
    pub connected: bool,
    pub info: Option<DeviceInfo>,
    pub firmware: Option<String>,
    pub observed_at: Option<Instant>,
    pub stale: bool,
    pub writes_blocked: bool,
    pub selected: usize,
    pub proposals: [Option<String>; 9],
    pub busy: Option<(u64, Action)>,
    pub status: String,
    pub address_edit: Option<String>,
    pub paired: Option<Vec<Device>>,
    pub paired_index: usize,
    pub confirmation: Option<Setting>,
    pub model_confirmation: bool,
    pub quit_confirmation: bool,
    pub help: bool,
    pub help_scroll: u16,
    pub log_open: bool,
    pub log_index: usize,
    pub history: VecDeque<String>,
    started_at: Instant,
    last_logged_status: String,
    disconnect_after: bool,
    worker_alive: bool,
}
impl App {
    pub fn new(config: &Config) -> Self {
        Self {
            address: config.address.clone().unwrap_or_default(),
            model_confirmed: config.model_confirmed,
            channel: config.channel,
            timeout_seconds: config.timeout.as_secs(),
            connected: false,
            info: None,
            firmware: None,
            observed_at: None,
            stale: false,
            writes_blocked: false,
            selected: 0,
            proposals: Default::default(),
            busy: None,
            status:
                "Disconnected. No Bluetooth access yet. Edit address (a) or read paired cache (p)."
                    .into(),
            address_edit: None,
            paired: None,
            paired_index: 0,
            confirmation: None,
            model_confirmation: false,
            quit_confirmation: false,
            help: false,
            help_scroll: 0,
            log_open: false,
            log_index: 0,
            history: VecDeque::new(),
            started_at: Instant::now(),
            last_logged_status: String::new(),
            disconnect_after: false,
            worker_alive: true,
        }
    }
    pub fn record_status(&mut self) {
        if self.status == self.last_logged_status {
            return;
        }
        self.last_logged_status = self.status.clone();
        let at_end = self.log_index >= self.history.len().saturating_sub(1);
        if self.history.len() == 100 {
            self.history.pop_front();
            self.log_index = self.log_index.saturating_sub(1);
        }
        self.history.push_back(format!(
            "[+{}s] {}",
            self.started_at.elapsed().as_secs(),
            super::view::clean(&self.status)
        ));
        if at_end {
            self.log_index = self.history.len().saturating_sub(1);
        }
    }
    pub fn accepted(&mut self, id: u64, action: Action) {
        self.status = match &action {
            Action::Paired => "Reading cached paired devices (no scan)...".into(),
            Action::Connect { .. } => "Connecting, then reading status and firmware...".into(),
            Action::Refresh => "Refreshing status and firmware...".into(),
            Action::Disconnect => "Disconnecting...".into(),
            Action::Set(setting) => format!(
                "Applying {}={} after preflight, then verifying acknowledgement and readback...",
                setting.key, setting.value
            ),
        };
        if matches!(action, Action::Refresh | Action::Set(_)) {
            self.stale = true;
        }
        self.busy = Some((id, action));
    }
    pub fn receive(&mut self, reply: Reply) -> Option<Action> {
        // Responses to an earlier target/request must never overwrite a newer UI.
        if self.busy.as_ref().map(|(id, _)| *id) != Some(reply.id) {
            return None;
        }
        let (_, action) = self.busy.take().expect("matching request exists");
        self.connected = reply.connected;
        self.writes_blocked = reply.writes_blocked;
        match reply.result {
            Ok(Output::Paired(devices)) => {
                self.status = if devices.is_empty() {
                    "No cached paired devices. Pair in OS Bluetooth settings first.".into()
                } else {
                    "Select the intended device and press Enter. Selection does not connect.".into()
                };
                self.paired = Some(devices);
                self.paired_index = 0;
            }
            Ok(Output::Snapshot(snapshot)) => {
                self.info = Some(snapshot.info);
                self.firmware = snapshot.firmware;
                self.observed_at = Some(Instant::now());
                self.stale = false;
                self.proposals = Default::default();
                self.status = snapshot.note.unwrap_or_else(|| {
                    "Status read successfully. Settings shown are device readback.".into()
                });
                if self.writes_blocked {
                    self.status
                        .push_str(" Explicit refresh (r) is still required to unlock writes.");
                }
            }
            Ok(Output::Verified { info, setting }) => {
                self.info = Some(info);
                self.observed_at = Some(Instant::now());
                self.stale = false;
                self.proposals = Default::default();
                self.status = format!(
                    "Verified {}={} by acknowledgement and matching device readback.",
                    setting.key, setting.value
                );
            }
            Ok(Output::Disconnected) => {
                self.clear_snapshot();
                self.status = "Disconnected. No automatic reconnect.".into();
            }
            Err(error) => {
                if !matches!(action, Action::Paired) {
                    self.stale = self.info.is_some();
                }
                if !self.connected {
                    self.clear_snapshot();
                }
                self.status = error;
            }
        }
        if self.disconnect_after {
            self.disconnect_after = false;
            self.paired = None;
            if self.connected {
                return Some(Action::Disconnect);
            }
        }
        None
    }
    fn clear_snapshot(&mut self) {
        self.info = None;
        self.firmware = None;
        self.observed_at = None;
        self.proposals = Default::default();
        self.stale = false;
    }
    pub fn worker_failed(&mut self, error: String) {
        if !self.worker_alive {
            return;
        }
        self.worker_alive = false;
        if self
            .busy
            .as_ref()
            .is_some_and(|(_, action)| action.is_write())
        {
            self.writes_blocked = true;
            self.status =
                format!("{error} An in-flight write may have completed; refresh after restarting.");
        } else {
            self.status = error;
        }
        self.busy = None;
        self.connected = false;
        self.stale = true;
        self.confirmation = None;
    }
    pub fn key(&mut self, key: KeyEvent) -> Intent {
        if key.kind != KeyEventKind::Press {
            return Intent::None;
        }
        let quit = key.code == KeyCode::Char('q')
            || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL));
        // Address entry uses q as text, but Ctrl-C remains a quit action.
        if quit && (self.address_edit.is_none() || key.modifiers.contains(KeyModifiers::CONTROL)) {
            if self.quit_confirmation || self.busy.is_none() {
                return Intent::Quit;
            }
            self.quit_confirmation = true;
            self.confirmation = None;
            self.status = "Cancellation requested. An in-flight write may complete. Press q or Enter to quit without waiting; Esc to stay.".into();
            return Intent::Cancel;
        }
        if self.quit_confirmation {
            return match key.code {
                KeyCode::Enter => Intent::Quit,
                KeyCode::Esc => {
                    self.quit_confirmation = false;
                    Intent::None
                }
                _ => Intent::None,
            };
        }
        if self.log_open {
            match key.code {
                KeyCode::Esc | KeyCode::Char('l') => self.log_open = false,
                KeyCode::Up => self.log_index = self.log_index.saturating_sub(1),
                KeyCode::Down => {
                    self.log_index = (self.log_index + 1).min(self.history.len().saturating_sub(1))
                }
                KeyCode::Home => self.log_index = 0,
                KeyCode::End => self.log_index = self.history.len().saturating_sub(1),
                _ => {}
            }
            return Intent::None;
        }
        if self.help {
            match key.code {
                KeyCode::Esc | KeyCode::Char('?') => self.help = false,
                KeyCode::Up => self.help_scroll = self.help_scroll.saturating_sub(1),
                KeyCode::Down => self.help_scroll = self.help_scroll.saturating_add(1).min(32),
                KeyCode::Home => self.help_scroll = 0,
                _ => {}
            }
            return Intent::None;
        }
        if let Some(edit) = &mut self.address_edit {
            match key.code {
                KeyCode::Esc => self.address_edit = None,
                KeyCode::Enter => match worker::validate_address(edit) {
                    Ok(()) => {
                        self.address = edit.to_ascii_uppercase();
                        self.address_edit = None;
                        self.status =
                            "Address saved locally. Press c to connect explicitly.".into();
                    }
                    Err(error) => self.status = error,
                },
                KeyCode::Backspace => {
                    edit.pop();
                }
                KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => edit.clear(),
                KeyCode::Char(c) if (c.is_ascii_hexdigit() || c == ':') && edit.len() < 17 => {
                    edit.push(c.to_ascii_uppercase());
                }
                _ => {}
            }
            return Intent::None;
        }
        if self.model_confirmation {
            if key.code == KeyCode::Char('y') {
                self.model_confirmed = true;
                self.model_confirmation = false;
                self.status = "Studio Pro HP206 protocol selected by you, not device-identity verified. Press c to connect.".into();
            }
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('n')) {
                self.model_confirmation = false;
            }
            return Intent::None;
        }
        if let Some(setting) = &self.confirmation {
            return match key.code {
                KeyCode::Char('y') => {
                    let setting = setting.clone();
                    self.confirmation = None;
                    if self.can_write() {
                        Intent::Request(Action::Set(setting))
                    } else {
                        Intent::None
                    }
                }
                KeyCode::Esc | KeyCode::Char('n') => {
                    self.confirmation = None;
                    Intent::None
                }
                _ => Intent::None,
            };
        }
        if let Some(devices) = &self.paired {
            match key.code {
                KeyCode::Esc => self.paired = None,
                KeyCode::Up => self.paired_index = self.paired_index.saturating_sub(1),
                KeyCode::Down => {
                    self.paired_index = (self.paired_index + 1).min(devices.len().saturating_sub(1))
                }
                KeyCode::Enter => {
                    if let Some(device) = devices.get(self.paired_index) {
                        if worker::validate_address(&device.address).is_ok() {
                            self.address = device.address.clone();
                            self.status = "Address selected locally. Confirm the intended device, then press c to connect.".into();
                        } else {
                            self.status =
                                "Cached device has an invalid address; edit it manually.".into();
                        }
                    }
                    self.paired = None;
                }
                _ => {}
            }
            return Intent::None;
        }
        if key.code == KeyCode::Esc {
            if self.busy.is_some() {
                self.status = "Cancellation requested; waiting for current stage. An in-flight write may complete. Further stages will be skipped.".into();
                return Intent::Cancel;
            }
            self.proposals[self.selected] = None;
            return Intent::None;
        }
        if key.code == KeyCode::Char('l') {
            self.record_status();
            self.log_open = true;
            self.log_index = self.history.len().saturating_sub(1);
            return Intent::None;
        }
        if key.code == KeyCode::Char('?') {
            self.help = true;
            self.help_scroll = 0;
            return Intent::None;
        }
        if key.code == KeyCode::Char('d') {
            if self.busy.is_some() {
                self.disconnect_after = true;
                self.status = "Cancellation requested; disconnect will follow the current stage. An in-flight write may complete.".into();
                return Intent::Cancel;
            }
            if self.connected {
                return Intent::Request(Action::Disconnect);
            }
        }
        if self.busy.is_some() || !self.worker_alive {
            return Intent::None;
        }
        match key.code {
            KeyCode::Char('a') if !self.connected => self.address_edit = Some(self.address.clone()),
            KeyCode::Char('p') if !self.connected => return Intent::Request(Action::Paired),
            KeyCode::Char('m') if !self.connected => self.model_confirmation = true,
            KeyCode::Char('c') if !self.connected => {
                if let Err(error) = worker::validate_address(&self.address) {
                    self.status = error;
                } else if !self.model_confirmed {
                    self.model_confirmation = true;
                } else {
                    return Intent::Request(Action::Connect {
                        address: self.address.clone(),
                        model_confirmed: true,
                    });
                }
            }
            KeyCode::Char('r') if self.connected => return Intent::Request(Action::Refresh),
            KeyCode::Char('a' | 'p' | 'm') if self.connected => {
                self.status = "Disconnect (d) before changing the target or protocol.".into()
            }
            KeyCode::Char('r') => {
                self.status = "Disconnected; connect (c) before refreshing.".into()
            }
            KeyCode::Up => self.selected = self.selected.saturating_sub(1),
            KeyCode::Down => self.selected = (self.selected + 1).min(settings::KEYS.len() - 1),
            KeyCode::Left | KeyCode::Right if self.can_write() => {
                let values = choices(self.selected);
                let current = self.proposals[self.selected]
                    .clone()
                    .or_else(|| self.info.as_ref()?.value(settings::KEYS[self.selected]));
                if let Some(current) = current {
                    if let Some(index) = values.iter().position(|value| *value == current) {
                        let next = if key.code == KeyCode::Right {
                            (index + 1) % values.len()
                        } else {
                            (index + values.len() - 1) % values.len()
                        };
                        self.proposals[self.selected] = Some(values[next].into());
                        self.status = "Proposed value only. Enter reviews the change; y confirms one write. Esc discards.".into();
                    }
                } else {
                    self.status =
                        "This setting is unavailable in device readback; writing is disabled."
                            .into();
                }
            }
            KeyCode::Enter if self.can_write() => {
                if let Some(value) = &self.proposals[self.selected] {
                    if self
                        .info
                        .as_ref()
                        .and_then(|info| info.value(settings::KEYS[self.selected]))
                        .as_ref()
                        == Some(value)
                    {
                        self.status =
                            "That value already matches the latest readback; no write needed."
                                .into();
                    } else {
                        self.confirmation =
                            Setting::parse(settings::KEYS[self.selected], value).ok();
                    }
                } else {
                    self.status =
                        "Use Left/Right to choose a proposed value before applying.".into();
                }
            }
            KeyCode::Left | KeyCode::Right | KeyCode::Enter => self.status =
                "Changes disabled. Connect and explicitly refresh after any uncertain operation."
                    .into(),
            _ => {}
        }
        Intent::None
    }
    fn can_write(&self) -> bool {
        self.connected
            && self.model_confirmed
            && self.info.is_some()
            && !self.stale
            && !self.writes_blocked
            && self.busy.is_none()
            && self.worker_alive
    }
}
