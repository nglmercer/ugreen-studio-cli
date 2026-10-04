//! Optional, explicitly operated terminal interface. Startup is entirely offline.
mod state;
mod view;
mod worker;

use crate::cache;
use crate::i18n::Lang;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use state::{App, Intent};
use std::{
    io::{self, IsTerminal},
    time::Duration,
};
use worker::Worker;

/// Startup values do not trigger discovery, connections, or setting
/// writes unless `autoconnect` asks to connect to a cached target.
/// Startup auto-loads the paired cache unless `skip_autoload` is set
/// (tests).
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
            autoconnect: false,
        }
    }
}
/// Run interactively on a real terminal. Ratatui restores raw mode and the
/// alternate screen on ordinary errors, normal exit, and Rust panics.
pub fn run(config: Config) -> io::Result<()> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,
            "The TUI requires interactive stdin and stdout terminals; run ugreen --help for CLI commands."));
    }
    if !(1..=30).contains(&config.channel)
        || config.timeout.is_zero()
        || config.timeout > Duration::from_secs(60)
    {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "TUI channel must be 1-30 and timeout must be greater than zero and at most 60 seconds."));
    }
    let mut app = App::new(&config);
    let mut worker = Worker::native(config)?;
    // Startup stays offline unless auto-connect is enabled with a
    // cached, confirmed target; then it connects directly. Failure
    // only sets status text; startup never writes settings.
    if !app.config_skip_autoload() {
        if app.config_autoconnect() && !app.address.is_empty() && app.model_confirmed {
            let action = worker::Action::Connect {
                address: app.address.clone(),
                model_confirmed: true,
            };
            match worker.submit(action.clone()) {
                Ok(id) => app.accepted(id, action),
                Err(error) => app.status = error,
            }
        } else if app.address.is_empty() {
            match worker.submit(worker::Action::Paired) {
                Ok(id) => app.accepted_loading(id),
                Err(error) => app.status = error,
            }
        }
    }
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
        eprintln!("An in-flight setting write may have completed. Explicitly refresh device status before making further changes.");
    }
    result
}
fn event_loop(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
    worker: &mut Worker,
) -> io::Result<()> {
    loop {
        match worker.poll() {
            Ok(Some(reply)) => {
                let was_connected = app.connected;
                if let Some(action) = app.receive(reply) {
                    dispatch(app, worker, Intent::Request(action));
                }
                // Remember the target that answered, so the next start
                // can offer it and auto-connect when enabled.
                if !was_connected && app.connected {
                    cache::save(&cache::Cache {
                        address: Some(app.address.clone()),
                        model_confirmed: true,
                        channel: Some(app.channel),
                        lang: Some(app.lang),
                        autoconnect: app.config_autoconnect(),
                    });
                }
            }
            Ok(None) => {}
            Err(error) => {
                app.worker_failed(error);
                return Err(io::Error::other(app.status.clone()));
            }
        }
        app.record_status();
        terminal.draw(|frame| view::render(frame, app))?;
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    let area = terminal.size()?;
                    if (area.width < view::MIN_WIDTH || area.height < view::MIN_HEIGHT)
                        && !matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
                        && !(app.quit_confirmation && key.code == KeyCode::Enter)
                        && !(key.code == KeyCode::Char('c')
                            && key.modifiers.contains(KeyModifiers::CONTROL))
                    {
                        continue;
                    }
                    let intent = app.key(key);
                    if dispatch(app, worker, intent) {
                        break;
                    }
                }
                Event::Mouse(mouse) => {
                    let area = terminal.size()?;
                    let intent = app.mouse(mouse, area.width, area.height);
                    if dispatch(app, worker, intent) {
                        break;
                    }
                }
                Event::Resize(_, _) => {} // draw uses the new size next pass
                _ => {}
            }
        }
    }
    Ok(())
}
fn dispatch(app: &mut App, worker: &mut Worker, intent: Intent) -> bool {
    match intent {
        Intent::None => {}
        Intent::Request(action) => match worker.submit(action.clone()) {
            Ok(id) => app.accepted(id, action),
            Err(error) => app.status = error,
        },
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
