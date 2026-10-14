//! Parsed command-line context shared by every handler:
//! global options, the resolved command and its arguments.
use super::registry::CommandSpec;
use crate::{
    bluetooth::{self, BluetoothAddress},
    client::Client,
    i18n::{txt, Lang, L},
    protocol,
};
use std::{fs, io::Read, time::Duration};

/// Global flags accepted before the command name.
#[derive(Debug, Default, Clone)]
pub struct Options {
    pub address: Option<String>,
    pub model: Option<String>,
    pub channel: Option<u8>,
    pub timeout: Option<Duration>,
    pub dry_run: bool,
    pub json: bool,
    pub lang: Option<Lang>,
    pub autoconnect: Option<bool>,
}

/// One parsed invocation: global options, the resolved
/// command, its subcommand when nested, and its arguments.
pub struct Invocation {
    pub options: Options,
    pub command: &'static CommandSpec,
    pub subcommand: Option<&'static CommandSpec>,
    pub args: Vec<String>,
}

impl Invocation {
    /// Effective language: explicit `--lang`, else environment.
    pub fn lang(&self) -> Lang {
        self.options.lang.unwrap_or_else(Lang::detect)
    }
    /// Localized catalog for the effective language.
    pub fn txt(&self) -> &'static L {
        txt(self.lang())
    }
    /// The spec that executes: the subcommand when the
    /// command has nested subcommands, else the command.
    pub fn target(&self) -> &'static CommandSpec {
        self.subcommand.unwrap_or(self.command)
    }
}

/// Substitute `{}` placeholders in argument order.
pub fn fill(template: &str, args: &[&dyn std::fmt::Display]) -> String {
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

/// Escape control characters so device names cannot
/// control the terminal.
pub fn terminal_text(text: &str) -> String {
    text.chars()
        .flat_map(|ch| {
            if ch.is_control() {
                ch.escape_default().collect::<Vec<_>>()
            } else {
                vec![ch]
            }
        })
        .collect()
}

/// Every live command connects through here, already wrapped in a
/// recording tee: most callers ignore the log, `capture` exports it.
pub fn connect(
    options: &Options,
) -> Result<Client<protocol::Monitored<bluetooth::Connection>>, String> {
    let t = txt(options.lang.unwrap_or_else(Lang::detect));
    if options.model.as_deref() != Some("studio-pro") {
        return Err(t.cli_need_model.into());
    }
    let address = options.address.as_deref().ok_or(t.cli_need_address)?;
    eprintln!(
        "{}",
        fill(
            t.cli_banner,
            &[&address, &options.channel.unwrap_or(1).to_string()]
        )
    );
    let timeout = options.timeout.unwrap_or(Duration::from_secs(3));
    BluetoothAddress::parse(address)
        .and_then(|address| {
            bluetooth::Connection::connect(&address, options.channel.unwrap_or(1), timeout)
        })
        .map(|io| {
            Client::new(
                protocol::Monitored::new(io, protocol::shared_monitor()),
                timeout,
            )
        })
        .map_err(|e| fill(t.cli_connect_fail, &[&e.to_string()]))
}

/// Read a bounded profile file; oversized profiles are
/// rejected before any parsing.
pub fn read_profile(path: &str) -> Result<String, String> {
    let f = fs::File::open(path).map_err(|e| format!("cannot open profile: {e}"))?;
    let mut text = String::new();
    f.take(16_385)
        .read_to_string(&mut text)
        .map_err(|e| format!("cannot read UTF-8 profile: {e}"))?;
    if text.len() > 16_384 {
        return Err("profile exceeds 16 KiB".into());
    }
    Ok(text)
}
