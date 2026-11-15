//! Page bodies.
use super::*;
use crate::{
    bluetooth::HostConnectionState,
    features::{FeatureKind, FeatureSpec},
    models::EvidenceStatus,
    protocol::static_analysis::Codec,
};
use ratatui::widgets::{List, ListItem, ListState};

/// A scrolling text page; the largest useful offset is stored for the
/// key handler.
fn text_page(frame: &mut Frame<'_>, area: Rect, app: &App, title: &str, lines: Vec<Line>) {
    let inner = usize::from(area.width.saturating_sub(2)).max(1);
    let rows: usize = lines
        .iter()
        .map(|line| line.width().max(1).div_ceil(inner))
        .sum();
    let visible = usize::from(area.height.saturating_sub(2));
    let max = rows.saturating_sub(visible).min(usize::from(u16::MAX)) as u16;
    app.max_scroll.set(max);
    frame.render_widget(
        Paragraph::new(lines)
            .block(panel(title))
            .scroll((app.scroll.min(max), 0))
            .wrap(Wrap { trim: false }),
        area,
    );
}
fn check(done: bool) -> Span<'static> {
    if done {
        Span::styled("[x] ", Style::default().fg(GOOD))
    } else {
        Span::styled("[ ] ", Style::default().fg(WARN))
    }
}
fn codec_name(codec: &Codec) -> String {
    match codec {
        Codec::Sbc => "SBC".into(),
        Codec::Aac => "AAC".into(),
        Codec::Ldac => "LDAC".into(),
        Codec::Unknown(v) => format!("0x{v:02X}"),
    }
}
fn codecs_text(app: &App) -> Option<String> {
    app.codecs
        .map(|pairs| pairs.map(|(_, c)| codec_name(&c)).join(" / "))
}
fn peers_summary(app: &App) -> Option<String> {
    app.peers.as_ref().map(|peers| {
        let names: Vec<String> = peers
            .slots
            .iter()
            .flatten()
            .map(|peer| clean(&peer.name))
            .collect();
        if names.is_empty() {
            "0".into()
        } else {
            format!("{} ({})", names.len(), names.join(", "))
        }
    })
}
fn audio_line(app: &App, status: &crate::audio::AudioStatus) -> String {
    let t = app.t();
    format!(
        "{} · {} · {}",
        status.profile.label(),
        clean(
            status
                .codec
                .as_ref()
                .map(|c| c.label())
                .unwrap_or(t.unknown)
        ),
        status.activity.label()
    )
}

pub(super) fn home(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let t = app.t();
    let tt = app.tt();
    let has_target = super::super::worker::validate_address(&app.address).is_ok();
    let mut lines = vec![heading(tt.home_steps)];
    lines.push(Line::from(vec![
        check(has_target),
        Span::raw(tt.step_device),
    ]));
    lines.push(Line::from(vec![
        check(app.model_confirmed),
        Span::raw(tt.step_protocol),
    ]));
    lines.push(Line::from(vec![
        check(app.connected()),
        Span::raw(if app.connected() {
            tt.step_ready
        } else {
            tt.step_connect
        }),
    ]));
    lines.push(Line::raw(""));
    lines.push(heading(tt.home_settings));
    let label_width = t.labels.iter().map(|l| text_width(l)).max().unwrap_or(0);
    for (index, label) in t.labels.iter().enumerate() {
        let value = app
            .current_value(index)
            .unwrap_or_else(|| t.unavailable.into());
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {}  ", pad(label, label_width)),
                Style::default().fg(MUTED),
            ),
            Span::raw(value),
        ]));
    }
    lines.push(Line::raw(""));
    lines.push(heading(tt.home_queries));
    let not_queried = || tt.home_not_queried.to_owned();
    lines.push(Line::from(vec![
        Span::styled(
            format!("  {}: ", tt.home_codecs),
            Style::default().fg(MUTED),
        ),
        Span::raw(codecs_text(app).unwrap_or_else(not_queried)),
    ]));
    lines.push(Line::from(vec![
        Span::styled(format!("  {}: ", tt.home_peers), Style::default().fg(MUTED)),
        Span::raw(peers_summary(app).unwrap_or_else(not_queried)),
    ]));
    let audio = match &app.audio {
        Some(list) if !list.is_empty() => list
            .iter()
            .map(|s| audio_line(app, s))
            .collect::<Vec<_>>()
            .join("; "),
        Some(_) => tt.audio_none.into(),
        None => not_queried(),
    };
    lines.push(Line::from(vec![
        Span::styled(format!("  {}: ", tt.home_audio), Style::default().fg(MUTED)),
        Span::raw(audio),
    ]));
    text_page(frame, area, app, Page::Home.label(tt), lines);
}

pub(super) fn settings(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let t = app.t();
    let (tabs, list, details) = settings_areas(area);
    render_tabs(frame, tabs, &t.categories, app.tab);
    let visible = &super::super::app::categories()[app.tab];
    let label_width = visible
        .iter()
        .map(|&i| text_width(t.labels[i]))
        .max()
        .unwrap_or(0);
    let value_width = visible
        .iter()
        .map(|&i| {
            app.current_value(i)
                .map_or(text_width(t.unavailable), |v| text_width(&v))
        })
        .max()
        .unwrap_or(0);
    let rows: Vec<ListItem> = visible
        .iter()
        .map(|&index| {
            let actual = app
                .current_value(index)
                .unwrap_or_else(|| t.unavailable.into());
            let mut spans = vec![
                Span::raw(format!("{}  ", pad(t.labels[index], label_width))),
                Span::styled(pad(&actual, value_width), Style::default().fg(TEXT)),
            ];
            if let Some(value) = &app.proposals[index] {
                spans.push(Span::styled(
                    fill(t.proposed, &[value]),
                    Style::default().fg(WARN),
                ));
            }
            ListItem::new(Line::from(spans))
        })
        .collect();
    let mut state =
        ListState::default().with_selected(visible.iter().position(|&i| i == app.selected));
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
        list,
        &mut state,
    );
    if let Some(details) = details {
        frame.render_widget(
            Paragraph::new(setting_detail_lines(app, app.selected))
                .block(panel(t.labels[app.selected]))
                .wrap(Wrap { trim: true }),
            details,
        );
    }
}

/// Description, allowed values (current one marked) and write state.
fn setting_detail_lines(app: &App, index: usize) -> Vec<Line<'static>> {
    let t = app.t();
    let key = crate::settings::KEYS[index];
    let mut lines = Vec::new();
    if let Some(spec) = crate::features::by_setting(key) {
        lines.push(Line::raw(spec.description(t).to_owned()));
        lines.push(Line::raw(""));
    }
    let current = app.current_value(index);
    let proposed = app.proposals[index].as_deref();
    let mut values = vec![Span::styled(format!("{key}: "), Style::default().fg(MUTED))];
    for (position, (value, _)) in super::super::app::choices(index).iter().enumerate() {
        if position > 0 {
            values.push(Span::styled(" | ", Style::default().fg(MUTED)));
        }
        let style = if Some(*value) == proposed {
            Style::default().fg(WARN).add_modifier(Modifier::BOLD)
        } else if current.as_deref() == Some(*value) {
            Style::default().fg(GOOD).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(TEXT)
        };
        values.push(Span::styled(value.to_string(), style));
    }
    lines.push(Line::from(values));
    lines.push(muted(fill(
        t.feature_current,
        &[&current.unwrap_or_else(|| t.unavailable.into())],
    )));
    if let Some(value) = proposed {
        lines.push(Line::styled(
            fill(t.feature_review_proposed, &[&value]),
            Style::default().fg(WARN),
        ));
    }
    lines.push(Line::raw(""));
    lines.push(if app.can_write() {
        Line::styled(t.status_proposed, Style::default().fg(ACCENT))
    } else if app.connected() {
        Line::styled(t.status_disabled, Style::default().fg(WARN))
    } else {
        Line::styled(app.tt().need_connection, Style::default().fg(WARN))
    });
    lines
}

fn render_tabs(frame: &mut Frame<'_>, area: Rect, labels: &[&str], active: usize) {
    let mut spans = Vec::new();
    let mut x = area.x;
    for (index, rect) in tab_rects(area, labels, active) {
        spans.push(Span::raw(" ".repeat(usize::from(rect.x - x))));
        let style = if index == active {
            Style::default()
                .fg(BACKGROUND)
                .bg(ACCENT)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(MUTED)
        };
        spans.push(Span::styled(format!(" {} ", labels[index]), style));
        x = rect.right();
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

pub(super) fn badge(app: &App, spec: &FeatureSpec) -> (&'static str, Color) {
    let tt = app.tt();
    if spec.kind == FeatureKind::OfflineAnalysis {
        (tt.badge_offline, MUTED)
    } else if !app.enabled(spec) {
        (tt.badge_blocked, WARN)
    } else if spec.historical_evidence() == EvidenceStatus::Verified {
        (tt.badge_verified, GOOD)
    } else {
        (tt.badge_static, ACCENT)
    }
}

pub(super) fn browser(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let t = app.t();
    let tt = app.tt();
    let regions = browser_areas(area, area.width >= WIDE_WIDTH);
    let browser = app.browser();
    render_tabs(frame, regions.tabs, &app.browser_tab_labels(), browser.tab);
    let mut search = vec![
        Span::styled(tt.search_prompt, Style::default().fg(MUTED)),
        Span::styled(clean(&browser.search), Style::default().fg(ACCENT)),
    ];
    if browser.searching {
        search.push(Span::styled("_", Style::default().fg(ACCENT)));
        search.push(Span::styled(tt.search_typing, Style::default().fg(MUTED)));
    }
    frame.render_widget(Paragraph::new(Line::from(search)), regions.search);
    let list = app.feature_list();
    let title = if app.page == Page::Research {
        Page::Research.label(tt)
    } else {
        Page::Features.label(tt)
    };
    let name_width = list
        .iter()
        .map(|spec| text_width(spec.name(t)))
        .max()
        .unwrap_or(0)
        .min(32);
    let items: Vec<ListItem> = if list.is_empty() {
        vec![ListItem::new(muted(tt.features_empty))]
    } else {
        list.iter()
            .map(|spec| {
                let (label, color) = badge(app, spec);
                let state = match spec.kind {
                    FeatureKind::VerifiedSetting => spec
                        .setting
                        .and_then(|key| app.info.as_ref()?.value(key))
                        .unwrap_or_else(|| label.into()),
                    _ => label.into(),
                };
                ListItem::new(Line::from(vec![
                    Span::raw(format!("{}  ", pad(spec.name(t), name_width))),
                    Span::styled(state, Style::default().fg(color)),
                ]))
            })
            .collect()
    };
    let offset = list_offset(app.browser_offset().get(), browser.index, regions.list);
    app.browser_offset().set(offset);
    let mut state = ListState::default()
        .with_offset(offset)
        .with_selected((!list.is_empty()).then_some(browser.index));
    frame.render_stateful_widget(
        List::new(items)
            .block(panel(title))
            .highlight_symbol("> ")
            .highlight_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        regions.list,
        &mut state,
    );
    if let Some(details) = regions.details {
        match list.get(browser.index) {
            Some(spec) => frame.render_widget(
                Paragraph::new(detail_lines(app, spec))
                    .block(panel(spec.name(t)))
                    .wrap(Wrap { trim: true }),
                details,
            ),
            None => {
                let intro = if app.page == Page::Research {
                    tt.research_intro
                } else {
                    ""
                };
                frame.render_widget(
                    Paragraph::new(intro)
                        .block(panel(title))
                        .wrap(Wrap { trim: true }),
                    details,
                )
            }
        }
    }
}

/// Category, kind, provenance, description, inputs and the last result.
pub(super) fn detail_lines(app: &App, spec: &FeatureSpec) -> Vec<Line<'static>> {
    let t = app.t();
    let tt = app.tt();
    let (badge, color) = badge(app, spec);
    let kind = match spec.kind {
        FeatureKind::VerifiedSetting => t.feature_kind_setting,
        FeatureKind::DeviceQuery => t.feature_kind_query,
        FeatureKind::DeviceControl => t.feature_kind_control,
        FeatureKind::OfflineAnalysis => t.feature_kind_offline,
    };
    let mut lines = vec![
        Line::from(vec![
            Span::styled(kind.to_owned(), Style::default().fg(MUTED)),
            Span::raw(" · "),
            Span::raw(t.feature_categories[spec.category.index()]),
            Span::raw(" · "),
            Span::styled(badge.to_owned(), Style::default().fg(color)),
        ]),
        Line::raw(""),
        Line::raw(spec.description(t).to_owned()),
        Line::raw(""),
    ];
    let provenance = if spec.kind == FeatureKind::OfflineAnalysis {
        t.feature_offline_workflow
    } else if !app.enabled(spec) {
        tt.provenance_blocked
    } else if spec.historical_evidence() == EvidenceStatus::Verified {
        tt.provenance_verified
    } else {
        tt.provenance_static
    };
    lines.push(Line::styled(
        provenance.to_owned(),
        Style::default().fg(color),
    ));
    match spec.kind {
        FeatureKind::VerifiedSetting => {
            let value = spec
                .setting
                .and_then(|key| app.info.as_ref()?.value(key))
                .unwrap_or_else(|| t.unavailable.into());
            lines.push(muted(fill(t.feature_current, &[&value])));
            lines.push(Line::styled(tt.go_settings, Style::default().fg(ACCENT)));
        }
        FeatureKind::OfflineAnalysis => {
            lines.push(Line::styled(
                fill(
                    t.feature_offline_hint,
                    &[&spec.offline_command().unwrap_or("")],
                ),
                Style::default().fg(ACCENT),
            ));
        }
        FeatureKind::DeviceQuery | FeatureKind::DeviceControl if app.enabled(spec) => {
            for argument in spec.arguments {
                lines.push(muted(format!(
                    "{}: {}",
                    (argument.label)(t),
                    argument.kind.hint(t)
                )));
            }
            if !app.connected() {
                lines.push(Line::styled(tt.need_connection, Style::default().fg(WARN)));
            } else {
                lines.push(Line::styled(
                    if spec.kind == FeatureKind::DeviceQuery {
                        t.feature_run_query
                    } else {
                        t.feature_configure
                    },
                    Style::default().fg(ACCENT),
                ));
            }
        }
        _ => {}
    }
    if let Some((_, text)) = app.last_result.as_ref().filter(|(id, _)| *id == spec.id) {
        lines.push(Line::raw(""));
        lines.push(Line::raw(fill(tt.last_result, &[&clean(text)])));
    }
    lines
}

pub(super) fn devices(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let t = app.t();
    let tt = app.tt();
    let items: Vec<ListItem> = match &app.devices {
        None => vec![ListItem::new(muted(tt.devices_not_loaded))],
        Some(list) if list.is_empty() => vec![ListItem::new(muted(t.paired_empty))],
        Some(list) => {
            let width = list
                .iter()
                .map(|d| text_width(&clean(&d.name)))
                .max()
                .unwrap_or(0)
                .min(28);
            list.iter()
                .map(|device| {
                    let (state, color) = match device.connected {
                        HostConnectionState::Connected => (t.connected, GOOD),
                        HostConnectionState::Disconnected => (t.disconnected, MUTED),
                        HostConnectionState::Unknown => (t.unknown, WARN),
                    };
                    let mut spans = vec![
                        Span::raw(format!("{}  ", device.address.as_str())),
                        Span::raw(format!("{}  ", pad(&clean(&device.name), width))),
                        Span::styled(format!("[{state}]"), Style::default().fg(color)),
                    ];
                    if let Some(model) = device.model {
                        spans.push(Span::styled(
                            format!("  {}", model.as_str()),
                            Style::default().fg(ACCENT),
                        ));
                    }
                    if device.address.as_str().eq_ignore_ascii_case(&app.address) {
                        spans.push(Span::styled(
                            format!("  <{}>", tt.devices_current),
                            Style::default().fg(ACCENT),
                        ));
                    }
                    ListItem::new(Line::from(spans))
                })
                .collect()
        }
    };
    let count = app.devices.as_ref().map_or(0, Vec::len);
    let offset = list_offset(app.devices_offset.get(), app.device_index, area);
    app.devices_offset.set(offset);
    let mut state = ListState::default()
        .with_offset(offset)
        .with_selected((count > 0).then_some(app.device_index));
    frame.render_stateful_widget(
        List::new(items)
            .block(panel(tt.devices_title))
            .highlight_symbol("> ")
            .highlight_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        area,
        &mut state,
    );
}

pub(super) fn peers(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let t = app.t();
    let tt = app.tt();
    let mut lines = vec![muted(tt.peers_intro), Line::raw("")];
    let dual = app.dual_value().unwrap_or_else(|| t.unknown.into());
    lines.push(Line::raw(fill(tt.peers_dual, &[&dual])));
    lines.push(Line::raw(""));
    match &app.peers {
        None => lines.push(Line::styled(
            if app.connected() {
                tt.peers_not_queried
            } else {
                tt.need_connection
            },
            Style::default().fg(WARN),
        )),
        Some(peers) if peers.slots.iter().all(Option::is_none) => {
            lines.push(Line::raw(tt.peers_none))
        }
        Some(peers) => {
            for (slot, peer) in peers.slots.iter().enumerate() {
                if let Some(peer) = peer {
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("  {}  ", slot + 1),
                            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(format!("{}  ", clean(&peer.name))),
                        Span::styled(peer.address.to_string(), Style::default().fg(MUTED)),
                    ]));
                }
            }
            if !peers.complete {
                lines.push(Line::styled(tt.peers_incomplete, Style::default().fg(WARN)));
            }
            lines.push(Line::raw(""));
            lines.push(muted(tt.peers_close_note));
        }
    }
    lines.push(muted(tt.peers_unsupported));
    text_page(frame, area, app, Page::Peers.label(tt), lines);
}

pub(super) fn audio(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let tt = app.tt();
    let mut lines = vec![muted(tt.audio_intro), Line::raw("")];
    match &app.audio {
        None => lines.push(Line::raw(tt.audio_not_read)),
        Some(list) if list.is_empty() => lines.push(Line::raw(tt.audio_none)),
        Some(list) => {
            for status in list {
                lines.push(Line::raw(audio_line(app, status)));
                lines.push(muted(format!(
                    "  {} · {}",
                    fill(tt.audio_source, &[&clean(status.source)]),
                    fill(
                        tt.audio_age,
                        &[&status.observed_at.elapsed().unwrap_or_default().as_secs()]
                    )
                )));
            }
        }
    }
    lines.push(Line::raw(""));
    lines.push(Line::raw(fill(
        tt.audio_vendor,
        &[&codecs_text(app).unwrap_or_else(|| tt.audio_vendor_none.into())],
    )));
    text_page(frame, area, app, Page::Audio.label(tt), lines);
}

pub(super) fn diagnostics(frame: &mut Frame<'_>, area: Rect, app: &App) {
    use crate::diagnostics::SocketCollision;
    let t = app.t();
    let tt = app.tt();
    let Some(report) = &app.diagnostics else {
        text_page(
            frame,
            area,
            app,
            Page::Diagnostics.label(tt),
            vec![Line::raw(tt.diag_not_read)],
        );
        return;
    };
    let same_target = report
        .address
        .as_ref()
        .is_some_and(|a| app.address.eq_ignore_ascii_case(a.as_str()));
    let connected = same_target && app.connected();
    let firmware = connected.then_some(app.firmware.as_deref()).flatten();
    let data = report.json_with_session(firmware, connected);
    let enabled = if data["write_capabilities"].as_u64().unwrap_or(0) != 0 {
        crate::features::FEATURES
            .iter()
            .filter(|spec| spec.kind != FeatureKind::OfflineAnalysis && app.enabled(spec))
            .count()
    } else {
        0
    };
    let mut lines = vec![
        Line::from(field(t.target, clean(&app.address))),
        Line::from(field(
            t.control,
            if connected {
                t.connected
            } else {
                t.disconnected
            }
            .into(),
        )),
        Line::from(field(t.firmware, firmware.unwrap_or(t.unknown).into())),
        Line::from(field(
            t.protocol,
            report
                .model
                .map(|m| m.to_string())
                .unwrap_or_else(|| t.unknown.into()),
        )),
        Line::from(field("RFCOMM:", report.channel.to_string())),
        Line::from(field(
            &format!("{}:", tt.diag_controls),
            enabled.to_string(),
        )),
        Line::raw(""),
    ];
    lines.push(Line::raw(match &report.host {
        Ok(host) => fill(
            tt.diag_bluetooth,
            &[&host.adapters.len(), &report.devices.len()],
        ),
        Err(error) => format!("Bluetooth: {}", clean(&error.message)),
    }));
    if let Some(error) = &report.device_error {
        lines.push(Line::styled(
            clean(&error.message),
            Style::default().fg(WARN),
        ));
    }
    if !connected {
        let socket = match &report.collision {
            SocketCollision::NotRequested => t.unknown.to_owned(),
            SocketCollision::NoObservedSocket => tt.diag_socket_none.to_owned(),
            SocketCollision::PossibleCollision { matching_sockets } => {
                fill(tt.diag_socket_matches, &[matching_sockets])
            }
            SocketCollision::Unavailable(_) => t.device_ui[7].to_owned(),
        };
        lines.push(Line::from(field(&format!("{}:", t.device_ui[6]), socket)));
    }
    lines.push(Line::raw(""));
    lines.push(heading(tt.diag_audio));
    match &report.audio {
        None => lines.push(Line::raw(t.unknown)),
        Some(Err(error)) => lines.push(Line::raw(clean(&error.message))),
        Some(Ok(list)) if list.is_empty() => lines.push(Line::raw(tt.audio_none)),
        Some(Ok(list)) => {
            for status in list {
                lines.push(Line::raw(audio_line(app, status)));
                lines.push(muted(format!(
                    "  {}",
                    fill(tt.audio_source, &[&clean(status.source)])
                )));
            }
        }
    }
    text_page(frame, area, app, Page::Diagnostics.label(tt), lines);
}
