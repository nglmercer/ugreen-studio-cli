//! Optional, explicitly operated terminal interface. Startup is entirely offline.
mod state;
mod view;
mod worker;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use state::{App, Intent};
use std::{
    io::{self, IsTerminal},
    time::Duration,
};
use worker::Worker;

/// Startup values do not trigger discovery, connections, or setting writes.
#[derive(Clone, Debug)]
pub struct Config {
    pub address: Option<String>,
    pub model_confirmed: bool,
    pub channel: u8,
    pub timeout: Duration,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            address: None,
            model_confirmed: false,
            channel: 1,
            timeout: Duration::from_secs(3),
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
    let result = ratatui::run(|terminal| -> io::Result<()> {
        let _stop_worker_on_exit = worker.shutdown_guard();
        loop {
            match worker.poll() {
                Ok(Some(reply)) => {
                    if let Some(action) = app.receive(reply) {
                        dispatch(&mut app, &mut worker, Intent::Request(action));
                    }
                }
                Ok(None) => {}
                Err(error) => {
                    app.worker_failed(error);
                    return Err(io::Error::other(app.status.clone()));
                }
            }
            app.record_status();
            terminal.draw(|frame| view::render(frame, &app))?;
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
                        if dispatch(&mut app, &mut worker, intent) {
                            break;
                        }
                    }
                    Event::Resize(_, _) => {} // draw uses the new size next pass
                    _ => {}
                }
            }
        }
        Ok(())
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
