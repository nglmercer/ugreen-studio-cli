//! Entirely offline: state, rendering and worker tests use fake sessions only.
mod engine;
mod ui;

use super::{
    app::{App, Intent, Modal, Page},
    view,
    worker::{Action, Backend, Engine, Output, Reply, Request, Session, Snapshot, Worker},
    Config,
};
use crate::{
    bluetooth::{BluetoothAddress, HostConnectionState, HostHeadset},
    settings::{Setting, StudioProState},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::TestBackend, Terminal};
use std::{
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    time::{Duration, Instant},
};

fn info() -> StudioProState {
    let mut raw = vec![0; 30];
    raw[0] = 76;
    raw[3] = 0xa0;
    StudioProState::new(raw).unwrap()
}
/// A paired listing entry; addresses must parse or the test itself is wrong.
fn device(address: &str, name: &str) -> HostHeadset {
    HostHeadset::paired(BluetoothAddress::parse(address).unwrap(), name)
}
fn config() -> Config {
    Config {
        address: Some("AA:BB:CC:DD:EE:FF".into()),
        model_confirmed: true,
        ..Config::default()
    }
}
fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}
fn ch(c: char) -> KeyEvent {
    key(KeyCode::Char(c))
}
/// Complete one accepted request with a reply.
fn complete(
    app: &mut App,
    id: u64,
    result: Result<Output, String>,
    connected: bool,
) -> Option<Action> {
    app.receive(Reply {
        id,
        result,
        connected,
        writes_blocked: false,
    })
}
/// Accept whatever request an intent carries, as the event loop does.
fn accept(app: &mut App, id: u64, intent: Intent) -> Action {
    let Intent::Request(action) = intent else {
        panic!("expected a request");
    };
    app.accepted(id, action.clone());
    action
}
fn ready() -> App {
    let mut app = App::new(&config());
    app.accepted(
        1,
        Action::Connect {
            address: app.address.clone(),
            model_confirmed: true,
        },
    );
    complete(
        &mut app,
        1,
        Ok(Output::Snapshot(Snapshot {
            info: info(),
            firmware: Some("0.2.5".into()),
            note: None,
        })),
        true,
    );
    app
}
fn render(app: &App, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| view::render(frame, app)).unwrap();
    let buffer = terminal.backend().buffer();
    let mut out = String::new();
    for y in 0..height {
        for x in 0..width {
            out.push_str(buffer[(x, y)].symbol());
        }
        out.push('\n');
    }
    out
}
