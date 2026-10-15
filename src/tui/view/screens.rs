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
