//! Key hints: one table per context. The footer shows the current
//! context's hints (and makes them clickable); the help screen is
//! generated from the same tables, so documentation cannot drift.
use super::app::{App, Modal, Page};
use crate::i18n::tui::TuiText;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone, Copy, Debug)]
pub(super) struct Hint {
    pub key: &'static str,
    pub label: &'static str,
    /// The key a click dispatches; `None` for informational ranges.
    pub event: Option<KeyEvent>,
}
const fn press(code: KeyCode) -> Option<KeyEvent> {
    Some(KeyEvent::new(code, KeyModifiers::NONE))
}
fn hint(key: &'static str, label: &'static str, event: Option<KeyEvent>) -> Hint {
    Hint { key, label, event }
}
fn char_hint(key: &'static str, label: &'static str) -> Hint {
    let code = KeyCode::Char(key.chars().next().unwrap_or(' '));
    hint(key, label, press(code))
}

/// Bindings that work on every page when no modal is open.
pub(super) fn global(tt: &'static TuiText) -> Vec<Hint> {
    vec![
        hint("F1-F8", tt.h_pages, None),
        hint("[ ]", tt.h_pages, None),
        char_hint("a", tt.h_address),
        char_hint("p", tt.h_devices),
        char_hint("m", tt.h_model),
        char_hint("c", tt.h_connect),
        char_hint("d", tt.h_disconnect),
        char_hint("r", tt.h_refresh),
        char_hint("s", tt.h_options),
        char_hint("L", tt.h_language),
        char_hint("l", tt.h_log),
        char_hint("?", tt.h_help),
        hint("Esc", tt.h_cancel, press(KeyCode::Esc)),
        char_hint("q", tt.h_quit),
    ]
}

/// Page-specific bindings, also listed in the help screen.
pub(super) fn page(app: &App, page: Page) -> Vec<Hint> {
    let tt = app.tt();
    let mut hints = match page {
        Page::Home => {
            let mut hints = Vec::new();
            if app.connected() {
                hints.push(char_hint("r", tt.h_refresh));
                hints.push(char_hint("d", tt.h_disconnect));
            } else {
                hints.push(char_hint("c", tt.h_connect));
                hints.push(char_hint("a", tt.h_address));
                hints.push(char_hint("p", tt.h_devices));
                hints.push(char_hint("m", tt.h_model));
            }
            hints
        }
        Page::Settings => vec![
            hint("↑↓", tt.h_move, None),
            hint("←→", tt.h_change, None),
            hint("Enter", tt.h_review, press(KeyCode::Enter)),
            hint("Tab", tt.h_tabs, press(KeyCode::Tab)),
            hint("1-9", tt.h_jump, None),
            hint("Esc", tt.h_discard, press(KeyCode::Esc)),
            char_hint("r", tt.h_refresh),
        ],
        Page::Features | Page::Research => vec![
            hint("↑↓", tt.h_move, None),
            hint("Tab", tt.h_tabs, press(KeyCode::Tab)),
            char_hint("/", tt.h_search),
            hint(
                "Enter",
                if app.width >= super::view::WIDE_WIDTH {
                    tt.h_open
                } else {
                    tt.h_details
                },
                press(KeyCode::Enter),
            ),
            hint("Esc", tt.h_back, press(KeyCode::Esc)),
        ],
        Page::Devices => vec![
            hint("↑↓", tt.h_move, None),
            hint("Enter", tt.h_select, press(KeyCode::Enter)),
            char_hint("r", tt.h_refresh),
            char_hint("h", tt.h_link_on),
            char_hint("H", tt.h_link_off),
        ],
        Page::Peers => vec![
            char_hint("r", tt.h_query_peers),
            hint("1/2", tt.h_close_peer, None),
            char_hint("e", tt.h_dual),
            char_hint("R", tt.h_unlock),
            hint("↑↓", tt.h_scroll, None),
        ],
        Page::Audio => vec![
            char_hint("r", tt.h_refresh),
            char_hint("k", tt.h_codec),
            hint("↑↓", tt.h_scroll, None),
        ],
        Page::Diagnostics => vec![char_hint("r", tt.h_refresh), hint("↑↓", tt.h_scroll, None)],
    };
    if app.page == page {
        hints.push(char_hint("?", tt.h_help));
        hints.push(char_hint("q", tt.h_quit));
    }
    hints
}

/// Bindings of an open modal.
pub(super) fn modal(tt: &'static TuiText, modal: &Modal) -> Vec<Hint> {
    let esc = |label| hint("Esc", label, press(KeyCode::Esc));
    let enter = |label| hint("Enter", label, press(KeyCode::Enter));
    match modal {
        Modal::Help { .. } => vec![hint("↑↓", tt.h_scroll, None), esc(tt.h_back)],
        Modal::Log { .. } => vec![
            hint("↑↓", tt.h_move, None),
            hint("Home/End", tt.h_move, None),
            esc(tt.h_back),
        ],
        Modal::Options { .. } => vec![
            hint("↑↓", tt.h_move, None),
            hint("←→", tt.h_change, None),
            enter(tt.h_open),
            esc(tt.h_back),
        ],
        Modal::Language { .. } | Modal::Values { .. } => vec![
            hint("↑↓", tt.h_move, None),
            enter(tt.h_select),
            esc(tt.h_back),
        ],
        Modal::Address { .. } => vec![
            enter(tt.h_apply),
            hint(
                "Ctrl+U",
                tt.h_clear,
                Some(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL)),
            ),
            esc(tt.h_cancel),
        ],
        Modal::Model => vec![char_hint("y", tt.h_confirm), esc(tt.h_cancel)],
        Modal::Form(_) => vec![
            hint("↑↓", tt.h_move, None),
            hint("←→", tt.h_change, None),
            enter(tt.h_review),
            hint("Backspace", tt.h_unset, press(KeyCode::Backspace)),
            esc(tt.h_back),
        ],
        Modal::Review(_) => vec![enter(tt.h_apply), esc(tt.h_cancel)],
        Modal::Details => vec![
            hint("↑↓", tt.h_move, None),
            enter(tt.h_open),
            esc(tt.h_back),
        ],
    }
}

/// What the footer shows right now.
pub(super) fn footer(app: &App) -> Vec<Hint> {
    let tt = app.tt();
    if app.quit_confirmation {
        return vec![
            char_hint("q", tt.h_quit),
            hint("Esc", tt.h_back, press(KeyCode::Esc)),
        ];
    }
    let mut hints = Vec::new();
    if app.working() {
        hints.push(hint("Esc", tt.h_cancel, press(KeyCode::Esc)));
    }
    if let Some(open) = &app.modal {
        hints.extend(modal(tt, open));
        return hints;
    }
    if app.browser_searching() {
        hints.push(hint("Enter", tt.h_done, press(KeyCode::Enter)));
        hints.push(hint(
            "Ctrl+U",
            tt.h_clear,
            Some(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL)),
        ));
        return hints;
    }
    hints.extend(page(app, app.page));
    hints
}
