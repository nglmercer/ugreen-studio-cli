//! Offline application commands: help, version,
//! models, the settings listing, the TUI and the
//! Windows host output-device query.
#[cfg(feature = "tui")]
use crate::bluetooth::BluetoothAddress;
use crate::cli::{
    context::{fill, terminal_text, Invocation},
    help, registry,
};
use crate::i18n::Lang;
#[cfg(feature = "tui")]
use std::time::Duration;

pub fn help(invocation: &Invocation) -> Result<(), String> {
    match invocation.args.first() {
        None => {
            print!("{}", help::full(invocation.lang()));
        }
        Some(path) => {
            let path = registry::lookup_path(path)
                .ok_or_else(|| fill(invocation.txt().cli_unknown_command, &[path]))?;
            print!("{}", help::command(invocation.lang(), &path));
        }
    }
    Ok(())
}

pub fn version(invocation: &Invocation) -> Result<(), String> {
    let _ = invocation;
    println!("ugreen {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

pub fn models(invocation: &Invocation) -> Result<(), String> {
    println!("{}", invocation.txt().cli_models);
    Ok(())
}

pub fn commands(_invocation: &Invocation) -> Result<(), String> {
    println!("{}", crate::settings::commands_help());
    Ok(())
}

pub fn tui(invocation: &Invocation) -> Result<(), String> {
    if invocation.options.dry_run {
        return Err(invocation.txt().cli_tui_dry.into());
    }
    run_tui(&invocation.options)
}

#[cfg(feature = "tui")]
fn run_tui(options: &crate::cli::context::Options) -> Result<(), String> {
    // The remembered target, protocol choice, channel,
    // timeout and language fill defaults; explicit
    // flags always win.
    let registry = crate::device::registry::DeviceRegistry::load();
    let autoconnect = options.autoconnect.unwrap_or(registry.autoconnect);
    if let Some(autoconnect) = options.autoconnect {
        let mut registry = registry.clone();
        registry.autoconnect = autoconnect;
        registry.save();
    }
    let address = options.address.clone().or_else(|| {
        registry
            .selected
            .as_ref()
            .map(|address| address.to_string())
    });
    // The model is confirmed per address, never guessed.
    let model_confirmed = options.model.as_deref() == Some("studio-pro")
        || address
            .as_deref()
            .and_then(|raw| BluetoothAddress::parse(raw).ok())
            .is_some_and(|address| {
                registry.model_of(&address) == Some(crate::models::Model::StudioPro)
            });
    crate::tui::run(crate::tui::Config {
        address,
        model_confirmed,
        channel: options.channel.or(registry.channel).unwrap_or(1),
        timeout: options
            .timeout
            .or(registry.timeout.map(Duration::from_secs))
            .unwrap_or(Duration::from_secs(3)),
        lang: options.lang.or(registry.lang).unwrap_or_else(Lang::detect),
        skip_autoload: false,
        autoconnect,
    })
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(not(feature = "tui"))]
fn run_tui(options: &crate::cli::context::Options) -> Result<(), String> {
    Err(crate::i18n::txt(options.lang.unwrap_or_else(Lang::detect))
        .cli_no_tui_feature
        .into())
}

pub fn audio_devices(invocation: &Invocation) -> Result<(), String> {
    if invocation.options.dry_run {
        return Err("audio-devices reads host output devices; --dry-run is unsupported".into());
    }
    for (id, name) in crate::finder::windows_output_devices().map_err(|e| e.to_string())? {
        println!("{id} {}", terminal_text(&name));
    }
    Ok(())
}
