use super::{
    worker::{self, Action, Output, Reply},
    Config,
};
use crate::{
    bluetooth::{BluetoothAddress, HostHeadset},
    device::registry::DeviceRegistry,
    i18n::{Lang, L},
    models::Model,
    settings::{self, Setting, StudioProState},
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
/// Setting categories, as absolute indices into
/// `settings::KEYS`: Audio, Connection, Environment, Feedback.
pub(super) const CATEGORIES: [&[usize]; 4] = [&[0, 1, 2, 3], &[4], &[5], &[6, 7, 8]];
/// App-settings rows: language, channel, timeout, target.
const OPTIONS: usize = 4;
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
    pub info: Option<StudioProState>,
    pub firmware: Option<String>,
    pub observed_at: Option<Instant>,
    pub stale: bool,
    pub writes_blocked: bool,
    pub selected: usize,
    pub proposals: [Option<String>; 9],
    pub tab: usize,
    pub picker: Option<usize>,
    pub busy: Option<(u64, Action)>,
    pub status: String,
    pub address_edit: Option<String>,
    pub paired: Option<Vec<HostHeadset>>,
    pub paired_index: usize,
    pub model_confirmation: bool,
    pub quit_confirmation: bool,
    pub help: bool,
    pub help_scroll: u16,
    pub options: bool,
    pub options_index: usize,
    pub lang_picker: bool,
    pub lang_index: usize,
    pub log_open: bool,
    pub log_index: usize,
    pub history: VecDeque<String>,
    started_at: Instant,
    last_logged_status: String,
    disconnect_after: bool,
    worker_alive: bool,
    skip_autoload: bool,
    autoconnect: bool,
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
            tab: 0,
            picker: None,
            busy: None,
            status: t.status_startup.into(),
            address_edit: None,
            paired: None,
            paired_index: 0,
            model_confirmation: false,
            quit_confirmation: false,
            help: false,
            help_scroll: 0,
            options: false,
            options_index: 0,
            lang_picker: false,
            lang_index: 0,
            log_open: false,
            log_index: 0,
            history: VecDeque::new(),
            started_at: Instant::now(),
            last_logged_status: String::new(),
            disconnect_after: false,
            worker_alive: true,
            skip_autoload: config.skip_autoload,
            autoconnect: config.autoconnect,
        }
    }
    pub fn t(&self) -> &'static L {
        crate::i18n::txt(self.lang)
    }
    pub fn config_skip_autoload(&self) -> bool {
        self.skip_autoload
    }
    pub fn config_autoconnect(&self) -> bool {
        self.autoconnect
    }
    pub fn accepted_loading(&mut self, id: u64) {
        self.status = self.t().status_loading.into();
        self.busy = Some((id, Action::Paired));
    }
    /// Open the app-settings screen. Every other modal
    /// closes first; the language row is selected because
    /// that is the most common change.
    pub fn open_options(&mut self) {
        self.help = false;
        self.log_open = false;
        self.address_edit = None;
        self.model_confirmation = false;
        self.paired = None;
        self.picker = None;
        self.options = true;
        self.options_index = 0;
        self.lang_picker = false;
    }
    /// Select one language directly and remember the choice
    /// in the target cache.
    pub fn apply_lang(&mut self, lang: Lang) {
        self.lang = lang;
        self.status = fill(self.t().status_lang, &[&lang.name()]);
        self.save_cache();
    }
    /// Remember the current target, channel, timeout and
    /// language so the next start can offer them.
    fn save_cache(&self) {
        let mut registry = DeviceRegistry::load();
        if let Ok(address) = BluetoothAddress::parse(&self.address) {
            registry.set_selected(&address);
            registry.set_model(&address, self.model_confirmed.then_some(Model::StudioPro));
        }
        registry.channel = Some(self.channel);
        registry.timeout = Some(self.timeout_seconds);
        registry.lang = Some(self.lang);
        registry.autoconnect = self.config_autoconnect();
        registry.save();
    }
    /// Report and persist a channel or timeout change;
    /// the worker applies it to the next connection.
    fn save_config_change(&mut self, template: &str, args: &[&dyn std::fmt::Display]) {
        self.status = fill(template, args);
        self.save_cache();
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
            // The options screen already set the saved
            // status before submitting the change.
            Action::Configure { .. } => self.status.clone(),
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
            // The status was set when the change was
            // submitted; nothing else to update.
            Ok(Output::Configured) => {}
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
        // Value list: clicking a row applies that value at once.
        if self.picker.is_some() {
            let rect = super::view::picker_rect(
                ratatui::layout::Rect::new(0, 0, width, height),
                choices(self.selected).len(),
            );
            let row = y.saturating_sub(rect.y) as usize;
            if y >= rect.y
                && y < rect.y + rect.height
                && x >= rect.x
                && x < rect.x + rect.width
                && row >= 2
            {
                let values = choices(self.selected);
                if row - 2 < values.len() {
                    return self.apply_value(self.selected, values[row - 2]);
                }
            }
            return Intent::None;
        }
        // App settings: a click selects the row; the
        // language row opens the language list, and a
        // language in that list applies at once.
        if self.lang_picker {
            let rect = super::view::options_rect(
                ratatui::layout::Rect::new(0, 0, width, height),
                1 + Lang::ALL.len(),
            );
            let row = y.saturating_sub(rect.y) as usize;
            if y >= rect.y
                && y < rect.y + rect.height
                && x >= rect.x
                && x < rect.x + rect.width
                && row >= 2
                && row - 2 < Lang::ALL.len()
            {
                self.lang_index = row - 2;
                let lang = Lang::ALL[self.lang_index];
                self.lang_picker = false;
                self.options = false;
                self.apply_lang(lang);
            }
            return Intent::None;
        }
        if self.options {
            let rect = super::view::options_rect(
                ratatui::layout::Rect::new(0, 0, width, height),
                1 + OPTIONS,
            );
            let row = y.saturating_sub(rect.y) as usize;
            if y >= rect.y
                && y < rect.y + rect.height
                && x >= rect.x
                && x < rect.x + rect.width
                && row >= 2
                && row - 2 < OPTIONS
            {
                self.options_index = row - 2;
                if self.options_index == 0 {
                    self.lang_index = Lang::ALL
                        .iter()
                        .position(|&lang| lang == self.lang)
                        .unwrap_or(0);
                    self.lang_picker = true;
                }
            }
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
                "Tab" => KeyCode::Tab,
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
        // Settings list: 7 header rows, 1 tab row, 1 panel
        // border; visible row i selects the category's i-th
        // setting. A second click proposes the next value.
        if y >= 9 {
            let visible = CATEGORIES[self.tab];
            let index = (y - 9) as usize;
            if index < visible.len() {
                let absolute = visible[index];
                if self.selected == absolute && self.busy.is_none() {
                    return self.key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
                }
                self.selected = absolute;
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
        if let Some(index) = self.picker {
            // Value list: pick directly instead of stepping
            // through values with Left/Right.
            let values = choices(self.selected);
            let count = values.len();
            match key.code {
                KeyCode::Esc => self.picker = None,
                KeyCode::Up => self.picker = Some(index.saturating_sub(1)),
                KeyCode::Down => self.picker = Some((index + 1).min(count.saturating_sub(1))),
                KeyCode::Home => self.picker = Some(0),
                KeyCode::End => self.picker = Some(count.saturating_sub(1)),
                KeyCode::Enter => {
                    return self.apply_value(self.selected, values[index]);
                }
                _ => {}
            }
            return Intent::None;
        }
        if self.lang_picker {
            // Language list: choose directly instead of
            // cycling through every installed language.
            let count = Lang::ALL.len();
            match key.code {
                KeyCode::Esc => self.lang_picker = false,
                KeyCode::Up => self.lang_index = self.lang_index.saturating_sub(1),
                KeyCode::Down => self.lang_index = (self.lang_index + 1).min(count - 1),
                KeyCode::Home => self.lang_index = 0,
                KeyCode::End => self.lang_index = count - 1,
                KeyCode::Enter => {
                    let lang = Lang::ALL[self.lang_index];
                    self.lang_picker = false;
                    self.options = false;
                    self.apply_lang(lang);
                }
                _ => {}
            }
            return Intent::None;
        }
        if self.options {
            match key.code {
                KeyCode::Esc => self.options = false,
                KeyCode::Up => self.options_index = self.options_index.saturating_sub(1),
                KeyCode::Down => self.options_index = (self.options_index + 1).min(OPTIONS - 1),
                KeyCode::Home => self.options_index = 0,
                KeyCode::End => self.options_index = OPTIONS - 1,
                KeyCode::Enter if self.options_index == 0 => {
                    self.lang_index = Lang::ALL
                        .iter()
                        .position(|&lang| lang == self.lang)
                        .unwrap_or(0);
                    self.lang_picker = true;
                }
                KeyCode::Left | KeyCode::Right => {
                    let forward = key.code == KeyCode::Right;
                    match self.options_index {
                        0 => {
                            let position = Lang::ALL
                                .iter()
                                .position(|&lang| lang == self.lang)
                                .unwrap_or(0);
                            let next = if forward {
                                (position + 1) % Lang::ALL.len()
                            } else {
                                (position + Lang::ALL.len() - 1) % Lang::ALL.len()
                            };
                            self.apply_lang(Lang::ALL[next]);
                        }
                        1 => {
                            self.channel = if forward {
                                self.channel % 30 + 1
                            } else {
                                (self.channel + 28) % 30 + 1
                            };
                            let template = self.t().status_channel_saved;
                            let channel = self.channel;
                            self.save_config_change(template, &[&channel]);
                            return Intent::Request(Action::Configure {
                                channel: self.channel,
                                timeout_seconds: self.timeout_seconds,
                            });
                        }
                        2 => {
                            self.timeout_seconds = if forward {
                                self.timeout_seconds % 60 + 1
                            } else {
                                (self.timeout_seconds + 58) % 60 + 1
                            };
                            let template = self.t().status_timeout_saved;
                            let timeout = self.timeout_seconds;
                            self.save_config_change(template, &[&timeout]);
                            return Intent::Request(Action::Configure {
                                channel: self.channel,
                                timeout_seconds: self.timeout_seconds,
                            });
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            return Intent::None;
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
                KeyCode::Down => {
                    // The help renders one row per shortcut
                    // line plus a blank row after each
                    // section; generous is fine because
                    // scrolling past the content just
                    // shows the panel border.
                    self.help_scroll = self.help_scroll.saturating_add(1).min(64);
                }
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
            if matches!(key.code, KeyCode::Char('y') | KeyCode::Enter) {
                self.model_confirmed = true;
                self.model_confirmation = false;
                self.status = self.t().status_model_on.into();
                // Confirming the protocol with a valid target
                // connects at once; no second `c` is needed.
                if worker::validate_address(&self.address).is_ok() {
                    return Intent::Request(Action::Connect {
                        address: self.address.clone(),
                        model_confirmed: true,
                    });
                }
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
                        // The listed address is typed and canonical by
                        // construction; nothing here can corrupt the target.
                        let unchanged = self.address.eq_ignore_ascii_case(device.address.as_str());
                        let keep_confirmation = unchanged && self.model_confirmed;
                        self.address = device.address.as_str().to_owned();
                        // Model confirmation is per address, never a
                        // session-wide flag: a pick re-derives it from
                        // the registry-confirmed model of that device,
                        // keeping the session choice only when the
                        // target itself did not change.
                        self.model_confirmed =
                            keep_confirmation || device.model == Some(Model::StudioPro);
                        self.paired = None;
                        // A confirmed protocol connects at
                        // once; the first time, Enter opens
                        // the protocol modal instead.
                        if self.model_confirmed {
                            return Intent::Request(Action::Connect {
                                address: self.address.clone(),
                                model_confirmed: true,
                            });
                        }
                        self.model_confirmation = true;
                        self.status = self.t().status_device_picked.into();
                    } else {
                        self.paired = None;
                    }
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
        if key.code == KeyCode::Char('s') {
            self.open_options();
            return Intent::None;
        }
        if key.code == KeyCode::Char('L') {
            self.open_options();
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
            KeyCode::Tab => self.switch_tab((self.tab + 1) % CATEGORIES.len()),
            KeyCode::BackTab => {
                self.switch_tab((self.tab + CATEGORIES.len() - 1) % CATEGORIES.len())
            }
            KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
                let visible = CATEGORIES[self.tab];
                let index = usize::from(c as u8 - b'0');
                if index <= visible.len() {
                    self.selected = visible[index - 1];
                }
            }
            KeyCode::Up => self.move_selection(false),
            KeyCode::Down => self.move_selection(true),
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
                if let Some(value) = self.proposals[self.selected].clone() {
                    // A proposal applies directly; the write itself is
                    // preflight-checked, sent and readback-verified.
                    return self.apply_value(self.selected, &value);
                }
                // No proposal yet: open the value list so a value can
                // be selected directly instead of stepping with arrows.
                let values = choices(self.selected);
                match self
                    .info
                    .as_ref()
                    .and_then(|info| info.value(settings::KEYS[self.selected]))
                {
                    Some(current) => {
                        let index = values
                            .iter()
                            .position(|value| *value == current)
                            .unwrap_or(0);
                        self.picker = Some(index);
                    }
                    None => self.status = self.t().status_unavailable_setting.into(),
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
    /// Switch the setting category and select its first setting.
    fn switch_tab(&mut self, tab: usize) {
        self.tab = tab % CATEGORIES.len();
        self.selected = CATEGORIES[self.tab][0];
        self.status = fill(self.t().status_category, &[&self.t().categories[self.tab]]);
    }
    /// Move within the active category; at a boundary, cross
    /// into the adjacent category instead of wrapping around.
    fn move_selection(&mut self, forward: bool) {
        let visible = CATEGORIES[self.tab];
        match visible.iter().position(|&index| index == self.selected) {
            Some(position) => {
                let next = if forward {
                    position + 1
                } else {
                    position.wrapping_sub(1)
                };
                if next < visible.len() {
                    self.selected = visible[next];
                    return;
                }
                let target = if forward {
                    self.tab + 1
                } else {
                    self.tab.wrapping_sub(1)
                };
                if target < CATEGORIES.len() {
                    let items = CATEGORIES[target];
                    self.tab = target;
                    self.selected = if forward {
                        items[0]
                    } else {
                        items[items.len() - 1]
                    };
                    self.status = fill(self.t().status_category, &[&self.t().categories[target]]);
                }
            }
            None => {
                // The selection belongs to another category: snap
                // into this one at the edge the motion came from.
                self.selected = if forward {
                    visible[0]
                } else {
                    visible[visible.len() - 1]
                };
            }
        }
    }
    /// Send one value as a write. Writing the value the device
    /// already reports is a no-op; anything else goes through
    /// the worker's preflight, acknowledgement and readback.
    fn apply_value(&mut self, absolute: usize, value: &str) -> Intent {
        self.picker = None;
        if self
            .info
            .as_ref()
            .and_then(|info| info.value(settings::KEYS[absolute]))
            .as_deref()
            == Some(value)
        {
            self.status = self.t().status_already_matches.into();
            return Intent::None;
        }
        match Setting::parse_in(self.lang, settings::KEYS[absolute], value) {
            Ok(setting) => {
                self.proposals[absolute] = None;
                Intent::Request(Action::Set(setting))
            }
            Err(_) => {
                self.status = self.t().status_unavailable_setting.into();
                Intent::None
            }
        }
    }
}
