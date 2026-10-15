//! Main screens; overlays and operation forms are rendered separately.
use super::*;

pub(super) fn render_home(frame: &mut Frame<'_>, regions: &Regions, app: &App) {
    let t = app.t();
    frame.render_widget(
        Paragraph::new(format!(
            "F2 {} · F3 {} · F4 {}",
            t.navigation[1], t.navigation[2], t.navigation[3]
        )),
        regions.tabs,
    );
    let mut lines = Vec::new();
    for spec in crate::features::FEATURES.iter().filter(|spec| {
        spec.kind == FeatureKind::VerifiedSetting
            && spec.category == crate::features::FeatureCategory::Audio
    }) {
        let actual = app
            .info
            .as_ref()
            .and_then(|info| info.value(spec.setting.unwrap_or_default()))
            .unwrap_or_else(|| t.unknown.into());
        lines.push(Line::from(format!("{}: {}", spec.name(t), actual)));
    }
    if regions.list.height > 6 {
        lines.push(Line::from(format!(
            "F6 {}: {}",
            t.navigation[5], t.feature_pending
        )));
    }
    frame.render_widget(
        Paragraph::new(lines)
            .block(panel(t.app_title))
            .wrap(Wrap { trim: true }),
        regions.list,
    );
}

pub(super) fn render_settings(frame: &mut Frame<'_>, _area: Rect, regions: &Regions, app: &App) {
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
    let visible = &super::super::state::categories()[app.tab];
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

pub(super) fn render_observation(
    frame: &mut Frame<'_>,
    regions: &Regions,
    app: &App,
    view: ObservationView,
) {
    let t = app.t();
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
            {
                lines.truncate(5);
                lines.push(Line::from("Connected devices (last query)"));
                match &app.peers {
                    Some(peers) if peers.slots.iter().all(Option::is_none) => {
                        lines.push(Line::from("Headset reports no connected peers"))
                    }
                    Some(peers) => {
                        for peer in peers.slots.iter().flatten() {
                            lines.push(Line::from(format!(
                                "{}: {}  {}",
                                peer.index,
                                clean(&peer.name),
                                peer.address
                            )));
                        }
                    }
                    None => lines.push(Line::from(t.unknown)),
                }
                lines.push(Line::from(
                    "r: query peers | 1/2: review close + dual off | d: dual | Esc: back",
                ));
                lines.push(Line::from(
                    "R: refresh core status / clear uncertain-write lock",
                ));
            }
        }
        ObservationView::Audio => {
            if let Some(observations) = &app.audio {
                if observations.is_empty() {
                    lines.push(Line::from(t.unknown));
                }
                for observation in observations {
                    lines.push(Line::from(format!(
                        "{} / {} / {}",
                        observation.profile.label(),
                        clean(
                            observation
                                .codec
                                .as_ref()
                                .map(|codec| codec.label())
                                .unwrap_or(t.unknown)
                        ),
                        observation.activity.label()
                    )));
                }
            } else {
                lines.push(Line::from(t.unknown));
            }
        }
        ObservationView::Diagnostics => {
            if let Some(report) = &app.diagnostics {
                let same_target = report
                    .address
                    .as_ref()
                    .is_some_and(|address| app.address.eq_ignore_ascii_case(address.as_str()));
                let connected = same_target && app.control_state.is_connected();
                let firmware = if connected {
                    app.firmware.as_deref()
                } else {
                    None
                };
                let data = report.json_with_session(firmware, connected);
                lines = vec![
                    Line::from(clean(&app.address)),
                    Line::from(format!(
                        "{}{}",
                        t.control,
                        if connected {
                            t.connected
                        } else {
                            t.disconnected
                        }
                    )),
                    Line::from(format!("{}{}", t.firmware, firmware.unwrap_or(t.unknown))),
                    Line::from(format!(
                        "{}{}",
                        t.protocol,
                        report
                            .model
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| t.unknown.into())
                    )),
                    Line::from(format!("RFCOMM: {}", report.channel)),
                    Line::from(format!(
                        "{}: {}",
                        t.device_ui[8],
                        if data["write_capabilities"].as_u64().unwrap_or(0) != 0 {
                            crate::features::FEATURES
                                .iter()
                                .filter(|spec| {
                                    matches!(
                                        spec.kind,
                                        FeatureKind::VerifiedSetting | FeatureKind::DeviceControl
                                    ) && spec.verified(crate::models::supported_capabilities())
                                })
                                .count()
                        } else {
                            0
                        }
                    )),
                    Line::from(""),
                ];
                match &report.host {
                    Ok(host) => lines.push(Line::from(format!(
                        "Bluetooth: {} adapters / {} devices",
                        host.adapters.len(),
                        report.devices.len()
                    ))),
                    Err(error) => {
                        lines.push(Line::from(format!("Bluetooth: {}", clean(&error.message))))
                    }
                }
                use crate::diagnostics::SocketCollision;
                let socket = match &report.collision {
                    SocketCollision::NotRequested => t.unknown.to_string(),
                    SocketCollision::NoObservedSocket => {
                        "No socket observed (not proof the channel is free)".into()
                    }
                    SocketCollision::PossibleCollision { matching_sockets } => {
                        format!("{matching_sockets} matching sockets (may include this session)")
                    }
                    SocketCollision::Unavailable(_) => t.device_ui[7].to_string(),
                };
                if !connected {
                    lines.push(Line::from(format!("{}: {}", t.device_ui[6], socket)));
                }
                lines.push(Line::from(""));
                lines.push(Line::from(t.page_labels[1]));
                if let Some(audio) = &report.audio {
                    match audio {
                        Ok(observations) if observations.is_empty() => {
                            lines.push(Line::from(t.unknown))
                        }
                        Ok(observations) => {
                            for observation in observations {
                                lines.push(Line::from(format!(
                                    "{} / {} / {}",
                                    observation.profile.label(),
                                    clean(
                                        observation
                                            .codec
                                            .as_ref()
                                            .map(|codec| codec.label())
                                            .unwrap_or(t.unknown)
                                    ),
                                    observation.activity.label()
                                )));
                                lines.push(Line::from(format!(
                                    "Source: {}",
                                    clean(observation.source)
                                )));
                            }
                        }
                        Err(error) => lines.push(Line::from(clean(&error.message))),
                    }
                } else {
                    lines.push(Line::from(t.unknown));
                }
                lines.push(Line::from(""));
                lines.push(Line::from(t.page_hint));
            } else {
                lines.push(Line::from(t.unknown));
            }
        }
    }
    frame.render_widget(
        Paragraph::new(lines)
            .block(panel(match view {
                ObservationView::Audio => t.page_labels[1],
                ObservationView::Multipoint => t.headset_connections[0],
                ObservationView::Diagnostics => t.page_labels[5],
            }))
            .scroll((app.observation_scroll, 0))
            .wrap(Wrap { trim: true }),
        regions.list,
    );
}

pub(super) fn render_devices(frame: &mut Frame<'_>, regions: &Regions, app: &App) {
    let t = app.t();
    let devices = app.paired.as_deref().unwrap_or_default();
    let rect = regions.list;
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
    let mut state = ListState::default()
        .with_offset(app.paired_scroll(rect.height))
        .with_selected(if devices.is_empty() {
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
