//! Thin executable entry: parse the command line,
//! dispatch through the registry, report errors.
//! All behavior lives in the `ugreen_cli` library.
use std::{env, process::ExitCode};

fn main() -> ExitCode {
    match ugreen_cli::cli::parse(env::args().skip(1).collect())
        .and_then(|invocation| ugreen_cli::cli::registry::dispatch(&invocation))
    {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
