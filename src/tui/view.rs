use super::state::{App, LABELS};
use crate::settings;
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
pub(super) fn render(frame: &mut Frame<'_>, app: &App) {
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
            "Resize to at least {MIN_WIDTH}x{MIN_HEIGHT}\nEsc: cancel  q: quit\n{}\n{}",
            if app.quit_confirmation {
                "In-flight write may complete. q/Enter: quit; Esc: stay"
            } else {
                "Actions disabled at this size."
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
        "CONNECTED"
    } else {
        "DISCONNECTED"
    };
    let protocol = if app.model_confirmed {
        "Studio Pro HP206 (user-selected)"
    } else {
        "not selected; m to confirm Studio Pro HP206"
    };
    let battery = app
        .info
        .as_ref()
        .and_then(|info| info.battery())
        .map_or("unavailable".into(), |v| format!("{v}%"));
    let snapshot = if app.stale {
        "STALE / not current".into()
    } else if let Some(at) = app.observed_at {
        format!("read {}s ago", at.elapsed().as_secs())
    } else {
        "not read".into()
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
                " Target: {}",
                if app.address.is_empty() {
                    "not selected".into()
                } else {
                    clean(&app.address)
                }
            )),
        ]),
        Line::raw(format!(" Protocol: {protocol}")),
        Line::raw(format!(
            " Battery: {battery}  Codec: unavailable (not exposed)"
        )),
        Line::raw(format!(
            " Firmware: {}  Snapshot: {snapshot}",
            clean(app.firmware.as_deref().unwrap_or("unavailable"))
        )),
        Line::raw(format!(
            " RFCOMM {} | per-operation timeout {}s | hardware unverified",
            app.channel, app.timeout_seconds
        )),
    ];
    frame.render_widget(
        Paragraph::new(header).block(panel(" UGREEN / Studio Pro control ")),
        sections[0],
    );
    let rows = settings::KEYS
        .iter()
        .enumerate()
        .map(|(index, key)| {
            let actual = app
                .info
                .as_ref()
                .and_then(|info| info.value(key))
                .unwrap_or_else(|| "unavailable".into());
            let proposal = app.proposals[index]
                .as_ref()
                .map(|value| format!("  -> {value} (proposed)"))
                .unwrap_or_default();
            ListItem::new(Line::from(vec![
                Span::raw(format!("{:<19} {:<10}", LABELS[index], actual)),
                Span::styled(proposal, Style::default().fg(Color::Yellow)),
            ]))
        })
        .collect::<Vec<_>>();
    let mut list_state = ListState::default().with_selected(Some(app.selected));
    let title = if app.writes_blocked {
        " Settings / WRITES BLOCKED: r to refresh "
    } else if app.stale {
        " Settings / last snapshot is stale "
    } else {
        " Settings / device readback, no automatic writes "
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
        " Working / Esc cancels later stages "
    } else {
        " Status "
    };
    frame.render_widget(
        Paragraph::new(clean(&app.status))
            .block(panel(status_title))
            .wrap(Wrap { trim: true }),
        sections[2],
    );
    frame.render_widget(Paragraph::new("a address  p paired cache  m model  c connect  r refresh  d disconnect\nArrows select/propose  Enter review  Esc cancel  l log  ? help  q quit").style(Style::default().fg(ACCENT)).wrap(Wrap { trim: true }), sections[3]);
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
                .block(panel(" Session log / Up Down Home End / l Esc close "))
                .highlight_symbol("> ")
                .highlight_style(Style::default().fg(ACCENT)),
            rect,
            &mut state,
        );
    } else if app.help {
        overlay_scrolled(
            frame,
            " Help / Up Down scroll / ? Esc close ",
            vec![
                "a          Edit address while disconnected; Enter saves locally".into(),
                "p          Read OS paired-device cache; never scans or pairs".into(),
                "m          Review and select Studio Pro HP206 protocol".into(),
                "c / r / d  Explicit connect / refresh / disconnect".into(),
                "Up/Down    Select setting (list scrolls)".into(),
                "Left/Right Propose value only; no write".into(),
                "Enter, y   Review setting, then confirm one write".into(),
                "Esc        Dismiss / discard proposal / cancel later I/O stages".into(),
                "q / Ctrl-C Quit (confirm if work is in flight)".into(),
                "l          Session log (latest 100 events); Up/Down, Home/End".into(),
                "? / Esc    Close this help".into(),
                "Writes require preflight, acknowledgement and matching readback.".into(),
                "An in-flight write may complete after cancellation or quit.".into(),
                "Uncertain writes stay blocked until an explicit refresh.".into(),
                "Model selection does not verify hardware identity; Max5c unsupported.".into(),
            ],
            app.help_scroll,
        );
    } else if let Some(edit) = &app.address_edit {
        overlay(
            frame,
            " Edit target address ",
            vec![
                clean(edit),
                "Type hexadecimal digits and colons. Backspace deletes; Ctrl-U clears.".into(),
                "Enter saves locally; Esc cancels. No connection is started.".into(),
            ],
        );
    } else if app.model_confirmation {
        overlay(
            frame,
            " Confirm model-specific protocol ",
            vec![
                "Use this only with UGREEN Studio Pro HP206.".into(),
                "Hardware compatibility has not been verified on this build.".into(),
                "HiTune Max5c uses conflicting command IDs and is unsupported.".into(),
                "This selects a protocol; it does not prove device identity.".into(),
                "y: select Studio Pro HP206    n / Esc: cancel".into(),
            ],
        );
    } else if let Some(setting) = &app.confirmation {
        overlay(
            frame,
            " Confirm one setting write ",
            vec![
                format!(
                    "Target: {} (user-selected Studio Pro HP206)",
                    clean(&app.address)
                ),
                format!("Change {} to {}?", setting.key, setting.value),
                "A preflight query, acknowledgement and readback are required.".into(),
                "No automatic retry or rollback. A partial write may take effect.".into(),
                "y: apply once    n / Esc: cancel".into(),
            ],
        );
    } else if let Some(devices) = &app.paired {
        let rect = centered(
            area,
            area.width.saturating_sub(4).min(100),
            area.height.saturating_sub(4),
        );
        frame.render_widget(Clear, rect);
        let items = if devices.is_empty() {
            vec![ListItem::new("No cached paired devices. Esc closes.")]
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
                .block(panel(" Paired cache / Up Down Enter select / Esc close "))
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
            " Quit while work is in flight? ",
            vec![
                "Cancellation requested; later stages will be skipped.".into(),
                "An in-flight setting write may still complete.".into(),
                "Quitting never waits for the worker.".into(),
                "q / Enter: quit now    Esc: stay and wait for the result".into(),
            ],
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
