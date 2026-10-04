use super::state::App;
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
    let rows = crate::settings::KEYS
        .iter()
        .enumerate()
        .map(|(index, key)| {
            let actual = app
                .info
                .as_ref()
                .and_then(|info| info.value(key))
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
    let mut list_state = ListState::default().with_selected(Some(app.selected));
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
        sections[1],
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
        sections[2],
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
        sections[3],
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
        overlay_scrolled(
            frame,
            t.shortcuts_title,
            t.shortcuts.iter().map(|s| (*s).into()).collect(),
            app.help_scroll,
        );
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
fn overlay(frame: &mut Frame<'_>, title: &str, lines: Vec<String>) {
    overlay_scrolled(frame, title, lines, 0);
}
fn overlay_scrolled(frame: &mut Frame<'_>, title: &str, lines: Vec<String>, scroll: u16) {
    let width = frame.area().width.saturating_sub(4).min(100);
    let inner_width = usize::from(width.saturating_sub(2)).max(1);
    let height = lines
        .iter()
        .map(|line| line.chars().count().max(1).div_ceil(inner_width))
        .sum::<usize>()
        .saturating_add(2)
        .min(usize::from(frame.area().height.saturating_sub(2))) as u16;
    let rect = centered(frame.area(), width, height);
    frame.render_widget(Clear, rect);
    frame.render_widget(
        Paragraph::new(lines.into_iter().map(Line::from).collect::<Vec<_>>())
            .block(panel(title))
            .scroll((scroll, 0))
            .wrap(Wrap { trim: true }),
        rect,
    );
}
