//! Typed feature forms, proposals and setting selection.
use super::*;

impl App {
    /// Activate the selected feature. Pending features only
    /// explain their state; verified settings hand editing
    /// back to the settings screen; queries and typed
    /// controls go through the worker's guards.
    pub(super) fn feature_activate(&mut self) -> Intent {
        let Some(spec) = self.selected_feature() else {
            return Intent::None;
        };
        if spec.id == features::FeatureId::MultipointPeers {
            return self.open_page(3);
        }
        let t = self.t();
        if !spec.verified(self.operation_capabilities()) {
            // Pending verification never opens a live
            // operation; the explanation says so.
            self.status = format!("{} - {}", t.feature_pending, t.feature_pending_reason);
            if self.width < super::super::view::WIDE_WIDTH {
                self.feature_details = true;
            }
            return Intent::None;
        }
        match spec.kind {
            FeatureKind::OfflineAnalysis => {
                let command = spec.offline_command().unwrap_or_default();
                self.status = fill(t.feature_offline_hint, &[&command]);
                Intent::None
            }
            FeatureKind::VerifiedSetting => {
                let key = spec.setting.unwrap_or_default();
                self.select_setting_by_key(key);
                self.features = false;
                self.route = super::super::routing::Route::Settings;
                self.feature_details = false;
                self.feature_search.clear();
                self.status = fill(t.status_edit_in_settings, &[&spec.name(t)]);
                Intent::None
            }
            FeatureKind::DeviceQuery => match spec.operation(&[]) {
                Ok(operation) => self.submit_feature(operation),
                Err(error) => {
                    self.status = error;
                    Intent::None
                }
            },
            FeatureKind::DeviceControl => {
                if spec.arguments.is_empty() {
                    self.review = Some(Review::Feature {
                        form: FeatureForm::new(spec.id),
                    });
                } else {
                    self.feature_form = Some(FeatureForm::new(spec.id));
                }
                Intent::None
            }
        }
    }
    /// Select the settings-screen row that edits a
    /// verified setting key.
    pub(super) fn select_setting_by_key(&mut self, key: &str) {
        if let Some(index) = settings::KEYS
            .iter()
            .position(|candidate| *candidate == key)
        {
            self.selected = index;
            let groups = categories();
            if let Some((tab, _)) = groups
                .iter()
                .enumerate()
                .find(|(_, group)| group.contains(&index))
            {
                self.tab = tab;
            }
        }
    }
    /// Submit a typed feature operation through the same
    /// guards the settings screen uses: capability, model
    /// confirmation, connection, busy and write lockout.
    pub(in crate::tui) fn submit_feature(&mut self, operation: Operation) -> Intent {
        let t = self.t();
        if let Err(error) = operation.check(self.operation_capabilities()) {
            self.status = error.to_string();
            return Intent::None;
        }
        if !self.control_state.is_connected() || !self.model_confirmed {
            self.status = t.status_must_connect.into();
            return Intent::None;
        }
        if self.busy.is_some() || !self.worker_alive {
            return Intent::None;
        }
        if operation.is_write() && self.writes_blocked {
            self.status = t.status_blocked_note.into();
            return Intent::None;
        }
        Intent::Request(Action::Feature(operation))
    }
    /// Cycle a toggle, choice or peer-index field to its
    /// next value; the draft is set, never left empty.
    pub(super) fn cycle_argument(&mut self, forward: bool) {
        let Some(form) = self.feature_form.as_mut() else {
            return;
        };
        let Some(spec) = lookup(form.id) else {
            return;
        };
        let Some(argument) = spec.arguments.get(form.field) else {
            return;
        };
        let draft = &mut form.drafts[form.field];
        match argument.kind {
            features::ArgumentKind::Toggle => {
                let on = draft.as_deref() == Some("on");
                *draft = Some(if on { "off" } else { "on" }.into());
            }
            features::ArgumentKind::PeerIndex => {
                let next = if draft.as_deref() == Some("1") {
                    "2"
                } else {
                    "1"
                };
                *draft = Some(next.into());
            }
            features::ArgumentKind::Choice(options) => {
                let position = draft
                    .as_deref()
                    .and_then(|value| options.iter().position(|(name, _)| *name == value));
                let next = match position {
                    Some(position) => {
                        let len = options.len();
                        if forward {
                            (position + 1) % len
                        } else {
                            (position + len - 1) % len
                        }
                    }
                    None => 0,
                };
                *draft = Some(options[next].0.into());
            }
            features::ArgumentKind::Side
            | features::ArgumentKind::Seconds
            | features::ArgumentKind::Byte
            | features::ArgumentKind::Mac
            | features::ArgumentKind::AudioOutput
            | features::ArgumentKind::Actions(_) => {}
        }
    }
    /// Type into the selected form field, accepting only
    /// the characters its argument kind allows.
    pub(super) fn feature_form_char(&mut self, c: char) {
        let Some(form) = self.feature_form.as_mut() else {
            return;
        };
        let Some(spec) = lookup(form.id) else {
            return;
        };
        let Some(argument) = spec.arguments.get(form.field) else {
            return;
        };
        let draft = form.drafts[form.field].get_or_insert_with(String::new);
        match argument.kind {
            features::ArgumentKind::Toggle
            | features::ArgumentKind::Side
            | features::ArgumentKind::Choice(_)
            | features::ArgumentKind::PeerIndex => {}
            features::ArgumentKind::Seconds => {
                if c.is_ascii_digit() && draft.chars().count() < 2 {
                    draft.push(c);
                }
            }
            features::ArgumentKind::Byte => {
                if c.is_ascii_digit() && draft.chars().count() < 3 {
                    draft.push(c);
                }
            }
            features::ArgumentKind::Mac => {
                if (c.is_ascii_hexdigit() || c == ':') && draft.chars().count() < 17 {
                    draft.push(c.to_ascii_uppercase());
                }
            }
            features::ArgumentKind::AudioOutput => {
                if !c.is_control() && draft.len() + c.len_utf8() <= 256 {
                    draft.push(c);
                }
            }
            features::ArgumentKind::Actions(_) => {
                if (c.is_ascii_digit() || c == ',') && draft.chars().count() < 30 {
                    draft.push(c);
                }
            }
        }
    }
}
