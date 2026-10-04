use super::state::{choices, App, CATEGORIES};
use crate::{i18n::Lang, settings};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

pub(super) const MIN_WIDTH: u16 = 56;
pub(super) const MIN_HEIGHT: u16 = 20;
const ACCENT: Color = Color::Cyan;
const MUTED: Color = Color::Gray;

/// Device names and errors are untrusted terminal text. Never render control
/// characters (including escape sequences), bidi overrides, or invisible marks.
pub(super) fn clean(text: &str) -> String {
    text.chars().map(|ch| if ch.is_control() || matches!(ch, '\u{200b}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2060}'..='\u{206f}' | '\u{feff}') { '�' } else { ch }).take(1024).collect()
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
    let sections = Layout::vertical([
        Constraint::Length(7),
        Constraint::Length(1),
        Constraint::Min(4),
        Constraint::Length(4),
        Constraint::Length(3),
    ])
    .split(area);
    let connection = if app.connected {
        t.connected
    } else {
        t.disconnected
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
    let header = vec![
        Line::from(vec![
            Span::styled(
                format!(" {connection} "),
                Style::default()
                    .fg(if app.connected {
                        Color::Green
                    } else {
                        Color::Yellow
                    })
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(
                " {}{}",
                t.target,
                if app.address.is_empty() {
                    t.target_none.into()
                } else {
                    clean(&app.address)
                }
            )),
        ]),
        Line::raw(format!(" {}{}", t.protocol, protocol)),
        Line::raw(format!(" {}{}  {}", t.battery, battery, t.codec_line)),
        Line::raw(format!(
            " {}{}  {}{}",
            t.firmware,
            clean(app.firmware.as_deref().unwrap_or(t.unavailable)),
            t.snapshot,
            snapshot
        )),
        Line::raw(format!(
            " {}",
            fill(t.rfcomm_line, &[&app.channel, &app.timeout_seconds])
        )),
    ];
    frame.render_widget(
        Paragraph::new(header).block(panel(t.app_title)),
        sections[0],
    );
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
    frame.render_widget(Paragraph::new(Line::from(tabs)), sections[1]);
    let visible = CATEGORIES[app.tab];
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
        sections[2],
        &mut list_state,
    );
    let status_title = if app.busy.is_some() {
        t.status_working
    } else {
        t.status_title
    };
    frame.render_widget(
        Paragraph::new(clean(&app.status))
            .block(panel(status_title))
            .wrap(Wrap { trim: true }),
        sections[3],
    );
    let footer: Vec<Line> = t
        .footer
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
        sections[4],
    );
    if app.log_open {
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
        lines.extend(values.iter().enumerate().map(|(position, value)| {
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
                    ListItem::new(format!(
                        "{}  {}",
                        clean(&device.address),
                        clean(&device.name)
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
                .block(panel(t.paired_title))
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
fn centered(area: Rect, width: u16, height: u16) -> Rect {
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
