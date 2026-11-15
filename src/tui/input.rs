//! Keyboard and mouse handling. Order: quit confirmation, open modal,
//! active search field, global keys, then the active page. Mouse hit
//! testing uses the same geometry functions as the renderer.
use super::{
    app::{categories, choices, fill, App, FeatureForm, Intent, Modal, Page, Review, OPTION_ROWS},
    view,
    worker::{self, Action},
};
use crate::{
    bluetooth::BluetoothAddress,
    device::registry::DeviceRegistry,
    features::{self, lookup, FeatureId, FeatureKind, Operation},
    i18n::Lang,
    settings::{self, Setting},
};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent};
use ratatui::layout::{Position, Rect};

fn contains(rect: Rect, x: u16, y: u16) -> bool {
    rect.contains(Position::new(x, y))
}

impl App {
    pub(super) fn browser_searching(&self) -> bool {
        matches!(self.page, Page::Features | Page::Research) && self.browser().searching
    }

    pub fn key(&mut self, key: KeyEvent) -> Intent {
        if key.kind != KeyEventKind::Press {
            return Intent::None;
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        if ctrl && key.code == KeyCode::Char('c') {
            return self.quit();
        }
        if self.quit_confirmation {
            return match key.code {
                KeyCode::Enter | KeyCode::Char('q') => Intent::Quit,
                KeyCode::Esc => {
                    self.quit_confirmation = false;
                    Intent::None
                }
                _ => Intent::None,
            };
        }
        if let Some(modal) = self.modal.take() {
            if key.code == KeyCode::Char('q') && !modal.takes_text() {
                self.modal = Some(modal);
                return self.quit();
            }
            return self.modal_key(modal, key);
        }
        if self.browser_searching() {
            return self.search_key(key);
        }
        if let Some(intent) = self.global_key(key) {
            return intent;
        }
        self.page_key(key)
    }

    /// Quitting is immediate unless a write may still be in flight.
    fn quit(&mut self) -> Intent {
        let write = self.busy.as_ref().is_some_and(|(_, a)| a.is_write());
        if self.quit_confirmation || !write {
            return Intent::Quit;
        }
        self.quit_confirmation = true;
        self.status = self.t().status_quit_confirm.into();
        Intent::Cancel
    }

    fn global_key(&mut self, key: KeyEvent) -> Option<Intent> {
        let t = self.t();
        let page_count = Page::ALL.len();
        let intent = match key.code {
            KeyCode::F(n @ 1..=8) => self.navigate(Page::ALL[usize::from(n - 1)]),
            KeyCode::Char(']') => self.navigate(Page::ALL[(self.page.index() + 1) % page_count]),
            KeyCode::Char('[') => {
                self.navigate(Page::ALL[(self.page.index() + page_count - 1) % page_count])
            }
            KeyCode::Char('q') => self.quit(),
            KeyCode::Char('?') => {
                self.modal = Some(Modal::Help { scroll: 0 });
                Intent::None
            }
            KeyCode::Char('l') => {
                self.record_status();
                self.modal = Some(Modal::Log {
                    index: self.history.len().saturating_sub(1),
                });
                Intent::None
            }
            KeyCode::Char('s') => {
                self.modal = Some(Modal::Options { index: 0 });
                Intent::None
            }
            KeyCode::Char('L') => {
                self.modal = Some(Modal::Language {
                    index: Lang::ALL.iter().position(|&l| l == self.lang).unwrap_or(0),
                });
                Intent::None
            }
            KeyCode::Esc if self.working() => {
                self.status = t.status_cancel_busy.into();
                Intent::Cancel
            }
            KeyCode::Char('d') if self.working() => {
                self.disconnect_after = true;
                self.status = t.status_cancel_then_disconnect.into();
                Intent::Cancel
            }
            KeyCode::Char('d') => {
                if self.connected() {
                    Intent::Request(Action::Disconnect)
                } else {
                    self.status = t.status_disconnected.into();
                    Intent::None
                }
            }
            _ if !self.idle() => {
                if matches!(
                    key.code,
                    KeyCode::Char(_) | KeyCode::Enter | KeyCode::Left | KeyCode::Right
                ) && self.worker_alive
                {
                    self.status = self.tt().busy_rejected.into();
                }
                // Movement keys still browse while work is in flight.
                return if matches!(
                    key.code,
                    KeyCode::Up
                        | KeyCode::Down
                        | KeyCode::Tab
                        | KeyCode::BackTab
                        | KeyCode::Home
                        | KeyCode::End
                ) {
                    None
                } else {
                    Some(Intent::None)
                };
            }
            KeyCode::Char('a') => {
                if self.connected() {
                    self.status = t.status_must_disconnect.into();
                } else {
                    self.modal = Some(Modal::Address {
                        text: self.address.clone(),
                    });
                }
                Intent::None
            }
            KeyCode::Char('m') => {
                if self.connected() {
                    self.status = t.status_must_disconnect.into();
                } else {
                    self.modal = Some(Modal::Model);
                }
                Intent::None
            }
            KeyCode::Char('c') if !self.connected() => self.connect(),
            KeyCode::Char('p') => self.navigate(Page::Devices),
            KeyCode::Char('r') => self.page_refresh(false),
            _ => return None,
        };
        Some(intent)
    }

    fn page_key(&mut self, key: KeyEvent) -> Intent {
        match self.page {
            Page::Home | Page::Audio | Page::Diagnostics | Page::Peers => self.text_page_key(key),
            Page::Settings => self.settings_key(key),
            Page::Features | Page::Research => self.browser_key(key),
            Page::Devices => self.devices_key(key),
        }
    }

    fn text_page_key(&mut self, key: KeyEvent) -> Intent {
        let max = self.max_scroll.get();
        match key.code {
            KeyCode::Up => self.scroll = self.scroll.saturating_sub(1),
            KeyCode::Down => self.scroll = (self.scroll + 1).min(max),
            KeyCode::PageUp => self.scroll = self.scroll.saturating_sub(10),
            KeyCode::PageDown => self.scroll = (self.scroll + 10).min(max),
            KeyCode::Home => self.scroll = 0,
            KeyCode::End => self.scroll = max,
            KeyCode::Esc if self.page != Page::Home => return self.navigate(Page::Home),
            KeyCode::Char('k') if self.page == Page::Audio => {
                return self.submit_feature(Operation::Codec)
            }
            KeyCode::Char('e') if self.page == Page::Peers => {
                self.select_setting_by_key("dual");
                return self.navigate(Page::Settings);
            }
            KeyCode::Char('R') if self.page == Page::Peers => {
                return if self.connected() {
                    Intent::Request(Action::Refresh)
                } else {
                    self.status = self.t().status_must_connect.into();
                    Intent::None
                };
            }
            KeyCode::Char(slot @ ('1' | '2')) if self.page == Page::Peers => {
                return self.review_close_peer(usize::from(slot as u8 - b'1'));
            }
            _ => {}
        }
        Intent::None
    }

    fn review_close_peer(&mut self, slot: usize) -> Intent {
        let Some(peer) = self.peers.as_ref().and_then(|p| p.slots[slot].as_ref()) else {
            self.status = self.tt().peers_not_queried.into();
            return Intent::None;
        };
        if !self.connected() {
            self.status = self.tt().need_connection.into();
            return Intent::None;
        }
        let form = FeatureForm {
            id: FeatureId::ClosePeer,
            drafts: vec![Some(peer.index.to_string()), Some(peer.address.to_string())],
            field: 0,
        };
        self.modal = Some(Modal::Review(Review::Feature {
            form,
            from_form: false,
        }));
        Intent::None
    }

    // ----- settings page ---------------------------------------------------

    fn settings_key(&mut self, key: KeyEvent) -> Intent {
        let t = self.t();
        match key.code {
            KeyCode::Tab => self.switch_tab((self.tab + 1) % categories().len()),
            KeyCode::BackTab => {
                self.switch_tab((self.tab + categories().len() - 1) % categories().len())
            }
            KeyCode::Up => self.move_selection(false),
            KeyCode::Down => self.move_selection(true),
            KeyCode::Char(c @ '1'..='9') => {
                let visible = &categories()[self.tab];
                let index = usize::from(c as u8 - b'1');
                if let Some(&absolute) = visible.get(index) {
                    self.selected = absolute;
                }
            }
            KeyCode::Esc => {
                if self.proposals[self.selected].take().is_none() {
                    return self.navigate(Page::Home);
                }
            }
            KeyCode::Left | KeyCode::Right if self.can_write() => {
                let values = choices(self.selected);
                let current = self.proposals[self.selected]
                    .clone()
                    .or_else(|| self.current_value(self.selected));
                match current.and_then(|c| values.iter().position(|(n, _)| *n == c)) {
                    Some(index) => {
                        let next = if key.code == KeyCode::Right {
                            (index + 1) % values.len()
                        } else {
                            (index + values.len() - 1) % values.len()
                        };
                        self.propose(self.selected, values[next].0);
                    }
                    None => self.status = t.status_unavailable_setting.into(),
                }
            }
            KeyCode::Enter if self.can_write() => {
                if let Some(value) = self.proposals[self.selected].clone() {
                    self.modal = Some(Modal::Review(Review::Setting {
                        index: self.selected,
                        value,
                    }));
                } else {
                    match self.current_value(self.selected) {
                        Some(current) => {
                            let index = choices(self.selected)
                                .iter()
                                .position(|(n, _)| *n == current)
                                .unwrap_or(0);
                            self.modal = Some(Modal::Values { index });
                        }
                        None => self.status = t.status_unavailable_setting.into(),
                    }
                }
            }
            KeyCode::Left | KeyCode::Right | KeyCode::Enter => {
                self.status = if self.connected() {
                    t.status_disabled.into()
                } else {
                    self.tt().need_connection.into()
                };
            }
            _ => {}
        }
        Intent::None
    }
    /// Draft a value locally. A proposal equal to the device value is no
    /// proposal at all.
    fn propose(&mut self, index: usize, value: &str) {
        if self.current_value(index).as_deref() == Some(value) {
            self.proposals[index] = None;
        } else {
            self.proposals[index] = Some(value.into());
        }
        self.status = self.t().status_proposed.into();
    }
    fn switch_tab(&mut self, tab: usize) {
        self.tab = tab % categories().len();
        self.selected = categories()[self.tab][0];
        self.status = fill(self.t().status_category, &[&self.t().categories[self.tab]]);
    }
    /// Move within the category; at an edge, continue into the next one.
    fn move_selection(&mut self, forward: bool) {
        let groups = categories();
        let visible = &groups[self.tab];
        let Some(position) = visible.iter().position(|&i| i == self.selected) else {
            self.selected = visible[0];
            return;
        };
        if forward && position + 1 < visible.len() {
            self.selected = visible[position + 1];
        } else if !forward && position > 0 {
            self.selected = visible[position - 1];
        } else if forward && self.tab + 1 < groups.len() {
            self.tab += 1;
            self.selected = groups[self.tab][0];
        } else if !forward && self.tab > 0 {
            self.tab -= 1;
            self.selected = *groups[self.tab].last().expect("non-empty category");
        }
    }
    /// Send one reviewed value: preflight, acknowledgement and readback
    /// happen in the worker. The device value itself needs no write.
    fn apply_value(&mut self, index: usize, value: &str) -> Intent {
        if !self.can_write() {
            self.status = self.t().status_disabled.into();
            return Intent::None;
        }
        if self.current_value(index).as_deref() == Some(value) {
            self.status = self.t().status_already_matches.into();
            self.proposals[index] = None;
            return Intent::None;
        }
        match Setting::parse_in(self.lang, settings::KEYS[index], value) {
            Ok(setting) => {
                self.proposals[index] = None;
                Intent::Request(Action::Set(setting))
            }
            Err(_) => {
                self.status = self.t().status_unavailable_setting.into();
                Intent::None
            }
        }
    }

    // ----- feature browsers --------------------------------------------------

    fn search_key(&mut self, key: KeyEvent) -> Intent {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let browser = self.browser_mut();
        match key.code {
            KeyCode::Enter | KeyCode::Esc => browser.searching = false,
            KeyCode::Backspace => {
                browser.search.pop();
                browser.index = 0;
            }
            KeyCode::Char('u') if ctrl => {
                browser.search.clear();
                browser.index = 0;
            }
            KeyCode::Char(c) if !c.is_control() && !ctrl => {
                if browser.search.chars().count() < 64 {
                    browser.search.push(c);
                    browser.index = 0;
                }
            }
            KeyCode::Up | KeyCode::Down => {
                browser.searching = false;
                return self.browser_key(key);
            }
            _ => {}
        }
        Intent::None
    }

    fn browser_key(&mut self, key: KeyEvent) -> Intent {
        let tabs = self.browser_tabs().len();
        let count = self.feature_list().len();
        let browser = self.browser_mut();
        match key.code {
            KeyCode::Up if count > 0 => browser.index = (browser.index + count - 1) % count,
            KeyCode::Down if count > 0 => browser.index = (browser.index + 1) % count,
            KeyCode::Home => browser.index = 0,
            KeyCode::End => browser.index = count.saturating_sub(1),
            KeyCode::Tab | KeyCode::Right => {
                browser.tab = (browser.tab + 1) % tabs;
                browser.index = 0;
            }
            KeyCode::BackTab | KeyCode::Left => {
                browser.tab = (browser.tab + tabs - 1) % tabs;
                browser.index = 0;
            }
            KeyCode::Char('/') => browser.searching = true,
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                browser.search.clear();
                browser.index = 0;
            }
            KeyCode::Esc => {
                if browser.search.is_empty() {
                    return self.navigate(Page::Home);
                }
                browser.search.clear();
                browser.index = 0;
            }
            KeyCode::Enter => {
                if self.width >= view::WIDE_WIDTH {
                    return self.feature_activate();
                }
                if self.selected_feature().is_some() {
                    self.modal = Some(Modal::Details);
                }
            }
            _ => {}
        }
        Intent::None
    }

    /// Activate the selected feature: disabled ones only explain why,
    /// settings hand over to the Settings page, queries run, controls
    /// open their typed form (or the review when they take no input).
    pub(super) fn feature_activate(&mut self) -> Intent {
        let Some(spec) = self.selected_feature() else {
            return Intent::None;
        };
        let t = self.t();
        let tt = self.tt();
        if spec.kind == FeatureKind::OfflineAnalysis {
            self.status = fill(
                t.feature_offline_hint,
                &[&spec.offline_command().unwrap_or("")],
            );
            return Intent::None;
        }
        if !self.enabled(spec) {
            self.status = format!("{}: {}", spec.name(t), tt.provenance_blocked);
            return Intent::None;
        }
        match spec.kind {
            FeatureKind::VerifiedSetting => {
                self.select_setting_by_key(spec.setting.unwrap_or_default());
                self.status = fill(t.status_edit_in_settings, &[&spec.name(t)]);
                self.navigate(Page::Settings)
            }
            FeatureKind::DeviceQuery if spec.id == FeatureId::MultipointPeers => {
                self.navigate(Page::Peers);
                self.page_refresh(false)
            }
            FeatureKind::DeviceQuery => match spec.operation(&[]) {
                Ok(operation) => self.submit_feature(operation),
                Err(error) => {
                    self.status = error;
                    Intent::None
                }
            },
            FeatureKind::DeviceControl => {
                if !self.connected() {
                    self.status = tt.need_connection.into();
                    return Intent::None;
                }
                let form = FeatureForm::new(spec.id);
                self.modal = Some(if spec.arguments.is_empty() {
                    Modal::Review(Review::Feature {
                        form,
                        from_form: false,
                    })
                } else {
                    Modal::Form(form)
                });
                Intent::None
            }
            FeatureKind::OfflineAnalysis => Intent::None,
        }
    }

    // ----- devices page -----------------------------------------------------

    fn devices_key(&mut self, key: KeyEvent) -> Intent {
        let count = self.devices.as_ref().map_or(0, Vec::len);
        match key.code {
            KeyCode::Up => self.device_index = self.device_index.saturating_sub(1),
            KeyCode::Down => {
                self.device_index = (self.device_index + 1).min(count.saturating_sub(1))
            }
            KeyCode::Home => self.device_index = 0,
            KeyCode::End => self.device_index = count.saturating_sub(1),
            KeyCode::Esc => return self.navigate(Page::Home),
            KeyCode::Char(c @ ('h' | 'H')) => {
                if let Some(device) = self.devices.as_ref().and_then(|d| d.get(self.device_index)) {
                    return Intent::Request(Action::HostConnection {
                        address: device.address.to_string(),
                        connected: c == 'h',
                    });
                }
            }
            KeyCode::Enter => return self.pick_device(),
            _ => {}
        }
        Intent::None
    }
    fn pick_device(&mut self) -> Intent {
        let Some(device) = self
            .devices
            .as_ref()
            .and_then(|d| d.get(self.device_index))
            .cloned()
        else {
            return Intent::None;
        };
        if self.connected() {
            if !self.address.eq_ignore_ascii_case(device.address.as_str()) {
                self.status = self.tt().devices_locked.into();
            }
            return Intent::None;
        }
        self.set_target(device.address.as_str().to_owned(), device.model);
        self.host_connection = device.connected;
        if self.model_confirmed {
            return self.connect();
        }
        self.modal = Some(Modal::Model);
        self.status = self.t().status_device_picked.into();
        Intent::None
    }

    // ----- modals -----------------------------------------------------------

    /// Handle a key for the modal taken out of `self.modal`; the modal is
    /// put back unless the key closes or replaces it.
    fn modal_key(&mut self, modal: Modal, key: KeyEvent) -> Intent {
        let t = self.t();
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let keep = |app: &mut Self, modal| {
            app.modal = Some(modal);
            Intent::None
        };
        match modal {
            Modal::Help { scroll } => match key.code {
                KeyCode::Esc | KeyCode::Char('?') => Intent::None,
                KeyCode::Up => keep(
                    self,
                    Modal::Help {
                        scroll: scroll.saturating_sub(1),
                    },
                ),
                KeyCode::Down => keep(
                    self,
                    Modal::Help {
                        scroll: (scroll + 1).min(self.max_scroll.get()),
                    },
                ),
                KeyCode::Home => keep(self, Modal::Help { scroll: 0 }),
                _ => keep(self, Modal::Help { scroll }),
            },
            Modal::Log { index } => {
                let last = self.history.len().saturating_sub(1);
                let index = match key.code {
                    KeyCode::Esc | KeyCode::Char('l') => return Intent::None,
                    KeyCode::Up => index.saturating_sub(1),
                    KeyCode::Down => (index + 1).min(last),
                    KeyCode::Home => 0,
                    KeyCode::End => last,
                    _ => index,
                };
                keep(self, Modal::Log { index })
            }
            Modal::Language { index } => {
                let last = Lang::ALL.len() - 1;
                match key.code {
                    KeyCode::Esc => Intent::None,
                    KeyCode::Up => keep(
                        self,
                        Modal::Language {
                            index: index.saturating_sub(1),
                        },
                    ),
                    KeyCode::Down => keep(
                        self,
                        Modal::Language {
                            index: (index + 1).min(last),
                        },
                    ),
                    KeyCode::Home => keep(self, Modal::Language { index: 0 }),
                    KeyCode::End => keep(self, Modal::Language { index: last }),
                    KeyCode::Enter => self.apply_lang(Lang::ALL[index]),
                    _ => keep(self, Modal::Language { index }),
                }
            }
            Modal::Options { index } => self.options_key(index, key),
            Modal::Values { index } => {
                let values = choices(self.selected);
                let last = values.len().saturating_sub(1);
                match key.code {
                    KeyCode::Esc => Intent::None,
                    KeyCode::Up => keep(
                        self,
                        Modal::Values {
                            index: index.saturating_sub(1),
                        },
                    ),
                    KeyCode::Down => keep(
                        self,
                        Modal::Values {
                            index: (index + 1).min(last),
                        },
                    ),
                    KeyCode::Home => keep(self, Modal::Values { index: 0 }),
                    KeyCode::End => keep(self, Modal::Values { index: last }),
                    KeyCode::Enter => {
                        if let Some((value, _)) = values.get(index) {
                            self.propose(self.selected, value);
                            if let Some(value) = self.proposals[self.selected].clone() {
                                self.modal = Some(Modal::Review(Review::Setting {
                                    index: self.selected,
                                    value,
                                }));
                            } else {
                                self.status = t.status_already_matches.into();
                            }
                        }
                        Intent::None
                    }
                    _ => keep(self, Modal::Values { index }),
                }
            }
            Modal::Address { mut text } => match key.code {
                KeyCode::Esc => Intent::None,
                KeyCode::Enter => match worker::validate_address(&text) {
                    Ok(()) => {
                        let address = text.to_ascii_uppercase();
                        let known = BluetoothAddress::parse(&address)
                            .ok()
                            .and_then(|a| DeviceRegistry::load().model_of(&a));
                        self.set_target(address, known);
                        self.status = t.status_address_saved.into();
                        Intent::None
                    }
                    Err(_) => {
                        self.status = t.status_bad_address.into();
                        keep(self, Modal::Address { text })
                    }
                },
                KeyCode::Backspace => {
                    text.pop();
                    keep(self, Modal::Address { text })
                }
                KeyCode::Char('u') if ctrl => keep(
                    self,
                    Modal::Address {
                        text: String::new(),
                    },
                ),
                KeyCode::Char(c) if (c.is_ascii_hexdigit() || c == ':') && text.len() < 17 => {
                    text.push(c.to_ascii_uppercase());
                    keep(self, Modal::Address { text })
                }
                _ => keep(self, Modal::Address { text }),
            },
            Modal::Model => match key.code {
                KeyCode::Char('y') | KeyCode::Enter => {
                    self.model_confirmed = true;
                    self.status = t.status_model_on.into();
                    if worker::validate_address(&self.address).is_ok() {
                        self.connect()
                    } else {
                        Intent::None
                    }
                }
                KeyCode::Esc | KeyCode::Char('n') => Intent::None,
                _ => keep(self, Modal::Model),
            },
            Modal::Form(form) => self.form_key(form, key),
            Modal::Review(review) => match key.code {
                KeyCode::Enter => self.commit(review),
                KeyCode::Esc => {
                    if let Review::Feature {
                        form,
                        from_form: true,
                    } = review
                    {
                        self.modal = Some(Modal::Form(form));
                    }
                    Intent::None
                }
                _ => keep(self, Modal::Review(review)),
            },
            Modal::Details => match key.code {
                KeyCode::Esc => Intent::None,
                KeyCode::Up | KeyCode::Down => {
                    self.browser_key(key);
                    keep(self, Modal::Details)
                }
                KeyCode::Enter => self.feature_activate(),
                _ => keep(self, Modal::Details),
            },
        }
    }

    fn commit(&mut self, review: Review) -> Intent {
        match review {
            Review::Setting { index, value } => self.apply_value(index, &value),
            Review::Feature { form, .. } => {
                let spec = lookup(form.id).expect("registry entry");
                match spec.operation(&form.drafts) {
                    Ok(operation) => self.submit_feature(operation),
                    Err(error) => {
                        self.status = error;
                        Intent::None
                    }
                }
            }
        }
    }

    fn options_key(&mut self, index: usize, key: KeyEvent) -> Intent {
        let t = self.t();
        let keep = |app: &mut Self, index| {
            app.modal = Some(Modal::Options { index });
        };
        match key.code {
            KeyCode::Esc => {}
            KeyCode::Up => keep(self, index.saturating_sub(1)),
            KeyCode::Down => keep(self, (index + 1).min(OPTION_ROWS - 1)),
            KeyCode::Home => keep(self, 0),
            KeyCode::End => keep(self, OPTION_ROWS - 1),
            KeyCode::Enter if index == 0 => {
                self.modal = Some(Modal::Language {
                    index: Lang::ALL.iter().position(|&l| l == self.lang).unwrap_or(0),
                });
            }
            KeyCode::Enter if index == 3 && !self.connected() => {
                self.modal = Some(Modal::Address {
                    text: self.address.clone(),
                });
            }
            KeyCode::Left | KeyCode::Right if self.idle() => {
                keep(self, index);
                let forward = key.code == KeyCode::Right;
                match index {
                    0 => {
                        let position = Lang::ALL.iter().position(|&l| l == self.lang).unwrap_or(0);
                        let count = Lang::ALL.len();
                        let next = if forward {
                            (position + 1) % count
                        } else {
                            (position + count - 1) % count
                        };
                        return self.apply_lang(Lang::ALL[next]);
                    }
                    1 => {
                        self.channel = if forward {
                            self.channel % 30 + 1
                        } else {
                            (self.channel + 28) % 30 + 1
                        };
                        self.status = fill(t.status_channel_saved, &[&self.channel]);
                    }
                    2 => {
                        self.timeout_seconds = if forward {
                            self.timeout_seconds % 60 + 1
                        } else {
                            (self.timeout_seconds + 58) % 60 + 1
                        };
                        self.status = fill(t.status_timeout_saved, &[&self.timeout_seconds]);
                    }
                    _ => return Intent::None,
                }
                self.save_cache();
                return Intent::Request(self.configuration());
            }
            _ => keep(self, index),
        }
        Intent::None
    }

    fn form_key(&mut self, mut form: FeatureForm, key: KeyEvent) -> Intent {
        let t = self.t();
        let spec = lookup(form.id).expect("registry entry");
        match key.code {
            KeyCode::Esc => return Intent::None,
            KeyCode::Up => form.field = form.field.saturating_sub(1),
            KeyCode::Down | KeyCode::Tab => {
                form.field = (form.field + 1).min(form.drafts.len().saturating_sub(1))
            }
            KeyCode::Left | KeyCode::Right => {
                cycle_argument(&mut form, key.code == KeyCode::Right);
                self.prefill_peer_mac(&mut form);
            }
            KeyCode::Backspace => {
                let draft = &mut form.drafts[form.field];
                match draft.as_mut() {
                    Some(text) if text.len() > 1 && accepts_text(spec, form.field) => {
                        text.pop();
                    }
                    _ => *draft = None,
                }
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                form.drafts[form.field] = None
            }
            KeyCode::Enter => {
                if let Some((argument, _)) = spec
                    .arguments
                    .iter()
                    .zip(&form.drafts)
                    .find(|(_, draft)| draft.is_none())
                {
                    self.status = fill(t.feature_err_required, &[&(argument.label)(t)]);
                } else {
                    match spec.operation(&form.drafts) {
                        Ok(_) => {
                            self.modal = Some(Modal::Review(Review::Feature {
                                form,
                                from_form: true,
                            }));
                            return Intent::None;
                        }
                        Err(error) => self.status = error,
                    }
                }
            }
            KeyCode::Char(c) => form_char(&mut form, c),
            _ => {}
        }
        self.modal = Some(Modal::Form(form));
        Intent::None
    }

    /// After choosing a peer slot, fill an empty MAC field from the last
    /// peer query so the review shows the exact identity being targeted.
    fn prefill_peer_mac(&self, form: &mut FeatureForm) {
        let Some(spec) = lookup(form.id) else { return };
        if spec.arguments.get(form.field).map(|a| a.kind) != Some(features::ArgumentKind::PeerIndex)
        {
            return;
        }
        let Some(slot) = form.drafts[form.field]
            .as_deref()
            .and_then(|v| v.parse::<usize>().ok())
            .and_then(|v| v.checked_sub(1))
        else {
            return;
        };
        let peer = self
            .peers
            .as_ref()
            .and_then(|p| p.slots.get(slot))
            .and_then(Option::as_ref);
        if let (Some(peer), Some(mac)) = (
            peer,
            spec.arguments
                .iter()
                .position(|a| a.kind == features::ArgumentKind::Mac),
        ) {
            form.drafts[mac] = Some(peer.address.to_string());
        }
    }

    // ----- mouse --------------------------------------------------------------

    /// Left click selects (a second click activates), the wheel moves,
    /// right click acts as Esc. Hitboxes come from the renderer's geometry.
    pub fn mouse(&mut self, event: MouseEvent, width: u16, height: u16) -> Intent {
        use crossterm::event::{MouseButton, MouseEventKind};
        let press = |code| KeyEvent::new(code, KeyModifiers::NONE);
        match event.kind {
            MouseEventKind::ScrollUp => return self.key(press(KeyCode::Up)),
            MouseEventKind::ScrollDown => return self.key(press(KeyCode::Down)),
            MouseEventKind::Down(MouseButton::Right) => return self.key(press(KeyCode::Esc)),
            MouseEventKind::Down(MouseButton::Left) => {}
            _ => return Intent::None,
        }
        if width < view::MIN_WIDTH || height < view::MIN_HEIGHT {
            return Intent::None;
        }
        let (x, y) = (event.column, event.row);
        let area = Rect::new(0, 0, width, height);
        let areas = view::areas(self, area);
        for (hint, rect) in &areas.slots {
            if contains(*rect, x, y) {
                return hint.event.map_or(Intent::None, |event| self.key(event));
            }
        }
        if self.quit_confirmation {
            return Intent::None;
        }
        if let Some(modal) = &self.modal {
            return self.modal_click(modal.clone(), area, x, y);
        }
        for (page, rect) in Page::ALL.iter().zip(view::nav_rects(areas.nav, self)) {
            if contains(rect, x, y) {
                return self.navigate(*page);
            }
        }
        match self.page {
            Page::Settings => {
                let (tabs, list, _) = view::settings_areas(areas.body);
                for (index, rect) in view::tab_rects(tabs, &self.t().categories, self.tab) {
                    if contains(rect, x, y) {
                        if index != self.tab {
                            self.switch_tab(index);
                        }
                        return Intent::None;
                    }
                }
                if let Some(row) = view::list_row(list, x, y, 0) {
                    if let Some(&absolute) = categories()[self.tab].get(row) {
                        if self.selected == absolute {
                            return self.key(press(KeyCode::Right));
                        }
                        self.selected = absolute;
                    }
                }
            }
            Page::Features | Page::Research => {
                let wide = width >= view::WIDE_WIDTH;
                let regions = view::browser_areas(areas.body, wide);
                let labels = self.browser_tab_labels();
                for (index, rect) in view::tab_rects(regions.tabs, &labels, self.browser().tab) {
                    if contains(rect, x, y) {
                        let browser = self.browser_mut();
                        browser.tab = index;
                        browser.index = 0;
                        return Intent::None;
                    }
                }
                if contains(regions.search, x, y) {
                    self.browser_mut().searching = true;
                    return Intent::None;
                }
                let offset = self.browser_offset().get();
                if let Some(row) = view::list_row(regions.list, x, y, offset) {
                    if row < self.feature_list().len() {
                        if self.browser().index == row {
                            return self.key(press(KeyCode::Enter));
                        }
                        self.browser_mut().index = row;
                    }
                }
            }
            Page::Devices => {
                let offset = self.devices_offset.get();
                if let Some(row) = view::list_row(areas.body, x, y, offset) {
                    if row < self.devices.as_ref().map_or(0, Vec::len) {
                        if self.device_index == row {
                            return self.key(press(KeyCode::Enter));
                        }
                        self.device_index = row;
                    }
                }
            }
            Page::Home | Page::Peers | Page::Audio | Page::Diagnostics => {}
        }
        Intent::None
    }

    fn modal_click(&mut self, modal: Modal, area: Rect, x: u16, y: u16) -> Intent {
        let rows = match &modal {
            Modal::Values { .. } => choices(self.selected).len(),
            Modal::Language { .. } => Lang::ALL.len(),
            Modal::Options { .. } => OPTION_ROWS,
            _ => return Intent::None,
        };
        let rect = view::list_modal_rect(area, rows);
        let Some(row) = view::list_row(rect, x, y, 0).filter(|row| *row < rows) else {
            return Intent::None;
        };
        let press = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        match modal {
            Modal::Values { .. } => self.modal = Some(Modal::Values { index: row }),
            Modal::Language { .. } => self.modal = Some(Modal::Language { index: row }),
            Modal::Options { .. } => self.modal = Some(Modal::Options { index: row }),
            _ => {}
        }
        self.key(press)
    }
}

fn accepts_text(spec: &features::FeatureSpec, field: usize) -> bool {
    spec.arguments.get(field).is_some_and(|argument| {
        !matches!(
            argument.kind,
            features::ArgumentKind::Toggle
                | features::ArgumentKind::Side
                | features::ArgumentKind::Choice(_)
                | features::ArgumentKind::PeerIndex
        )
    })
}

/// Cycle a toggle, choice or peer field; the draft is always set.
fn cycle_argument(form: &mut FeatureForm, forward: bool) {
    use features::ArgumentKind as K;
    let Some(argument) = lookup(form.id).and_then(|spec| spec.arguments.get(form.field)) else {
        return;
    };
    let draft = &mut form.drafts[form.field];
    let options: &[(&str, u8)] = match argument.kind {
        K::Toggle => &[("on", 1), ("off", 0)],
        K::Side => &[("left", 0), ("right", 1)],
        K::PeerIndex => &[("1", 1), ("2", 2)],
        K::Choice(options) => options,
        K::Seconds | K::Byte | K::Mac | K::AudioOutput | K::Actions(_) => return,
    };
    let position = draft
        .as_deref()
        .and_then(|value| options.iter().position(|(name, _)| *name == value));
    let next = match position {
        Some(position) if forward => (position + 1) % options.len(),
        Some(position) => (position + options.len() - 1) % options.len(),
        None => 0,
    };
    *draft = Some(options[next].0.into());
}

/// Type into the selected field, accepting only what its kind allows.
fn form_char(form: &mut FeatureForm, c: char) {
    use features::ArgumentKind as K;
    let Some(argument) = lookup(form.id).and_then(|spec| spec.arguments.get(form.field)) else {
        return;
    };
    let (allowed, limit) = match argument.kind {
        K::Toggle | K::Side | K::Choice(_) | K::PeerIndex => return,
        K::Seconds => (c.is_ascii_digit(), 2),
        K::Byte => (c.is_ascii_digit(), 3),
        K::Mac => (c.is_ascii_hexdigit() || c == ':', 17),
        K::AudioOutput => (!c.is_control(), 256),
        K::Actions(_) => (c.is_ascii_digit() || c == ',', 30),
    };
    let draft = form.drafts[form.field].get_or_insert_with(String::new);
    if allowed && draft.chars().count() < limit {
        draft.push(if argument.kind == K::Mac {
            c.to_ascii_uppercase()
        } else {
            c
        });
    }
}
