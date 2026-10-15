mod dialogs;
mod features;
mod header;
mod screens;
use super::state::{choices, feature_tab_labels, App, FeatureForm, ObservationView, Review};
use crate::{
    bluetooth::HostConnectionState,
    features::{lookup, FeatureKind, FeatureSpec, SafetyLevel},
    i18n::{Lang, L},
    models::{Capabilities, Model},
    settings,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use features::{feature_detail_lines, render_features};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

pub(super) const MIN_WIDTH: u16 = 56;
pub(super) const MIN_HEIGHT: u16 = 20;
/// The feature listing and its details panel sit
/// side by side from this width on; below it the
/// details open as a separate view.
pub(super) const WIDE_WIDTH: u16 = 100;
const ACCENT: Color = Color::Cyan;
const MUTED: Color = Color::Gray;

/// Device names and errors are untrusted terminal text. Never render control
/// characters (including escape sequences), bidi overrides, or invisible marks.
pub(super) fn clean(text: &str) -> String {
    text.chars().map(|ch| if ch.is_control() || matches!(ch, '\u{200b}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2060}'..='\u{206f}' | '\u{feff}') { '�' } else { ch }).take(1024).collect()
}
/// The computed screen regions, shared by the renderer
/// and the mouse handler so a click maps to the row
/// that is actually drawn.
pub(super) struct Regions {
    pub header: Rect,
    pub navigation: Rect,
    pub tabs: Rect,
    pub search: Rect,
    pub list: Rect,
    pub status: Rect,
    pub footer: Rect,
}
pub(super) fn regions(area: Rect, features: bool) -> Regions {
    if features {
        let sections = Layout::vertical([
            Constraint::Length(8),
            Constraint::Length(if area.width < WIDE_WIDTH { 2 } else { 1 }),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(4),
            Constraint::Length(4),
            Constraint::Length(3),
        ])
        .split(area);
        Regions {
            header: sections[0],
            navigation: sections[1],
            tabs: sections[2],
            search: sections[3],
            list: sections[4],
            status: sections[5],
            footer: sections[6],
        }
    } else {
        let sections = Layout::vertical([
            Constraint::Length(8),
            Constraint::Length(if area.width < WIDE_WIDTH { 2 } else { 1 }),
            Constraint::Length(1),
            Constraint::Min(4),
            Constraint::Length(4),
            Constraint::Length(3),
        ])
        .split(area);
        Regions {
            header: sections[0],
            navigation: sections[1],
            tabs: sections[2],
            search: Rect::default(),
            list: sections[3],
            status: sections[4],
            footer: sections[5],
        }
    }
}
/// The settings footer plus its "F features" slot,
/// laid out the same way for rendering and clicks.
pub(super) fn footer_rows(t: &L) -> [[(&'static str, &'static str); 6]; 3] {
    let mut rows = t.footer;
    if let Some(slot) = rows[2].iter_mut().find(|(key, _)| key.is_empty()) {
        *slot = ("F", t.features_title.trim());
    }
    rows
}
/// Footer hitboxes: one rectangle per rendered
/// "k v  " slot, in row-major order.
pub(super) fn footer_slots(
    area: Rect,
    rows: &[[(&'static str, &'static str); 6]],
) -> Vec<(&'static str, Rect)> {
    let mut slots = Vec::new();
    for (row, entries) in rows.iter().enumerate() {
        let y = area.y + area.height.saturating_sub(3) + row as u16;
        if y >= area.bottom() {
            break;
        }
        let mut x = area.x;
        for (key, value) in entries {
            if key.is_empty() {
                continue;
            }
            let width = text_width(&format!("{key} {value}  ")) as u16;
            if x < area.right() {
                slots.push((*key, Rect::new(x, y, width.min(area.right() - x), 1)));
            }
            x += width;
        }
    }
    slots
}
/// The key event a footer slot dispatches.
pub(super) fn footer_key(key: &str) -> Option<KeyEvent> {
    let code = match key {
        "Enter" => KeyCode::Enter,
        "Esc" => KeyCode::Esc,
        "Tab" => KeyCode::Tab,
        "BackTab" => KeyCode::BackTab,
        "Up" => KeyCode::Up,
        "Down" => KeyCode::Down,
        "Left" => KeyCode::Left,
        "Right" => KeyCode::Right,
        "Backspace" => KeyCode::Backspace,
        "<" => KeyCode::Left,
        ">" => KeyCode::Right,
        "Ctrl+U" => {
            return Some(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL));
        }
        other if other.chars().count() == 1 => KeyCode::Char(other.chars().next().unwrap_or('?')),
        _ => return None,
    };
    Some(KeyEvent::new(code, KeyModifiers::NONE))
}
/// Tab-bar hitboxes: one rectangle per visible
/// " label " tab, starting at the scroll offset.
pub(super) fn tab_rects(area: Rect, labels: &[&'static str], scroll: usize) -> Vec<Rect> {
    let mut rects = Vec::new();
    let mut x = area.x;
    for label in labels.iter().skip(scroll) {
        let width = 2 + text_width(label) as u16;
        if x.saturating_add(width) > area.x + area.width {
            break;
        }
        rects.push(Rect::new(x, area.y, width, 1));
        x += width;
    }
    rects
}
pub(super) fn text_width(text: &str) -> usize {
    Line::from(text).width()
}
/// Six fixed cells keep mouse and keyboard navigation aligned at every size.
pub(super) fn navigation_rects(area: Rect) -> [Rect; 6] {
    let columns = if area.width < WIDE_WIDTH { 3 } else { 6 };
    std::array::from_fn(|index| {
        let column = index as u16 % columns;
        let row = index as u16 / columns;
        let left = area.width * column / columns;
        let right = area.width * (column + 1) / columns;
        Rect::new(area.x + left, area.y + row, right - left, 1)
    })
}
fn render_navigation(frame: &mut Frame<'_>, regions: &Regions, app: &App) {
    for (index, rect) in navigation_rects(regions.navigation).into_iter().enumerate() {
        let style = if index == app.route.index() {
            Style::default()
                .fg(Color::Black)
                .bg(ACCENT)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(MUTED)
        };
        frame.render_widget(
            Paragraph::new(format!("F{} {}", index + 1, app.t().navigation[index])).style(style),
            rect,
        );
    }
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
    let regions = regions(area, app.features);
    header::render(frame, &regions, app);
    render_navigation(frame, &regions, app);
    if let Some(view) = app.observation_view {
        screens::render_observation(frame, &regions, app, view);
    } else if app.paired.is_some() {
        screens::render_devices(frame, &regions, app);
    } else if app.features {
        render_features(frame, area, &regions, app);
    } else if app.route == super::routing::Route::Home {
        screens::render_home(frame, &regions, app);
    } else {
        screens::render_settings(frame, area, &regions, app);
    }
    let status_title = if app.busy.is_some() {
        t.status_working
    } else {
        t.status_title
    };
    frame.render_widget(
        Paragraph::new(clean(&app.status))
            .block(panel(status_title))
            .wrap(Wrap { trim: true }),
        regions.status,
    );
    let rows = if app.features {
        t.features_footer
    } else {
        footer_rows(t)
    };
    let footer: Vec<Line> = rows
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
    // Fixed rows keep footer hitboxes aligned. Long translations clip within
    // their own row instead of wrapping over a different row's shortcuts.
    for (row, line) in footer
        .into_iter()
        .take(usize::from(regions.footer.height))
        .enumerate()
    {
        frame.render_widget(
            Paragraph::new(line),
            Rect::new(
                regions.footer.x,
                regions.footer.y + row as u16,
                regions.footer.width,
                1,
            ),
        );
    }
    dialogs::render(frame, app);
}
fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let limit = width.max(1);
    let mut wrapped = Vec::new();
    for paragraph in text.split('\n') {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > limit {
                wrapped.push(line);
                line = String::new();
            }
            if line.is_empty() {
                line.push_str(word);
            } else {
                line.push(' ');
                line.push_str(word);
            }
        }
        wrapped.push(line);
    }
    wrapped
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
/// Shared geometry for the six page rows and mouse hit testing.
pub(super) fn pages_rect(area: Rect) -> Rect {
    centered(
        area,
        area.width.saturating_sub(4).min(100),
        9.min(area.height),
    )
}
