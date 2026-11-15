//! Rendering. Every rectangle a click can hit comes from a function in
//! this module that the mouse handler calls too, so the two never drift.
mod modals;
mod pages;

use super::{
    app::{App, Page},
    keymap::{self, Hint},
};
use crate::bluetooth::{ControlSessionState, HostConnectionState};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
    Frame,
};

pub(super) const MIN_WIDTH: u16 = 56;
pub(super) const MIN_HEIGHT: u16 = 20;
/// From this width the feature list and its details sit side by side.
pub(super) const WIDE_WIDTH: u16 = 100;

pub(super) const ACCENT: Color = Color::Cyan;
pub(super) const MUTED: Color = Color::Gray;
pub(super) const TEXT: Color = Color::Rgb(218, 226, 239);
pub(super) const BACKGROUND: Color = Color::Rgb(15, 20, 30);
pub(super) const GOOD: Color = Color::Green;
pub(super) const WARN: Color = Color::Yellow;
pub(super) const BAND: Color = Color::Rgb(26, 33, 46);

/// Device names and errors are untrusted terminal text. Never render
/// control characters (including escapes), bidi overrides or invisible marks.
pub(super) fn clean(text: &str) -> String {
    text.chars()
        .map(|ch| {
            if ch.is_control()
                || matches!(ch, '\u{200b}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2060}'..='\u{206f}' | '\u{feff}')
            {
                '�'
            } else {
                ch
            }
        })
        .take(1024)
        .collect()
}
pub(super) fn text_width(text: &str) -> usize {
    Line::from(text).width()
}
/// Pad to a display width (CJK-aware), never truncating.
pub(super) fn pad(text: &str, width: usize) -> String {
    let mut out = text.to_owned();
    out.push_str(&" ".repeat(width.saturating_sub(text_width(text))));
    out
}
pub(super) fn panel(title: &str) -> Block<'_> {
    Block::bordered()
        .style(Style::default().fg(TEXT).bg(BACKGROUND))
        .title(title)
        .border_style(Style::default().fg(MUTED))
}
pub(super) fn heading(text: impl Into<String>) -> Line<'static> {
    Line::styled(
        text.into(),
        Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
    )
}
pub(super) fn muted(text: impl Into<String>) -> Line<'static> {
    Line::styled(text.into(), Style::default().fg(MUTED))
}

/// The frame's fixed regions plus the footer's clickable hints.
pub(super) struct Areas {
    pub header: Rect,
    pub nav: Rect,
    pub body: Rect,
    pub status: Rect,
    pub slots: Vec<(Hint, Rect)>,
}
pub(super) fn areas(app: &App, area: Rect) -> Areas {
    let wide = area.width >= WIDE_WIDTH;
    let hints = keymap::footer(app);
    // Lay the hints out once to learn how many footer rows they need.
    let probe = footer_slots(Rect::new(area.x, 0, area.width, 3), &hints);
    let footer_rows = probe.last().map_or(1, |(_, rect)| rect.y + 1).clamp(1, 2);
    let sections = Layout::vertical([
        Constraint::Length(if wide { 2 } else { 3 }),
        Constraint::Length(if wide { 1 } else { 2 }),
        Constraint::Min(4),
        Constraint::Length(2),
        Constraint::Length(footer_rows),
    ])
    .split(area);
    Areas {
        header: sections[0],
        nav: sections[1],
        body: sections[2],
        status: sections[3],
        slots: footer_slots(sections[4], &hints),
    }
}
/// Hints flow left to right and wrap to the next row of `area`.
pub(super) fn footer_slots(area: Rect, hints: &[Hint]) -> Vec<(Hint, Rect)> {
    let mut slots = Vec::new();
    let (mut x, mut y) = (area.x, area.y);
    for hint in hints {
        let width = text_width(&format!("{} {}", hint.key, hint.label)) as u16;
        if x > area.x && x + width > area.right() {
            x = area.x;
            y += 1;
        }
        if y >= area.bottom() {
            break;
        }
        slots.push((*hint, Rect::new(x, y, width.min(area.right() - x), 1)));
        x += width + 2;
    }
    slots
}
pub(super) fn nav_label(app: &App, page: Page) -> String {
    format!(" F{} {}", page.index() + 1, page.label(app.tt()))
}
/// Eight navigation cells: one row on wide terminals, two rows of four.
/// Cells are sized to their labels when they fit, else split evenly.
pub(super) fn nav_rects(area: Rect, app: &App) -> [Rect; 8] {
    let columns: u16 = if area.height >= 2 { 4 } else { 8 };
    let widths: Vec<u16> = Page::ALL
        .iter()
        .map(|page| text_width(&nav_label(app, *page)) as u16)
        .collect();
    let fits = widths
        .chunks(usize::from(columns))
        .all(|row| row.iter().sum::<u16>() <= area.width);
    if fits && columns == 8 {
        let mut x = area.x;
        return std::array::from_fn(|index| {
            let rect = Rect::new(x, area.y, widths[index], 1);
            x += widths[index];
            rect
        });
    }
    std::array::from_fn(|index| {
        let column = index as u16 % columns;
        let row = index as u16 / columns;
        let left = area.width * column / columns;
        let right = area.width * (column + 1) / columns;
        Rect::new(area.x + left, area.y + row, right - left, 1)
    })
}
/// Tab hitboxes `(tab index, rect)`. The strip scrolls just enough to keep
/// the active tab visible, so no scroll state is needed.
pub(super) fn tab_rects(area: Rect, labels: &[&str], active: usize) -> Vec<(usize, Rect)> {
    let widths: Vec<u16> = labels.iter().map(|l| 2 + text_width(l) as u16).collect();
    let mut first = 0;
    while first < active
        && widths[first..=active.min(widths.len() - 1)]
            .iter()
            .map(|w| w + 1)
            .sum::<u16>()
            > area.width
    {
        first += 1;
    }
    let mut rects = Vec::new();
    let mut x = area.x;
    for (index, width) in widths.iter().enumerate().skip(first) {
        if x + width > area.right() {
            break;
        }
        rects.push((index, Rect::new(x, area.y, *width, 1)));
        x += width + 1;
    }
    rects
}
/// First visible row of a bordered list: the previous offset, moved only
/// as far as needed to keep `selected` visible.
pub(super) fn list_offset(previous: usize, selected: usize, list: Rect) -> usize {
    let visible = usize::from(list.height.saturating_sub(2)).max(1);
    previous
        .min(selected)
        .max(selected.saturating_sub(visible - 1))
}
/// The list row under a click inside a bordered list, if any.
pub(super) fn list_row(list: Rect, x: u16, y: u16, offset: usize) -> Option<usize> {
    let inside = x > list.x && x + 1 < list.right() && y > list.y && y + 1 < list.bottom();
    inside.then(|| usize::from(y - list.y - 1) + offset)
}
/// Settings: tab strip, list and (wide terminals) a details panel.
pub(super) fn settings_areas(body: Rect) -> (Rect, Rect, Option<Rect>) {
    let parts = Layout::vertical([Constraint::Length(1), Constraint::Min(3)]).split(body);
    if body.width >= WIDE_WIDTH {
        let halves = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(parts[1]);
        (parts[0], halves[0], Some(halves[1]))
    } else {
        (parts[0], parts[1], None)
    }
}
pub(super) struct BrowserAreas {
    pub tabs: Rect,
    pub search: Rect,
    pub list: Rect,
    pub details: Option<Rect>,
}
pub(super) fn browser_areas(body: Rect, wide: bool) -> BrowserAreas {
    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(3),
    ])
    .split(body);
    let (list, details) = if wide {
        let halves = Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(rows[2]);
        (halves[0], Some(halves[1]))
    } else {
        (rows[2], None)
    };
    BrowserAreas {
        tabs: rows[0],
        search: rows[1],
        list,
        details,
    }
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
/// Geometry of the value, language and options lists.
pub(super) fn list_modal_rect(area: Rect, rows: usize) -> Rect {
    let width = area.width.saturating_sub(4).min(56);
    let height = (rows as u16 + 2).min(area.height.saturating_sub(2));
    centered(area, width, height)
}

pub(super) fn render(frame: &mut Frame<'_>, app: &App) {
    let t = app.t();
    let area = frame.area();
    frame.render_widget(
        Block::new().style(Style::default().fg(TEXT).bg(BACKGROUND)),
        area,
    );
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        let text = format!(
            "{}\n{}\n{}\n{}",
            fill(t.resize_to, &[&MIN_WIDTH, &MIN_HEIGHT]),
            t.resize_quit,
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
    let areas = areas(app, area);
    render_header(frame, areas.header, app);
    render_nav(frame, areas.nav, app);
    match app.page {
        Page::Home => pages::home(frame, areas.body, app),
        Page::Settings => pages::settings(frame, areas.body, app),
        Page::Features | Page::Research => pages::browser(frame, areas.body, app),
        Page::Devices => pages::devices(frame, areas.body, app),
        Page::Peers => pages::peers(frame, areas.body, app),
        Page::Audio => pages::audio(frame, areas.body, app),
        Page::Diagnostics => pages::diagnostics(frame, areas.body, app),
    }
    render_status(frame, areas.status, app);
    for (hint, rect) in &areas.slots {
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    hint.key,
                    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(" {}", hint.label), Style::default().fg(MUTED)),
            ])),
            *rect,
        );
    }
    modals::render(frame, app);
}

pub(super) use super::app::fill;

fn state_span(label: &str, value: &str, good: bool) -> Vec<Span<'static>> {
    vec![
        Span::styled(
            label.trim_end().to_owned() + " ",
            Style::default().fg(MUTED),
        ),
        Span::styled(
            value.to_owned(),
            Style::default()
                .fg(if good { GOOD } else { WARN })
                .add_modifier(Modifier::BOLD),
        ),
    ]
}
fn field(label: &str, value: String) -> Vec<Span<'static>> {
    vec![
        Span::styled(
            label.trim_end().to_owned() + " ",
            Style::default().fg(MUTED),
        ),
        Span::raw(value),
    ]
}
const GAP: &str = "   ";

fn render_header(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let t = app.t();
    let control = match app.control_state {
        ControlSessionState::Connected => t.connected,
        ControlSessionState::Disconnected => t.disconnected,
        ControlSessionState::Connecting => t.status_connecting,
        ControlSessionState::Error => t.unavailable,
    };
    let link = match app.host_connection {
        HostConnectionState::Connected => t.connected,
        HostConnectionState::Disconnected => t.disconnected,
        HostConnectionState::Unknown => t.unknown,
    };
    let target = if app.address.is_empty() {
        t.target_none.to_owned()
    } else {
        let name = app
            .devices
            .iter()
            .flatten()
            .find(|d| d.address.as_str().eq_ignore_ascii_case(&app.address))
            .map(|d| format!(" ({})", clean(&d.name)))
            .unwrap_or_default();
        format!("{}{name}", clean(&app.address))
    };
    let battery = app
        .info
        .as_ref()
        .and_then(|info| info.battery())
        .map_or(t.unavailable.to_owned(), |v| fill(t.battery_pct, &[&v]));
    let snapshot = if app.stale {
        t.snapshot_stale.to_owned()
    } else if let Some(at) = app.observed_at {
        fill(t.snapshot_ago, &[&at.elapsed().as_secs()])
    } else {
        t.snapshot_never.to_owned()
    };
    let protocol = if app.model_confirmed {
        t.protocol_on
    } else {
        t.protocol_off
    };
    let mut first = vec![Span::styled(
        format!("{} ", t.app_title.trim()),
        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
    )];
    first.extend(state_span(t.control, control, app.connected()));
    first.push(Span::raw(GAP));
    first.extend(state_span(
        t.bluetooth,
        link,
        app.host_connection.is_connected(),
    ));
    first.push(Span::raw(GAP));
    first.extend(field(t.target, target));
    let mut second = field(t.battery, battery);
    second.push(Span::raw(GAP));
    second.extend(field(
        t.firmware,
        clean(app.firmware.as_deref().unwrap_or(t.unavailable)),
    ));
    second.push(Span::raw(GAP));
    second.extend(field(t.snapshot, snapshot));
    second.push(Span::raw(GAP));
    second.extend(field(t.protocol, protocol.to_owned()));
    frame.render_widget(
        Paragraph::new(vec![Line::from(first), Line::from(second)])
            .style(Style::default().fg(TEXT).bg(BAND))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_nav(frame: &mut Frame<'_>, area: Rect, app: &App) {
    for (page, rect) in Page::ALL.iter().zip(nav_rects(area, app)) {
        let style = if *page == app.page {
            Style::default()
                .fg(BACKGROUND)
                .bg(ACCENT)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(MUTED)
        };
        frame.render_widget(Paragraph::new(nav_label(app, *page)).style(style), rect);
    }
}

fn render_status(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let tt = app.tt();
    let mut spans = Vec::new();
    if app.working() {
        spans.push(Span::styled(
            format!("[{}] ", tt.busy),
            Style::default().fg(WARN).add_modifier(Modifier::BOLD),
        ));
    }
    spans.push(Span::raw(clean(&app.status)));
    frame.render_widget(
        Paragraph::new(Line::from(spans))
            .style(Style::default().fg(TEXT).bg(BAND))
            .wrap(Wrap { trim: true }),
        area,
    );
}
