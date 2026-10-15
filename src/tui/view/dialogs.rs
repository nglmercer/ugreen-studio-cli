//! Modal forms, confirmations and legacy pickers.
use super::*;

pub(super) fn render(frame: &mut Frame<'_>, app: &App) {
    let t = app.t();
    let area = frame.area();
    if let Some(index) = app.view_picker {
        let rect = pages_rect(area);
        let mut lines = vec![Line::from(t.page_hint)];
        lines.extend(t.page_labels.iter().enumerate().map(|(i, label)| {
            Line::from(format!("{}{}", if i == index { "> " } else { "  " }, label))
        }));
        frame.render_widget(Clear, rect);
        frame.render_widget(Paragraph::new(lines).block(panel("v")), rect);
    } else if app.observation_view.is_some() {
        // Observation screens occupy the content area; cached dialogs stay hidden.
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
        // target. Channel and timeout apply to the next connection.
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
