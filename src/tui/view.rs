use super::state::{choices, feature_tab_labels, App, FeatureForm, ObservationView, Review};
use crate::{
    bluetooth::HostConnectionState,
    features::{lookup, FeatureKind, FeatureSpec, SafetyLevel},
    i18n::{Lang, L},
    models::{Capabilities, Model},
    settings,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

pub(super) const MIN_WIDTH: u16 = 56;
pub(super) const MIN_HEIGHT: u16 = 20;
/// The feature listing and its details panel sit
/// side by side from this width on; below it the
/// details open as a separate view.
pub(super) const WIDE_WIDTH: u16 = 100;
const ACCENT: Color = Color::Cyan;
const MUTED: Color = Color::Gray;

/// Device names and errors are untrusted terminal text. Never render control
/// characters (including escape sequences), bidi overrides, or invisible marks.
pub(super) fn clean(text: &str) -> String {
    text.chars().map(|ch| if ch.is_control() || matches!(ch, '\u{200b}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2060}'..='\u{206f}' | '\u{feff}') { '�' } else { ch }).take(1024).collect()
}
/// The computed screen regions, shared by the renderer
/// and the mouse handler so a click maps to the row
/// that is actually drawn.
pub(super) struct Regions {
    pub header: Rect,
    pub tabs: Rect,
    pub search: Rect,
    pub list: Rect,
    pub status: Rect,
    pub footer: Rect,
}
pub(super) fn regions(area: Rect, features: bool) -> Regions {
    if features {
        let sections = Layout::vertical([
            Constraint::Length(7),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(4),
            Constraint::Length(4),
            Constraint::Length(3),
        ])
        .split(area);
        Regions {
            header: sections[0],
            tabs: sections[1],
            search: sections[2],
            list: sections[3],
            status: sections[4],
            footer: sections[5],
        }
    } else {
        let sections = Layout::vertical([
            Constraint::Length(7),
            Constraint::Length(1),
            Constraint::Min(4),
            Constraint::Length(4),
            Constraint::Length(3),
        ])
        .split(area);
        Regions {
            header: sections[0],
            tabs: sections[1],
            search: Rect::default(),
            list: sections[2],
            status: sections[3],
            footer: sections[4],
        }
    }
}
/// The settings footer plus its "F features" slot,
/// laid out the same way for rendering and clicks.
pub(super) fn footer_rows(t: &L) -> [[(&'static str, &'static str); 6]; 3] {
    let mut rows = t.footer;
    if let Some(slot) = rows[2].iter_mut().find(|(key, _)| key.is_empty()) {
        *slot = ("F", "features");
    }
    rows
}
/// Footer hitboxes: one rectangle per rendered
/// "k v  " slot, in row-major order.
pub(super) fn footer_slots(
    area: Rect,
    rows: &[[(&'static str, &'static str); 6]],
) -> Vec<(&'static str, Rect)> {
    let mut slots = Vec::new();
    for (row, entries) in rows.iter().enumerate() {
        let y = area.y + area.height.saturating_sub(3) + row as u16;
        let mut x = area.x + 1;
        for (key, value) in entries {
            if key.is_empty() {
                continue;
            }
            let width = key.chars().count() as u16 + value.chars().count() as u16 + 3;
            slots.push((*key, Rect::new(x, y, width, 1)));
            x += width;
        }
    }
    slots
}
/// The key event a footer slot dispatches.
pub(super) fn footer_key(key: &str) -> Option<KeyEvent> {
    let code = match key {
        "Enter" => KeyCode::Enter,
        "Esc" => KeyCode::Esc,
        "Tab" => KeyCode::Tab,
        "BackTab" => KeyCode::BackTab,
        "Up" => KeyCode::Up,
        "Down" => KeyCode::Down,
        "Left" => KeyCode::Left,
        "Right" => KeyCode::Right,
        "Backspace" => KeyCode::Backspace,
        "<" => KeyCode::Left,
        ">" => KeyCode::Right,
        "Ctrl+U" => {
            return Some(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL));
        }
        other if other.chars().count() == 1 => KeyCode::Char(other.chars().next().unwrap_or('?')),
        _ => return None,
    };
    Some(KeyEvent::new(code, KeyModifiers::NONE))
}
/// Tab-bar hitboxes: one rectangle per visible
/// " label " tab, starting at the scroll offset.
pub(super) fn tab_rects(area: Rect, labels: &[&'static str], scroll: usize) -> Vec<Rect> {
    let mut rects = Vec::new();
    let mut x = area.x + 1;
    for label in labels.iter().skip(scroll) {
        let width = 2 + label.chars().count() as u16;
        if x.saturating_add(width) > area.x + area.width {
            break;
        }
        rects.push(Rect::new(x, area.y, width, 1));
        x += width;
    }
    rects
}
fn panel(title: &str) -> Block<'_> {
    Block::bordered()
        .style(
            Style::default()
                .fg(Color::Rgb(218, 226, 239))
                .bg(Color::Rgb(15, 20, 30)),
        )
        .title(title)
        .border_style(Style::default().fg(MUTED))
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
pub(super) fn render(frame: &mut Frame<'_>, app: &App) {
    let t = app.t();
    let area = frame.area();
    frame.render_widget(
        Block::new().style(
            Style::default()
                .fg(Color::Rgb(218, 226, 239))
                .bg(Color::Rgb(15, 20, 30)),
        ),
        area,
    );
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        let text = format!(
            "{}\n{}  q: {}\n{}\n{}",
            fill(t.resize_to, &[&MIN_WIDTH, &MIN_HEIGHT]),
            t.resize_quit,
            "quit",
            if app.quit_confirmation {
                t.resize_inflight
            } else {
                t.resize_disabled
            },
            clean(&app.status)
        );
        frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: false }), area);
        return;
    }
    let regions = regions(area, app.features);
    let vendor_codec = app
        .codecs
        .map(|pairs| {
            use crate::protocol::static_analysis::Codec;
            let names = pairs.map(|(_, codec)| match codec {
                Codec::Sbc => "SBC".into(),
                Codec::Aac => "AAC".into(),
                Codec::Ldac => "LDAC".into(),
                Codec::Unknown(v) => format!("unknown({v})"),
            });
            format!("Codecs (last query): {}/{}", names[0], names[1])
        })
        .unwrap_or_else(|| {
            if !Model::StudioPro
                .capabilities()
                .contains(Capabilities::CODEC_STATUS)
            {
                t.codec_unverified.into()
            } else {
                t.codec_line.into()
            }
        });
    let codec_line = match &app.audio {
        Some(observations) if !observations.is_empty() => format!(
            "{} | {}",
            observations
                .iter()
                .map(|status| format!(
                    "{}: {:?}/{:?} {:?} ({}s)",
                    t.host_commands[0],
                    status.profile,
                    status.codec,
                    status.activity,
                    status.observed_at.elapsed().unwrap_or_default().as_secs()
                ))
                .collect::<Vec<_>>()
                .join("; "),
            vendor_codec
        ),
        Some(_) => format!(
            "{}: {} | {}",
            t.host_commands[0], t.unavailable, vendor_codec
        ),
        None => vendor_codec,
    };
    // The OS Bluetooth link and the RFCOMM control session are shown
    // as two separate lines: an open session implies the link, but the
    // link can be up without a session (never the other way around).
    let bluetooth = match app.host_connection {
        HostConnectionState::Connected => t.connected,
        HostConnectionState::Disconnected => t.disconnected,
        HostConnectionState::Unknown => t.unknown,
    };
    let control = match app.control_state {
        crate::bluetooth::ControlSessionState::Connected => t.connected,
        crate::bluetooth::ControlSessionState::Disconnected => t.disconnected,
        crate::bluetooth::ControlSessionState::Connecting => t.status_connecting,
        crate::bluetooth::ControlSessionState::Error => t.unavailable,
    };
    let protocol: String = if app.model_confirmed {
        t.protocol_on.into()
    } else {
        t.protocol_off.into()
    };
    let battery = app
        .info
        .as_ref()
        .and_then(|info| info.battery())
        .map_or(t.unavailable.into(), |v| fill(t.battery_pct, &[&v]));
    let snapshot = if app.stale {
        t.snapshot_stale.into()
    } else if let Some(at) = app.observed_at {
        fill(t.snapshot_ago, &[&at.elapsed().as_secs()])
    } else {
        t.snapshot_never.into()
    };
    let state_style = |connected: bool| {
        Style::default()
            .fg(if connected {
                Color::Green
            } else {
                Color::Yellow
            })
            .add_modifier(Modifier::BOLD)
    };
    let header = vec![
        Line::from(Span::styled(
            format!(
                " {}{}  {}",
                t.bluetooth,
                bluetooth,
                app.host_observed_at
                    .map(|at| fill(t.snapshot_ago, &[&at.elapsed().as_secs()]))
                    .unwrap_or_else(|| t.snapshot_never.into())
            ),
            state_style(app.host_connection.is_connected()),
        )),
        Line::from(Span::styled(
            format!(" {}{}", t.control, control),
            state_style(app.control_state.is_connected()),
        )),
        Line::raw(format!(
            " {}{}",
            t.target,
            if app.address.is_empty() {
                t.target_none.into()
            } else {
                clean(&app.address)
            }
        )),
        Line::raw(format!(" {}{}", t.protocol, protocol)),
        Line::raw(format!(" {}{}  {}", t.battery, battery, codec_line)),
        Line::raw(format!(
            " {}{}  {}{}",
            t.firmware,
            clean(app.firmware.as_deref().unwrap_or(t.unavailable)),
            t.snapshot,
            snapshot
        )),
    ];
    frame.render_widget(
        Paragraph::new(header).block(panel(t.app_title)),
        regions.header,
    );
    if app.features {
        render_features(frame, area, &regions, app);
    } else {
        render_settings(frame, area, &regions, app);
    }
    let status_title = if app.busy.is_some() {
        t.status_working
    } else {
        t.status_title
    };
    frame.render_widget(
        Paragraph::new(clean(&app.status))
            .block(panel(status_title))
            .wrap(Wrap { trim: true }),
        regions.status,
    );
    let rows = if app.features {
        t.features_footer
    } else {
        footer_rows(t)
    };
    let footer: Vec<Line> = rows
        .iter()
        .map(|row| {
            Line::from(
                row.iter()
                    .filter(|(k, _)| !k.is_empty())
                    .map(|(k, v)| Span::styled(format!("{k} {v}  "), Style::default().fg(ACCENT)))
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    frame.render_widget(
        Paragraph::new(footer).wrap(Wrap { trim: true }),
        regions.footer,
    );
    if let Some(index) = app.view_picker {
        let rect = pages_rect(area);
        let mut lines = vec![Line::from(t.page_hint)];
        lines.extend(t.page_labels.iter().enumerate().map(|(i, label)| {
            Line::from(format!("{}{}", if i == index { "> " } else { "  " }, label))
        }));
        frame.render_widget(Clear, rect);
        frame.render_widget(Paragraph::new(lines).block(panel("v")), rect);
    } else if let Some(view) = app.observation_view {
        let mut lines = vec![Line::from(t.page_hint), Line::from(clean(&app.address))];
        match view {
            ObservationView::Multipoint => {
                let dual = app
                    .info
                    .as_ref()
                    .and_then(|info| info.value("dual"))
                    .unwrap_or_else(|| t.unknown.into());
                lines = vec![
                    Line::from(clean(&app.address)),
                    Line::from(t.headset_connections[1]),
                    Line::from(format!("{}: {}", t.labels[4], dual)),
                    Line::from(
                        app.observed_at
                            .map(|at| fill(t.snapshot_ago, &[&at.elapsed().as_secs()]))
                            .unwrap_or_else(|| t.snapshot_never.into()),
                    ),
                    Line::from(if app.stale { t.snapshot_stale } else { "" }),
                    Line::from(format!("{}: {}", t.page_labels[3], t.feature_pending)),
                    Line::from(t.headset_connections[2]),
                    Line::from(t.headset_connections[3]),
                    Line::from(t.headset_connections[4]),
                ];
            }
            ObservationView::Audio => {
                if let Some(observations) = &app.audio {
                    if observations.is_empty() {
                        lines.push(Line::from(t.unknown));
                    }
                    for observation in observations {
                        for line in serde_json::to_string_pretty(&observation.json())
                            .unwrap_or_default()
                            .lines()
                        {
                            lines.push(Line::from(clean(line)));
                        }
                    }
                } else {
                    lines.push(Line::from(t.unknown));
                }
            }
            ObservationView::Diagnostics => {
                if let Some(report) = &app.diagnostics {
                    for line in serde_json::to_string_pretty(&report.json())
                        .unwrap_or_default()
                        .lines()
                    {
                        lines.push(Line::from(clean(line)));
                    }
                } else {
                    lines.push(Line::from(t.unknown));
                }
            }
        }
        overlay_lines(
            frame,
            match view {
                ObservationView::Audio => t.page_labels[1],
                ObservationView::Multipoint => t.headset_connections[0],
                ObservationView::Diagnostics => t.page_labels[5],
            },
            lines,
            app.observation_scroll,
        );
    } else if app.feature_details {
        // Narrow terminals: the details view is a
        // separate screen over the workspace.
        if let Some(spec) = app.selected_feature() {
            let lines = feature_detail_lines(app, spec, usize::from(area.width));
            overlay_lines(frame, spec.name(t), lines, 0);
        }
    } else if let Some(form) = &app.feature_form {
        let spec = lookup(form.id).expect("registry entry");
        let lines = feature_form_lines(app, form, spec);
        overlay_lines(
            frame,
            &fill(t.feature_form_title, &[&spec.name(t)]),
            lines,
            0,
        );
    } else if app.review.is_some() {
        let (title, lines) = review_lines(app);
        overlay_lines(frame, &title, lines, 0);
    } else if app.log_open {
        let rect = centered(
            area,
            area.width.saturating_sub(4).min(110),
            area.height.saturating_sub(4),
        );
        frame.render_widget(Clear, rect);
        let items = app
            .history
            .iter()
            .map(|entry| ListItem::new(clean(entry)))
            .collect::<Vec<_>>();
        let mut state = ListState::default().with_selected(if items.is_empty() {
            None
        } else {
            Some(app.log_index)
        });
        frame.render_stateful_widget(
            List::new(items)
                .block(panel(t.log_title))
                .highlight_symbol("> ")
                .highlight_style(Style::default().fg(ACCENT)),
            rect,
            &mut state,
        );
    } else if app.help {
        // Two-column table: an aligned key column and a
        // description column, grouped by bold headings.
        let mut lines: Vec<Line> = Vec::new();
        for section in t.shortcuts {
            lines.push(Line::styled(
                section.title.to_string(),
                Style::default()
                    .fg(Color::Rgb(218, 226, 239))
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ));
            for (key, description) in section.items {
                lines.push(Line::from(vec![
                    Span::styled(format!(" {key:<12}"), Style::default().fg(ACCENT)),
                    Span::raw(*description),
                ]));
            }
            lines.push(Line::raw(""));
        }
        for (key, label) in [
            ("v", t.page_hint),
            ("A", t.page_labels[1]),
            ("D", t.page_labels[5]),
            ("P", t.headset_connections[0]),
        ] {
            lines.push(Line::from(format!(" {key:<12}{label}")));
        }
        overlay_lines(frame, t.shortcuts_title, lines, app.help_scroll);
    } else if let Some(edit) = &app.address_edit {
        let mut lines = vec![clean(edit)];
        lines.extend(t.edit_lines.iter().map(|s| (*s).into()));
        overlay(frame, t.edit_title, lines);
    } else if app.model_confirmation {
        overlay(
            frame,
            t.model_title,
            t.model_lines.iter().map(|s| (*s).into()).collect(),
        );
    } else if let Some(index) = app.picker {
        // Value list: pick directly instead of stepping
        // through values with Left/Right. The current
        // device value is marked with *.
        let values = choices(app.selected);
        let current = app
            .info
            .as_ref()
            .and_then(|info| info.value(settings::KEYS[app.selected]))
            .unwrap_or_default();
        let mut lines = vec![t.picker_hint.into()];
        lines.extend(values.iter().enumerate().map(|(position, (value, _))| {
            let cursor = if position == index { "> " } else { "  " };
            let mark = if *value == current { "*" } else { " " };
            format!("{cursor}{value} {mark}")
        }));
        let rect = picker_rect(area, values.len());
        frame.render_widget(Clear, rect);
        frame.render_widget(
            Paragraph::new(lines.into_iter().map(Line::from).collect::<Vec<_>>())
                .block(panel(t.picker_title))
                .wrap(Wrap { trim: false }),
            rect,
        );
    } else if app.lang_picker {
        // Language list: pick one directly instead of
        // cycling; the active language is marked *.
        let mut lines = vec![Line::from(t.options_pick_hint)];
        lines.extend(Lang::ALL.iter().enumerate().map(|(position, lang)| {
            let cursor = if position == app.lang_index {
                "> "
            } else {
                "  "
            };
            let mark = if *lang == app.lang { "*" } else { " " };
            Line::raw(format!("{cursor}{} {mark}", lang.name()))
        }));
        let rect = options_rect(area, 1 + Lang::ALL.len());
        frame.render_widget(Clear, rect);
        frame.render_widget(
            Paragraph::new(lines).block(panel(t.options_pick_title)),
            rect,
        );
    } else if app.options {
        // App settings: language, channel, timeout and
        // target. Channel and timeout apply to the next
        // connection; every change is remembered.
        let rows = [
            (t.options_language, app.lang.name().to_string()),
            (t.options_channel, app.channel.to_string()),
            (t.options_timeout, format!("{} s", app.timeout_seconds)),
            (t.options_target, clean(&app.address)),
        ];
        let mut lines = vec![Line::from(t.options_hint)];
        lines.extend(rows.iter().enumerate().map(|(position, (label, value))| {
            let cursor = if position == app.options_index {
                "> "
            } else {
                "  "
            };
            Line::from(vec![
                Span::raw(cursor),
                Span::styled(format!(" {label:<16}"), Style::default().fg(MUTED)),
                Span::raw(value),
            ])
        }));
        let rect = options_rect(area, 1 + rows.len());
        frame.render_widget(Clear, rect);
        frame.render_widget(Paragraph::new(lines).block(panel(t.options_title)), rect);
    } else if let Some(devices) = &app.paired {
        let rect = centered(
            area,
            area.width.saturating_sub(4).min(100),
            area.height.saturating_sub(4),
        );
        frame.render_widget(Clear, rect);
        let items = if devices.is_empty() {
            vec![ListItem::new(t.paired_empty)]
        } else {
            devices
                .iter()
                .map(|device| {
                    // Every row states its link state explicitly; an
                    // unknown state is shown, never hidden.
                    let state = match device.connected {
                        HostConnectionState::Connected => t.connected,
                        HostConnectionState::Disconnected => t.disconnected,
                        HostConnectionState::Unknown => t.unknown,
                    };
                    let model = match device.model {
                        Some(model) => format!("  model={}", model.as_str()),
                        None => String::new(),
                    };
                    ListItem::new(format!(
                        "{}  {}  [{}]{}",
                        clean(device.address.as_str()),
                        clean(&device.name),
                        state,
                        model
                    ))
                })
                .collect()
        };
        let mut state = ListState::default().with_selected(if devices.is_empty() {
            None
        } else {
            Some(app.paired_index)
        });
        frame.render_stateful_widget(
            List::new(items)
                .block(panel(t.paired_title).title_bottom(format!(
                    "p: {}  h: {}  H: {}",
                    t.status_refreshing, t.host_commands[4], t.host_commands[5]
                )))
                .highlight_symbol("> ")
                .highlight_style(Style::default().fg(ACCENT)),
            rect,
            &mut state,
        );
    }
    // Quit warning must be topmost even if another modal is open.
    if app.quit_confirmation {
        overlay(
            frame,
            t.quit_title,
            t.quit_lines.iter().map(|s| (*s).into()).collect(),
        );
    }
}
fn render_settings(frame: &mut Frame<'_>, _area: Rect, regions: &Regions, app: &App) {
    let t = app.t();
    // Category tab bar: the active category is inverted so
    // the settings below it are easy to attribute.
    let tabs: Vec<Span> = t
        .categories
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let label = format!(" {name} ");
            if index == app.tab {
                Span::styled(
                    label,
                    Style::default()
                        .fg(Color::Rgb(15, 20, 30))
                        .bg(ACCENT)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(label, Style::default().fg(MUTED))
            }
        })
        .collect();
    frame.render_widget(Paragraph::new(Line::from(tabs)), regions.tabs);
    let visible = &super::state::categories()[app.tab];
    let rows = visible
        .iter()
        .map(|&index| {
            let actual = app
                .info
                .as_ref()
                .and_then(|info| info.value(settings::KEYS[index]))
                .unwrap_or_else(|| t.unavailable.into());
            let proposal = app.proposals[index]
                .as_ref()
                .map(|value| format!("  {}", fill(t.proposed, &[value])))
                .unwrap_or_default();
            ListItem::new(Line::from(vec![
                Span::raw(format!("{:<19} {:<10}", t.labels[index], actual)),
                Span::styled(proposal, Style::default().fg(Color::Yellow)),
            ]))
        })
        .collect::<Vec<_>>();
    let highlight = visible.iter().position(|&index| index == app.selected);
    let mut list_state = ListState::default().with_selected(highlight);
    let title = if app.writes_blocked {
        t.settings_blocked
    } else if app.stale {
        t.settings_stale
    } else {
        t.settings_title
    };
    frame.render_stateful_widget(
        List::new(rows)
            .block(panel(title))
            .highlight_symbol("> ")
            .highlight_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        regions.list,
        &mut list_state,
    );
}
fn render_features(frame: &mut Frame<'_>, area: Rect, regions: &Regions, app: &App) {
    let t = app.t();
    // Category tabs, scrolled so the active tab stays
    // visible however long the localized names are.
    let labels = feature_tab_labels(t);
    let rects = tab_rects(regions.tabs, &labels, app.feature_tab_scroll);
    let spans = labels
        .iter()
        .enumerate()
        .skip(app.feature_tab_scroll)
        .take(rects.len())
        .map(|(index, label)| {
            let style = if index == app.feature_tab {
                Style::default()
                    .fg(Color::Rgb(15, 20, 30))
                    .bg(ACCENT)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(MUTED)
            };
            Span::styled(format!(" {label} "), style)
        })
        .collect::<Vec<_>>();
    frame.render_widget(Paragraph::new(Line::from(spans)), regions.tabs);
    // Search row: typing filters the registry listing.
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw(t.features_search),
            Span::styled(clean(&app.feature_search), Style::default().fg(ACCENT)),
        ])),
        regions.search,
    );
    let list = app.feature_list();
    let caps = Model::StudioPro.capabilities();
    let mut lines = Vec::new();
    let label_width = list
        .iter()
        .map(|spec| Line::from(spec.name(t)).width())
        .max()
        .unwrap_or(0)
        .min(48);
    for (position, spec) in list.iter().enumerate() {
        let selected = position == app.feature_index;
        let style = if selected {
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(218, 226, 239))
        };
        let marker = if selected { "> " } else { "  " };
        let state = if !spec.verified(caps) {
            t.feature_pending.to_string()
        } else {
            match spec.kind {
                FeatureKind::VerifiedSetting => app
                    .info
                    .as_ref()
                    .and_then(|info| info.value(spec.setting.unwrap_or_default()))
                    .unwrap_or_else(|| t.unavailable.into()),
                FeatureKind::DeviceQuery => t.feature_kind_query.into(),
                FeatureKind::DeviceControl => t.feature_kind_control.into(),
                FeatureKind::OfflineAnalysis => t.feature_kind_offline.into(),
            }
        };
        lines.push(Line::from(vec![
            Span::raw(marker),
            Span::styled(
                format!(
                    "{}{}  ",
                    spec.name(t),
                    " ".repeat(label_width.saturating_sub(Line::from(spec.name(t)).width()))
                ),
                style,
            ),
            Span::styled(
                state,
                if selected {
                    style
                } else {
                    Style::default().fg(MUTED)
                },
            ),
        ]));
    }
    if area.width >= WIDE_WIDTH {
        // Wide terminals: the listing and the details
        // sit side by side.
        let halves = Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(regions.list);
        frame.render_widget(
            Paragraph::new(lines)
                .block(panel(t.features_title))
                .scroll((app.feature_list_scroll as u16, 0)),
            halves[0],
        );
        if let Some(spec) = list.get(app.feature_index) {
            let details = feature_detail_lines(app, spec, usize::from(halves[1].width));
            frame.render_widget(
                Paragraph::new(details)
                    .block(panel(spec.name(t)))
                    .wrap(Wrap { trim: true }),
                halves[1],
            );
        } else {
            frame.render_widget(
                Paragraph::new(Vec::<Line>::new()).block(panel(t.features_title)),
                halves[1],
            );
        }
    } else {
        // Narrow terminals: the listing fills the panel
        // and Enter opens the details as a view.
        frame.render_widget(
            Paragraph::new(lines)
                .block(panel(t.features_title))
                .scroll((app.feature_list_scroll as u16, 0)),
            regions.list,
        );
    }
}
/// The details content for one feature: its category,
/// kind, availability, description, configuration
/// fields and the reason a pending feature cannot run.
fn feature_detail_lines(app: &App, spec: &FeatureSpec, width: usize) -> Vec<Line<'static>> {
    let t = app.t();
    let caps = Model::StudioPro.capabilities();
    let verified = spec.verified(caps);
    let muted = Style::default().fg(MUTED);
    let mut lines: Vec<Line> = vec![
        Line::styled(
            spec.name(t).to_string(),
            Style::default()
                .fg(Color::Rgb(218, 226, 239))
                .add_modifier(Modifier::BOLD),
        ),
        Line::raw(fill(
            t.feature_category_label,
            &[&t.feature_categories[spec.category.index()]],
        )),
        Line::styled(kind_label(spec.kind, t).to_string(), muted),
    ];
    if verified {
        lines.push(Line::styled(
            t.feature_available.to_string(),
            Style::default().fg(Color::Green),
        ));
    } else {
        lines.push(Line::styled(
            t.feature_pending.to_string(),
            Style::default().fg(Color::Yellow),
        ));
        lines.push(Line::styled(
            t.feature_pending_reason.to_string(),
            Style::default().fg(Color::Yellow),
        ));
        lines.push(Line::styled(
            t.feature_live_disabled.to_string(),
            Style::default().fg(Color::Yellow),
        ));
        lines.push(Line::styled(t.feature_offline_workflow.to_string(), muted));
    }
    for wrapped in wrap_text(spec.description(t), width.saturating_sub(4)) {
        lines.push(Line::raw(wrapped));
    }
    match spec.kind {
        FeatureKind::VerifiedSetting => {
            let value = app
                .info
                .as_ref()
                .and_then(|info| info.value(spec.setting.unwrap_or_default()))
                .unwrap_or_else(|| t.unavailable.into());
            lines.push(Line::styled(fill(t.feature_current, &[&value]), muted));
            lines.push(Line::styled(
                t.feature_edit_in_settings.to_string(),
                Style::default().fg(ACCENT),
            ));
        }
        FeatureKind::DeviceQuery if verified => {
            lines.push(Line::styled(
                t.feature_run_query.to_string(),
                Style::default().fg(ACCENT),
            ));
        }
        FeatureKind::DeviceControl if verified => {
            lines.push(Line::styled(
                t.feature_configure.to_string(),
                Style::default().fg(ACCENT),
            ));
            for argument in spec.arguments {
                lines.push(Line::from(vec![
                    Span::styled(format!("{}: ", (argument.label)(t)), muted),
                    Span::raw(argument.kind.hint(t)),
                ]));
            }
        }
        FeatureKind::OfflineAnalysis => {
            let command = spec.offline_command().unwrap_or_default();
            lines.push(Line::raw(fill(t.feature_offline_hint, &[&command])));
        }
        _ => {}
    }
    lines
}
fn kind_label(kind: FeatureKind, t: &L) -> &'static str {
    match kind {
        FeatureKind::VerifiedSetting => t.feature_kind_setting,
        FeatureKind::DeviceQuery => t.feature_kind_query,
        FeatureKind::DeviceControl => t.feature_kind_control,
        FeatureKind::OfflineAnalysis => t.feature_kind_offline,
    }
}
/// The typed form for one control feature: every
/// argument with its draft value and input hint.
fn feature_form_lines(app: &App, form: &FeatureForm, spec: &FeatureSpec) -> Vec<Line<'static>> {
    let t = app.t();
    let muted = Style::default().fg(MUTED);
    let mut lines: Vec<Line> = Vec::new();
    for (position, (argument, draft)) in spec.arguments.iter().zip(form.drafts.iter()).enumerate() {
        let cursor = if position == form.field { "> " } else { "  " };
        let value = draft.as_deref().unwrap_or(t.feature_field_unset);
        lines.push(Line::from(vec![
            Span::raw(cursor),
            Span::styled(format!("{}: ", (argument.label)(t)), muted),
            Span::raw(clean(value)),
            Span::styled(format!("  ({})", argument.kind.hint(t)), muted),
        ]));
    }
    lines.push(Line::styled(t.feature_form_hint.to_string(), muted));
    lines.push(Line::raw(clean(&app.status)));
    lines
}
/// The write review modal: target device, current and
/// proposed values, and whether readback can verify
/// the outcome. Destructive operations carry their
/// own high-risk warning.
fn review_lines(app: &App) -> (String, Vec<Line<'static>>) {
    let t = app.t();
    let muted = Style::default().fg(MUTED);
    let mut lines: Vec<Line> = Vec::new();
    let title = match app.review.as_ref().expect("review is open") {
        Review::Setting { value } => {
            let index = app.selected;
            let label = t.labels[index];
            let current = app
                .info
                .as_ref()
                .and_then(|info| info.value(settings::KEYS[index]))
                .unwrap_or_else(|| t.unavailable.into());
            let noack = !settings::spec(index)
                .map(|spec| spec.expects_ack)
                .unwrap_or(true);
            let readback = if noack {
                t.feature_readback_noack
            } else {
                t.feature_readback_verified
            };
            lines.push(review_row(
                fill(t.feature_review_target, &[&clean(&app.address)]),
                muted,
            ));
            lines.push(review_row(
                fill(t.feature_review_previous, &[&current]),
                muted,
            ));
            lines.push(review_row(fill(t.feature_review_proposed, &[value]), muted));
            lines.push(review_row(
                fill(t.feature_review_readback, &[&readback]),
                muted,
            ));
            label.to_string()
        }
        Review::Feature { form } => {
            let spec = lookup(form.id).expect("registry entry");
            lines.push(review_row(
                fill(t.feature_review_target, &[&clean(&app.address)]),
                muted,
            ));
            lines.push(review_row(
                fill(t.feature_review_values, &[&spec.summarize(&form.drafts)]),
                muted,
            ));
            lines.push(review_row(
                fill(
                    t.feature_review_readback,
                    &[&t.feature_readback_unconfirmed],
                ),
                muted,
            ));
            if spec.safety == SafetyLevel::Destructive {
                lines.push(Line::styled(
                    t.feature_confirm_reset.to_string(),
                    Style::default().fg(Color::Yellow),
                ));
            }
            fill(t.feature_review_title, &[&spec.name(t)])
        }
    };
    lines.push(Line::styled(
        t.feature_confirm_apply.to_string(),
        Style::default().fg(ACCENT),
    ));
    lines.push(Line::raw(clean(&app.status)));
    (title, lines)
}
fn review_row(line: String, style: Style) -> Line<'static> {
    Line::styled(line, style)
}
fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let limit = width.max(1);
    let mut wrapped = Vec::new();
    for paragraph in text.split('\n') {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > limit {
                wrapped.push(line);
                line = String::new();
            }
            if line.is_empty() {
                line.push_str(word);
            } else {
                line.push(' ');
                line.push_str(word);
            }
        }
        wrapped.push(line);
    }
    wrapped
}
pub(super) fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    )
}
/// Geometry of the value-list modal, shared with the mouse
/// handler so a click maps to the same row that is drawn.
pub(super) fn picker_rect(area: Rect, count: usize) -> Rect {
    let width = area.width.saturating_sub(4).min(40);
    let height = (count as u16 + 4).min(area.height.saturating_sub(2));
    centered(area, width, height)
}
/// Geometry of the app-settings and language modals,
/// shared with the mouse handler like `picker_rect`.
pub(super) fn options_rect(area: Rect, lines: usize) -> Rect {
    let width = area.width.saturating_sub(4).min(56);
    let height = (lines as u16 + 3).min(area.height.saturating_sub(2));
    centered(area, width, height)
}
fn line_width(line: &Line) -> usize {
    line.spans
        .iter()
        .map(|span| span.content.chars().count())
        .sum()
}
fn overlay_lines(frame: &mut Frame<'_>, title: &str, lines: Vec<Line>, scroll: u16) {
    let width = frame.area().width.saturating_sub(4).min(100);
    let inner_width = usize::from(width.saturating_sub(2)).max(1);
    let height = lines
        .iter()
        .map(|line| line_width(line).max(1).div_ceil(inner_width))
        .sum::<usize>()
        .saturating_add(2)
        .min(usize::from(frame.area().height.saturating_sub(2))) as u16;
    let rect = centered(frame.area(), width, height);
    frame.render_widget(Clear, rect);
    frame.render_widget(
        Paragraph::new(lines)
            .block(panel(title))
            .scroll((scroll, 0))
            .wrap(Wrap { trim: true }),
        rect,
    );
}
fn overlay(frame: &mut Frame<'_>, title: &str, lines: Vec<String>) {
    overlay_lines(frame, title, lines.into_iter().map(Line::from).collect(), 0);
}

/// Shared geometry for the six page rows and mouse hit testing.
pub(super) fn pages_rect(area: Rect) -> Rect {
    centered(
        area,
        area.width.saturating_sub(4).min(100),
        9.min(area.height),
    )
}
