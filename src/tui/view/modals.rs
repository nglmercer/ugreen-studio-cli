//! Modal dialogs, drawn over the page. The quit warning is always topmost.
use super::super::app::{choices, FeatureForm, Modal, Review, OPTION_ROWS};
use super::*;
use crate::{
    features::{lookup, FeatureSpec, SafetyLevel},
    i18n::Lang,
    settings,
};
use ratatui::widgets::{Clear, List, ListItem, ListState};

pub(super) fn render(frame: &mut Frame<'_>, app: &App) {
    let t = app.t();
    let area = frame.area();
    match &app.modal {
        None => {}
        Some(Modal::Help { scroll }) => help(frame, app, *scroll),
        Some(Modal::Log { index }) => {
            let rect = centered(
                area,
                area.width.saturating_sub(4).min(110),
                area.height.saturating_sub(4),
            );
            let items: Vec<ListItem> = app
                .history
                .iter()
                .map(|entry| ListItem::new(clean(entry)))
                .collect();
            let mut state =
                ListState::default().with_selected((!items.is_empty()).then_some(*index));
            frame.render_widget(Clear, rect);
            frame.render_stateful_widget(
                List::new(items)
                    .block(panel(t.log_title))
                    .highlight_symbol("> ")
                    .highlight_style(Style::default().fg(ACCENT)),
                rect,
                &mut state,
            );
        }
        Some(Modal::Options { index }) => {
            let rows = [
                (t.options_language, app.lang.name().to_owned()),
                (t.options_channel, app.channel.to_string()),
                (t.options_timeout, format!("{} s", app.timeout_seconds)),
                (
                    t.options_target,
                    if app.address.is_empty() {
                        t.target_none.into()
                    } else {
                        clean(&app.address)
                    },
                ),
            ];
            debug_assert_eq!(rows.len(), OPTION_ROWS);
            let width = rows.iter().map(|(l, _)| text_width(l)).max().unwrap_or(0);
            let items = rows
                .iter()
                .map(|(label, value)| {
                    Line::from(vec![
                        Span::styled(
                            format!("{}  ", pad(label, width)),
                            Style::default().fg(MUTED),
                        ),
                        Span::raw(value.clone()),
                    ])
                })
                .collect();
            list_modal(frame, t.options_title, items, *index);
        }
        Some(Modal::Language { index }) => {
            let items = Lang::ALL
                .iter()
                .map(|lang| {
                    let mark = if *lang == app.lang { "  *" } else { "" };
                    Line::raw(format!("{}{mark}", lang.name()))
                })
                .collect();
            list_modal(frame, t.options_pick_title, items, *index);
        }
        Some(Modal::Values { index }) => {
            let current = app.current_value(app.selected).unwrap_or_default();
            let items = choices(app.selected)
                .iter()
                .map(|(value, _)| {
                    let mark = if *value == current { "  *" } else { "" };
                    Line::raw(format!("{value}{mark}"))
                })
                .collect();
            let title = format!(" {} ", t.labels[app.selected]);
            list_modal(frame, &title, items, *index);
        }
        Some(Modal::Address { text }) => {
            let mut lines = vec![Line::styled(
                format!("{}_", clean(text)),
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            )];
            lines.extend(t.edit_lines.iter().map(|s| muted(*s)));
            dialog(frame, t.edit_title, lines);
        }
        Some(Modal::Model) => dialog(
            frame,
            t.model_title,
            t.model_lines.iter().map(|s| Line::raw(*s)).collect(),
        ),
        Some(Modal::Form(form)) => {
            let spec = lookup(form.id).expect("registry entry");
            dialog(
                frame,
                &fill(t.feature_form_title, &[&spec.name(t)]),
                form_lines(app, form, spec),
            );
        }
        Some(Modal::Review(review)) => {
            let (title, lines) = review_lines(app, review);
            dialog(frame, &title, lines);
        }
        Some(Modal::Details) => {
            if let Some(spec) = app.selected_feature() {
                dialog(frame, spec.name(t), super::pages::detail_lines(app, spec));
            }
        }
    }
    if app.quit_confirmation {
        dialog(
            frame,
            t.quit_title,
            t.quit_lines.iter().map(|s| Line::raw(*s)).collect(),
        );
    }
}

fn list_modal(frame: &mut Frame<'_>, title: &str, items: Vec<Line<'static>>, index: usize) {
    let rect = list_modal_rect(frame.area(), items.len());
    let mut state = ListState::default().with_selected(Some(index));
    frame.render_widget(Clear, rect);
    frame.render_stateful_widget(
        List::new(items.into_iter().map(ListItem::new).collect::<Vec<_>>())
            .block(panel(title))
            .highlight_symbol("> ")
            .highlight_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        rect,
        &mut state,
    );
}

/// A centered dialog sized to its wrapped content.
fn dialog(frame: &mut Frame<'_>, title: &str, lines: Vec<Line>) {
    let area = frame.area();
    let width = area.width.saturating_sub(4).min(90);
    let inner = usize::from(width.saturating_sub(2)).max(1);
    let height = lines
        .iter()
        .map(|line| line.width().max(1).div_ceil(inner))
        .sum::<usize>()
        .saturating_add(2)
        .min(usize::from(area.height.saturating_sub(2))) as u16;
    let rect = centered(area, width, height);
    frame.render_widget(Clear, rect);
    frame.render_widget(
        Paragraph::new(lines)
            .block(panel(title))
            .wrap(Wrap { trim: false }),
        rect,
    );
}

fn help(frame: &mut Frame<'_>, app: &App, scroll: u16) {
    let t = app.t();
    let tt = app.tt();
    let area = frame.area();
    let mut lines: Vec<Line> = Vec::new();
    let section = |lines: &mut Vec<Line>, title: &str, hints: Vec<super::keymap::Hint>| {
        lines.push(Line::styled(
            title.to_owned(),
            Style::default()
                .fg(TEXT)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ));
        for hint in hints {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {}", pad(hint.key, 12)),
                    Style::default().fg(ACCENT),
                ),
                Span::raw(hint.label),
            ]));
        }
        lines.push(Line::raw(""));
    };
    section(&mut lines, tt.help_global, super::keymap::global(tt));
    for page in Page::ALL {
        let hints = super::keymap::page(app, page);
        if !hints.is_empty() {
            section(
                &mut lines,
                &format!("F{} {}", page.index() + 1, page.label(tt)),
                hints,
            );
        }
    }
    // Mouse and safety notes stay in the translated catalog.
    for item in t.shortcuts.iter().skip(3) {
        lines.push(Line::styled(
            item.title.to_owned(),
            Style::default()
                .fg(TEXT)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ));
        for (key, description) in item.items {
            lines.push(Line::from(vec![
                Span::styled(format!("  {}", pad(key, 12)), Style::default().fg(ACCENT)),
                Span::raw(*description),
            ]));
        }
        lines.push(Line::raw(""));
    }
    let rect = centered(
        area,
        area.width.saturating_sub(4).min(100),
        area.height.saturating_sub(2),
    );
    let inner = usize::from(rect.width.saturating_sub(2)).max(1);
    let rows: usize = lines
        .iter()
        .map(|line| line.width().max(1).div_ceil(inner))
        .sum();
    let max = rows.saturating_sub(usize::from(rect.height.saturating_sub(2))) as u16;
    app.max_scroll.set(max);
    frame.render_widget(Clear, rect);
    frame.render_widget(
        Paragraph::new(lines)
            .block(panel(t.shortcuts_title))
            .scroll((scroll.min(max), 0))
            .wrap(Wrap { trim: false }),
        rect,
    );
}

fn form_lines(app: &App, form: &FeatureForm, spec: &FeatureSpec) -> Vec<Line<'static>> {
    let t = app.t();
    let mut lines = Vec::new();
    for (position, (argument, draft)) in spec.arguments.iter().zip(&form.drafts).enumerate() {
        let selected = position == form.field;
        let value = draft.as_deref().map(clean);
        lines.push(Line::from(vec![
            Span::raw(if selected { "> " } else { "  " }),
            Span::styled(
                format!("{}: ", (argument.label)(t)),
                Style::default().fg(MUTED),
            ),
            match value {
                Some(value) => Span::styled(
                    value,
                    Style::default().fg(if selected { ACCENT } else { TEXT }),
                ),
                None => Span::styled(t.feature_field_unset, Style::default().fg(WARN)),
            },
            Span::styled(
                format!("  ({})", argument.kind.hint(t)),
                Style::default().fg(MUTED),
            ),
        ]));
    }
    lines.push(Line::raw(""));
    lines.push(muted(t.feature_form_hint));
    lines.push(Line::raw(clean(&app.status)));
    lines
}

fn review_lines(app: &App, review: &Review) -> (String, Vec<Line<'static>>) {
    let t = app.t();
    let target = fill(t.feature_review_target, &[&clean(&app.address)]);
    let mut lines = vec![muted(target)];
    let title = match review {
        Review::Setting { index, value } => {
            let current = app
                .current_value(*index)
                .unwrap_or_else(|| t.unavailable.into());
            let noack = !settings::spec(*index).is_none_or(|spec| spec.expects_ack);
            lines.push(muted(fill(t.feature_review_previous, &[&current])));
            lines.push(Line::styled(
                fill(t.feature_review_proposed, &[value]),
                Style::default().fg(ACCENT),
            ));
            lines.push(muted(fill(
                t.feature_review_readback,
                &[&if noack {
                    t.feature_readback_noack
                } else {
                    t.feature_readback_verified
                }],
            )));
            fill(t.feature_review_title, &[&t.labels[*index]])
        }
        Review::Feature { form, .. } => {
            let spec = lookup(form.id).expect("registry entry");
            let summary = spec.summarize(&form.drafts);
            if !summary.is_empty() {
                lines.push(Line::styled(
                    fill(t.feature_review_values, &[&clean(&summary)]),
                    Style::default().fg(ACCENT),
                ));
            }
            lines.push(Line::styled(
                super::pages::badge(app, spec).0.to_owned(),
                Style::default().fg(super::pages::badge(app, spec).1),
            ));
            lines.push(muted(fill(
                t.feature_review_readback,
                &[&t.feature_readback_unconfirmed],
            )));
            if spec.safety == SafetyLevel::Destructive {
                lines.push(Line::styled(
                    t.feature_confirm_reset,
                    Style::default().fg(WARN).add_modifier(Modifier::BOLD),
                ));
            }
            fill(t.feature_review_title, &[&spec.name(t)])
        }
    };
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        t.feature_confirm_apply,
        Style::default().fg(ACCENT),
    ));
    (title, lines)
}
