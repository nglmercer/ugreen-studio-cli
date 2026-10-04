use super::{
    worker::{self, Action, Output, Reply},
    Config,
};
use crate::{
    i18n::{Lang, L},
    settings::{self, DeviceInfo, Setting},
    transport::Device,
};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent};
use std::{collections::VecDeque, time::Instant};

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
fn fill(template: &str, args: &[&dyn std::fmt::Display]) -> String {
    let mut out = template.to_owned();
    for arg in args {
        if let Some(pos) = out.find("{}") {
            out.replace_range(pos..pos + 2, &arg.to_string());
        } else {
            break;
        }
    }
    out
}
pub(super) struct App {
    pub lang: Lang,
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
    skip_autoload: bool,
}
impl App {
    pub fn new(config: &Config) -> Self {
        let t = crate::i18n::txt(config.lang);
        Self {
            lang: config.lang,
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
            status: t.status_startup.into(),
            address_edit: None,
            paired: None,
            paired_index: 0,
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
            skip_autoload: config.skip_autoload,
        }
    }
    pub fn t(&self) -> &'static L {
        crate::i18n::txt(self.lang)
    }
    pub fn config_skip_autoload(&self) -> bool {
        self.skip_autoload
    }
    pub fn accepted_loading(&mut self, id: u64) {
        self.status = self.t().status_loading.into();
        self.busy = Some((id, Action::Paired));
    }
    pub fn toggle_lang(&mut self) {
        self.lang = match self.lang {
            Lang::En => Lang::Es,
            Lang::Es => Lang::En,
        };
        self.status = self.t().status_lang.into();
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
        let t = self.t();
        self.status = match &action {
            Action::Paired => t.status_loading.into(),
            Action::Connect { .. } => t.status_connecting.into(),
            Action::Refresh => t.status_refreshing.into(),
            Action::Disconnect => t.status_disconnecting.into(),
            Action::Set(setting) => {
                let noack = !setting.expects_ack();
                let template = if noack {
                    t.status_applying.replace(
                        "verifying acknowledgement and readback",
                        "verifying readback (no ack for spatial audio)",
                    )
                } else {
                    t.status_applying.into()
                };
                let template = if noack && self.lang == Lang::Es {
                    t.status_applying.replace(
                        "verificando confirmación y lectura",
                        "verificando lectura (sin confirmación para audio espacial)",
                    )
                } else {
                    template
                };
                fill(&template, &[&setting.key, &setting.value])
            }
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
        let t = self.t();
        match reply.result {
            Ok(Output::Paired(devices)) => {
                self.status = if devices.is_empty() {
                    t.status_paired_empty.into()
                } else {
                    t.status_paired_pick.into()
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
                self.status = fill(t.status_verified, &[&setting.key, &setting.value]);
            }
            Ok(Output::Disconnected) => {
                self.clear_snapshot();
                self.status = t.status_disconnected.into();
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
    }
    /// Mouse: click selects settings/devices/footer buttons; second click on
    /// the same setting advances its proposal; wheel scrolls; right-click is Esc.
    /// Coordinates are 0-based terminal columns/rows.
    pub fn mouse(&mut self, event: MouseEvent, width: u16, height: u16) -> Intent {
        use crossterm::event::{MouseButton, MouseEventKind};
        match event.kind {
            MouseEventKind::ScrollUp => {
                return self.key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
            }
            MouseEventKind::ScrollDown => {
                return self.key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
            }
            MouseEventKind::Down(MouseButton::Right) => {
                return self.key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
            }
            MouseEventKind::Down(MouseButton::Left) => {}
            _ => return Intent::None,
        }
        let (x, y) = (event.column, event.row);
        if width < super::view::MIN_WIDTH || height < super::view::MIN_HEIGHT {
            return Intent::None;
        }
        // Footer: 3 rows at the bottom; each row holds 6 shortcut slots.
        let footer_top = height.saturating_sub(3);
        if y >= footer_top {
            let row = (y - footer_top) as usize;
            let slot = (x / width.max(1).saturating_div(6).max(1)) as usize;
            let key = self
                .t()
                .footer
                .get(row)
                .and_then(|r| r.get(slot.min(5)))
                .map(|(k, _)| *k)
                .unwrap_or("");
            if key.is_empty() {
                return Intent::None;
            }
            let code = match key {
                "Enter" => KeyCode::Enter,
                "Esc" => KeyCode::Esc,
                "<" => KeyCode::Left,
                ">" => KeyCode::Right,
                c => KeyCode::Char(c.chars().next().unwrap_or('?')),
            };
            return self.key(KeyEvent::new(code, KeyModifiers::NONE));
        }
        // Paired-device dialog rows start at the dialog's list area; map by
        // relative index when the dialog is open.
        if self.paired.is_some() {
            let count = self.paired.as_ref().map(Vec::len).unwrap_or(0);
            // Dialog is centered; approximate list start from height.
            let list_top = height.saturating_sub(count as u16).saturating_sub(4) / 2 + 2;
            if y >= list_top {
                let idx = (y - list_top) as usize;
                if idx < count {
                    self.paired_index = idx;
                    return self.key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
                }
            }
            return Intent::None;
        }
        if self.model_confirmation || self.address_edit.is_some() {
            return Intent::None;
        }
        if self.help || self.log_open {
            return Intent::None;
        }
        // Settings list: header takes 7 rows; row i selects, second click
        // proposes the next value. Applying always needs an explicit Enter.
        if y >= 7 {
            let idx = (y - 7) as usize;
            if idx < settings::KEYS.len() {
                if self.selected == idx && self.busy.is_none() {
                    self.selected = idx;
                    return self.key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
                }
                self.selected = idx;
            }
        }
        Intent::None
    }
    pub fn key(&mut self, key: KeyEvent) -> Intent {
        if key.kind != KeyEventKind::Press {
            return Intent::None;
        }
        let t = self.t();
        let quit = key.code == KeyCode::Char('q')
            || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL));
        // Address entry uses q as text, but Ctrl-C remains a quit action.
        if quit && (self.address_edit.is_none() || key.modifiers.contains(KeyModifiers::CONTROL)) {
            // A lone paired-cache read cannot corrupt device state, so quitting
            // through it needs no confirmation; anything else in flight might
            // still complete a write.
            let harmless = self
                .busy
                .as_ref()
                .is_none_or(|(_, action)| matches!(action, Action::Paired));
            if self.quit_confirmation || harmless {
                return Intent::Quit;
            }
            self.quit_confirmation = true;
            self.status = t.status_quit_confirm.into();
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
                        self.status = self.t().status_address_saved.into();
                    }
                    Err(_) => self.status = self.t().status_bad_address.into(),
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
                self.status = self.t().status_model_on.into();
            }
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('n')) {
                self.model_confirmation = false;
            }
            return Intent::None;
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
                            self.status = self.t().status_device_picked.into();
                        } else {
                            self.status = self.t().status_device_bad.into();
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
                self.status = self.t().status_cancel_busy.into();
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
        if key.code == KeyCode::Char('L') {
            self.toggle_lang();
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
                self.status = self.t().status_cancel_then_disconnect.into();
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
                if worker::validate_address(&self.address).is_err() {
                    self.status = self.t().status_bad_address.into();
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
                self.status = self.t().status_must_disconnect.into()
            }
            KeyCode::Char('r') => self.status = self.t().status_must_connect.into(),
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
                        self.status = self.t().status_proposed.into();
                    }
                } else {
                    self.status = self.t().status_unavailable_setting.into();
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
                        self.status = self.t().status_already_matches.into();
                    } else if let Ok(setting) =
                        Setting::parse_in(self.lang, settings::KEYS[self.selected], value)
                    {
                        // Enter applies directly; the write itself is
                        // preflight-checked, sent and readback-verified.
                        self.proposals[self.selected] = None;
                        return Intent::Request(Action::Set(setting));
                    }
                } else {
                    self.status = self.t().status_pick_first.into();
                }
            }
            KeyCode::Left | KeyCode::Right | KeyCode::Enter => {
                self.status = self.t().status_disabled.into()
            }
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
