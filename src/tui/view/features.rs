//! Registry list and feature details, shared by everyday and research browsing.
use super::*;

pub(super) fn render_features(frame: &mut Frame<'_>, area: Rect, regions: &Regions, app: &App) {
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
    let workspace_title = if app.feature_scope == super::super::workspace::FeatureScope::Research {
        t.page_labels[4]
    } else {
        t.features_title
    };
    let list = app.feature_list();
    let caps = app.operation_capabilities();
    let mut lines = Vec::new();
    if list.is_empty() {
        lines.push(Line::raw(t.device_ui[4]));
        lines.push(Line::raw(format!("F6 {}", t.navigation[5])));
    }
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
                .block(panel(workspace_title))
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
                Paragraph::new(Vec::<Line>::new()).block(panel(workspace_title)),
                halves[1],
            );
        }
    } else {
        // Narrow terminals: the listing fills the panel
        // and Enter opens the details as a view.
        frame.render_widget(
            Paragraph::new(lines)
                .block(panel(workspace_title))
                .scroll((app.feature_list_scroll as u16, 0)),
            regions.list,
        );
    }
}
/// The details content for one feature: its category,
/// kind, availability, description, configuration
/// fields and the reason a pending feature cannot run.
pub(super) fn feature_detail_lines(
    app: &App,
    spec: &FeatureSpec,
    width: usize,
) -> Vec<Line<'static>> {
    let t = app.t();
    let caps = app.operation_capabilities();
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
