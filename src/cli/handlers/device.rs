//! Device-facing commands: status, discovery,
//! session capture and the capture-gated queries.
use crate::cli::context::{connect, fill, terminal_text, Invocation};
use crate::features::{FeatureId, Operation};
use crate::settings;

/// Shared path for the capture-gated queries: the
/// capability gate refuses before any connection.
fn feature(invocation: &Invocation, id: FeatureId) -> Result<(), String> {
    let spec = crate::features::lookup(id).ok_or("feature is not registered")?;
    let query = spec.operation(&[])?;
    if invocation.options.dry_run {
        return Err("use candidate for offline static query packets".into());
    }
    query
        .check(crate::models::Model::StudioPro.capabilities())
        .map_err(|error| error.to_string())?;
    let mut client = connect(&invocation.options)?;
    match query {
        Operation::Peers => {
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
        Operation::Codec => {
            println!("codecs={:?}", client.codecs().map_err(|e| e.to_string())?)
        }
        Operation::Id => {
            println!("id={}", client.headphone_id().map_err(|e| e.to_string())?)
        }
        Operation::Color => println!(
            "color={}",
            client.headphone_color().map_err(|e| e.to_string())?
        ),
        Operation::BoxMac => {
            println!("box_mac={}", client.box_mac().map_err(|e| e.to_string())?)
        }
        _ => return Err("registered query is not a device read".into()),
    }
    Ok(())
}

pub fn peers(invocation: &Invocation) -> Result<(), String> {
    feature(invocation, FeatureId::MultipointPeers)
}

pub fn codec(invocation: &Invocation) -> Result<(), String> {
    feature(invocation, FeatureId::CodecStatus)
}

pub fn id(invocation: &Invocation) -> Result<(), String> {
    feature(invocation, FeatureId::HeadphoneId)
}

pub fn color(invocation: &Invocation) -> Result<(), String> {
    feature(invocation, FeatureId::HeadphoneColor)
}

pub fn box_mac(invocation: &Invocation) -> Result<(), String> {
    feature(invocation, FeatureId::BoxMac)
}

pub fn status(invocation: &Invocation) -> Result<(), String> {
    let t = invocation.txt();
    if invocation.options.dry_run {
        return Err("status cannot be combined with --dry-run; use decode for offline data".into());
    }
    let mut client = connect(&invocation.options)?;
    let info = client
        .info_in(invocation.lang())
        .map_err(|e| fill(t.cli_preflight_fail, &[&e.to_string()]))?;
    if invocation.options.json {
        let firmware = client
            .firmware_in(invocation.lang())
            .map_err(|e| e.to_string())?;
        let address = crate::bluetooth::BluetoothAddress::parse(
            invocation
                .options
                .address
                .as_deref()
                .ok_or(t.cli_need_address)?,
        )
        .map_err(|error| error.to_string())?;
        println!(
            "{}",
            crate::application::status_json(
                &address,
                crate::models::Model::StudioPro,
                &firmware,
                &info
            )
        );
        return Ok(());
    }
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
        .firmware_in(invocation.lang())
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
    if devices.is_empty() && !invocation.options.json {
        println!("{}", t.cli_no_paired);
    }
    crate::device::DeviceRegistry::load().annotate(&mut devices);
    if invocation.options.json {
        let rows: Vec<_> = devices.iter().map(|d| serde_json::json!({"address":d.address.as_str(),"name":d.name,"paired":d.paired,"host_link":d.connected.as_str(),"model":d.model.map(|m|m.as_str()),"control_session":"unknown","audio_transport":"unknown"})).collect();
        println!(
            "{}",
            serde_json::json!({"schema_version":1,"kind":"devices","devices":rows})
        );
        return Ok(());
    }
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
    let info = client.info_in(invocation.lang());
    let firmware = client.firmware_in(invocation.lang());
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
