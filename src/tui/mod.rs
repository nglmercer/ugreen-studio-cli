//! Optional, explicitly operated terminal interface. Startup is entirely offline.
mod app;
mod input;
mod keymap;
mod view;
mod worker;
mod workspace;

use crate::i18n::Lang;
use app::{App, Intent};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use std::{
    io::{self, IsTerminal},
    time::Duration,
};
use worker::Worker;

/// Startup values do not trigger discovery or setting writes. The cached
/// target is restored, and the interface connects to it at startup unless
/// `autoconnect` is off (`--no-autoconnect`). Startup auto-loads the
/// paired cache when there is no cached target (tests set `skip_autoload`).
#[derive(Clone, Debug)]
pub struct Config {
    pub address: Option<String>,
    pub model_confirmed: bool,
    pub channel: u8,
    pub timeout: Duration,
    pub lang: Lang,
    pub skip_autoload: bool,
    pub autoconnect: bool,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            address: None,
            model_confirmed: false,
            channel: 1,
            timeout: Duration::from_secs(3),
            lang: Lang::En,
            skip_autoload: true,
            autoconnect: true,
        }
    }
}
/// Run interactively on a real terminal. Ratatui restores raw mode and the
/// alternate screen on ordinary errors, normal exit, and Rust panics.
pub fn run(config: Config) -> io::Result<()> {
    let t = crate::i18n::txt(config.lang);
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, t.tui_no_tty));
    }
    if !(1..=30).contains(&config.channel)
        || config.timeout.is_zero()
        || config.timeout > Duration::from_secs(60)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            t.tui_bad_channel,
        ));
    }
    let mut app = App::new(&config);
    let mut worker = Worker::native(config)?;
    let first = app.startup();
    dispatch(&mut app, &mut worker, first);
    let result = ratatui::run(|terminal| -> io::Result<()> {
        let _stop_worker_on_exit = worker.shutdown_guard();
        // Mouse clicks/wheel need explicit capture; ratatui restores it after
        // the closure. Non-interactive terminals or headless backends fail the
        // enable call; the TUI keeps working with keyboard only.
        let mouse_on = execute!(io::stdout(), event::EnableMouseCapture).is_ok();
        let outcome = event_loop(terminal, &mut app, &mut worker);
        if mouse_on {
            let _ = execute!(io::stdout(), event::DisableMouseCapture);
        }
        outcome
    });
    // Dropping never waits for a native operation; cancellation prevents later
    // stages, but bytes already handed to the OS cannot be recalled.
    let possibly_inflight = app
        .busy
        .as_ref()
        .is_some_and(|(_, action)| action.is_write());
    drop(worker);
    if possibly_inflight {
        eprintln!("{}", app.t().cli_inflight_after_quit);
    }
    result
}
fn event_loop(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
    worker: &mut Worker,
) -> io::Result<()> {
    let host_signals = crate::bluetooth::HostSignalMonitor::start();
    let mut last_host_refresh = std::time::Instant::now();
    loop {
        // Host link state refreshes in the background while the worker is
        // free; it never blocks input and yields to user requests.
        if app.busy.is_none()
            && app.queued.is_none()
            && app.worker_alive
            && worker::validate_address(&app.address).is_ok()
            && (host_signals.take_changed()
                || last_host_refresh.elapsed() >= Duration::from_secs(5))
        {
            last_host_refresh = std::time::Instant::now();
            let address = app.address.clone();
            dispatch(
                app,
                worker,
                Intent::Request(worker::Action::HostRefresh { address }),
            );
        }
        match worker.poll() {
            Ok(Some(reply)) => {
                if let Some(action) = app.receive(reply) {
                    dispatch(app, worker, Intent::Request(action));
                }
                if app.busy.is_none() {
                    if let Some(action) = app.queued.take() {
                        dispatch(app, worker, Intent::Request(action));
                    }
                }
            }
            Ok(None) => {}
            Err(error) => {
                app.worker_failed(error);
                return Err(io::Error::other(app.status.clone()));
            }
        }
        let size = terminal.size()?;
        app.width = size.width;
        app.height = size.height;
        app.record_status();
        terminal.draw(|frame| view::render(frame, app))?;
        if !event::poll(Duration::from_millis(50))? {
            continue;
        }
        let intent = match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                let area = terminal.size()?;
                let too_small = area.width < view::MIN_WIDTH || area.height < view::MIN_HEIGHT;
                let allowed = matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
                    || (app.quit_confirmation && key.code == KeyCode::Enter)
                    || (key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL));
                if too_small && !allowed {
                    continue;
                }
                app.key(key)
            }
            Event::Mouse(mouse) => {
                let area = terminal.size()?;
                app.mouse(mouse, area.width, area.height)
            }
            _ => Intent::None,
        };
        if dispatch(app, worker, intent) {
            break;
        }
    }
    Ok(())
}
/// Send an intent to the worker. A request made while only the background
/// host refresh runs cancels that refresh and waits in `queued`.
fn dispatch(app: &mut App, worker: &mut Worker, intent: Intent) -> bool {
    match intent {
        Intent::None => {}
        Intent::Request(action) => {
            if app.background() {
                worker.cancel();
                app.queued = Some(action);
                return false;
            }
            match worker.submit(action.clone()) {
                Ok(id) => app.accepted(id, action),
                Err(error) => app.status = error,
            }
        }
        Intent::Cancel => worker.cancel(),
        Intent::Quit => {
            worker.cancel();
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests;
