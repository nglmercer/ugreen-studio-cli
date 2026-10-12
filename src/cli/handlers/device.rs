//! Device-facing commands: status, discovery,
//! session capture and the capture-gated queries.
use crate::cli::context::{connect, fill, terminal_text, Invocation};
use crate::models::Capabilities as C;
use crate::settings;

/// Which capture-gated query a feature command runs.
enum FeatureQuery {
    Peers,
    Codec,
    Id,
    Color,
    BoxMac,
}

/// Shared path for the capture-gated queries: the
/// capability gate refuses before any connection.
fn feature(invocation: &Invocation, query: FeatureQuery) -> Result<(), String> {
    let (bit, feature) = match query {
        FeatureQuery::Peers => (C::MULTIPOINT_PEERS, "multipoint peer list"),
        FeatureQuery::Codec => (C::CODEC_STATUS, "codec status"),
        FeatureQuery::Id => (C::HEADPHONE_ID, "headphone ID"),
        FeatureQuery::Color => (C::HEADPHONE_COLOR, "headphone color"),
        FeatureQuery::BoxMac => (C::BOX_MAC, "box MAC"),
    };
    if invocation.options.dry_run {
        return Err("use candidate for offline static query packets".into());
    }
    if !crate::models::Model::StudioPro.capabilities().contains(bit) {
        return Err(format!(
            "{feature} is not verified for Studio Pro; official-app capture required"
        ));
    }
    let mut client = connect(&invocation.options)?;
    match query {
        FeatureQuery::Peers => {
            let peers = client.peers().map_err(|e| e.to_string())?;
            for peer in peers.slots.into_iter().flatten() {
                println!(
                    "peer={} address={} name={}",
                    peer.index,
                    peer.address,
                    terminal_text(&peer.name)
                );
            }
        }
        FeatureQuery::Codec => {
            println!("codecs={:?}", client.codecs().map_err(|e| e.to_string())?)
        }
        FeatureQuery::Id => {
            println!("id={}", client.headphone_id().map_err(|e| e.to_string())?)
        }
        FeatureQuery::Color => println!(
            "color={}",
            client.headphone_color().map_err(|e| e.to_string())?
        ),
        FeatureQuery::BoxMac => {
            println!("box_mac={}", client.box_mac().map_err(|e| e.to_string())?)
        }
    }
    Ok(())
}

pub fn peers(invocation: &Invocation) -> Result<(), String> {
    feature(invocation, FeatureQuery::Peers)
}

pub fn codec(invocation: &Invocation) -> Result<(), String> {
    feature(invocation, FeatureQuery::Codec)
}

pub fn id(invocation: &Invocation) -> Result<(), String> {
    feature(invocation, FeatureQuery::Id)
}

pub fn color(invocation: &Invocation) -> Result<(), String> {
    feature(invocation, FeatureQuery::Color)
}

pub fn box_mac(invocation: &Invocation) -> Result<(), String> {
    feature(invocation, FeatureQuery::BoxMac)
}

pub fn status(invocation: &Invocation) -> Result<(), String> {
    let t = invocation.txt();
    if invocation.options.dry_run {
        return Err("status cannot be combined with --dry-run; use decode for offline data".into());
    }
    let mut client = connect(&invocation.options)?;
    let info = client.info().map_err(|e| format!("status failed: {e}"))?;
    println!("{}", t.cli_model_note);
    println!(
        "battery={}",
        info.battery()
            .map(|value| format!("{value}%"))
            .unwrap_or(t.cli_unknown_value.into())
    );
    for key in settings::KEYS {
        println!(
            "{key}={}",
            info.value(key).unwrap_or(t.cli_unknown_value.into())
        );
    }
    let firmware = client
        .firmware()
        .map_err(|e| fill(t.cli_firmware_fail, &[&e.to_string()]))?;
    println!("firmware={firmware}");
    println!("raw_device_info={}", crate::protocol::hex(&info.raw));
    Ok(())
}

pub fn discover(invocation: &Invocation) -> Result<(), String> {
    let t = invocation.txt();
    if invocation.options.dry_run {
        return Err(t.cli_tui_dry.into());
    }
    let mut devices =
        crate::bluetooth::list_devices().map_err(|e| fill(t.cli_paired_fail, &[&e.to_string()]))?;
    if devices.is_empty() {
        println!("{}", t.cli_no_paired);
    }
    crate::device::DeviceRegistry::load().annotate(&mut devices);
    for device in devices {
        match device.model {
            Some(model) => println!(
                "{}  {}  model={}",
                device.address,
                terminal_text(&device.name),
                model.as_str()
            ),
            None => println!("{}  {}", device.address, terminal_text(&device.name)),
        }
    }
    Ok(())
}

pub fn capture(invocation: &Invocation) -> Result<(), String> {
    let t = invocation.txt();
    if invocation.options.dry_run {
        return Err("capture cannot be combined with --dry-run; it records a live session".into());
    }
    let mut client = connect(&invocation.options)?;
    // Record the whole session even when a query fails: a
    // failed capture is exactly when the bytes matter most.
    // stdout stays pure fixture hex; context goes to stderr.
    let info = client.info();
    let firmware = client.firmware();
    let (_transport, log) = client.into_inner().into_inner_and_log();
    let export = log
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .export();
    print!("{export}");
    let info = info.map_err(|e| fill(t.cli_preflight_fail, &[&e.to_string()]))?;
    eprintln!(
        "battery={}",
        info.battery()
            .map(|value| format!("{value}%"))
            .unwrap_or(t.cli_unknown_value.into())
    );
    match firmware {
        Ok(firmware) => eprintln!("firmware={firmware}"),
        Err(e) => eprintln!("{}", fill(t.cli_firmware_fail, &[&e.to_string()])),
    }
    Ok(())
}
