//! Host diagnostics and read-only OS audio, separate from headset control.
use crate::{
    audio::{AudioStatusProvider, OsAudio},
    bluetooth::{self, BluetoothAddress},
    cli::context::{terminal_text, Invocation},
};
use serde_json::json;

pub fn audio_status(invocation: &Invocation) -> Result<(), String> {
    if invocation.options.dry_run {
        return Err("audio status reads the OS; --dry-run is unsupported".into());
    }
    let raw = invocation
        .options
        .address
        .as_deref()
        .ok_or(invocation.txt().cli_need_address)?;
    let address = BluetoothAddress::parse(raw).map_err(|e| e.to_string())?;
    let observations = OsAudio.observe(&address).map_err(|e| e.to_string())?;
    if invocation.options.json {
        println!(
            "{}",
            json!({"schema_version":1,"kind":"audio_status","address":address.as_str(),"observations":observations.iter().map(|v|v.json()).collect::<Vec<_>>()})
        );
    } else if observations.is_empty() {
        println!("{}", invocation.txt().unavailable);
    } else {
        for observation in observations {
            println!("{}", observation.json());
        }
    }
    Ok(())
}
pub fn capabilities(invocation: &Invocation) -> Result<(), String> {
    // No device connection: firmware evidence is deliberately absent.
    let features: Vec<_> = crate::features::FEATURES.iter().map(|spec| json!({
        "id":format!("{:?}",spec.id), "implementation_status":"tested_mock",
        "evidence_status":if spec.kind==crate::features::FeatureKind::VerifiedSetting {"verified"} else if spec.kind==crate::features::FeatureKind::OfflineAnalysis {"unsupported"} else {"static_candidate"},
        "verified_firmware":if spec.kind==crate::features::FeatureKind::VerifiedSetting {Some("0.2.5")} else {None},
        "operation_authorized":false,
    })).collect();
    println!(
        "{}",
        json!({"schema_version":1,"kind":"capabilities","model":invocation.options.model,"observed_firmware":null,"features":features})
    );
    Ok(())
}
pub fn host_action(invocation: &Invocation) -> Result<(), String> {
    if invocation.options.dry_run {
        return Err("host actions do not support --dry-run".into());
    }
    let address = BluetoothAddress::parse(
        invocation
            .options
            .address
            .as_deref()
            .ok_or(invocation.txt().cli_need_address)?,
    )
    .map_err(|e| e.to_string())?;
    let connected = invocation.target().name == "connect";
    bluetooth::set_host_connection(&address, connected).map_err(|e| e.to_string())?;
    println!(
        "{} {}",
        address,
        if connected {
            "host_connect_requested"
        } else {
            "host_disconnect_requested"
        }
    );
    Ok(())
}
pub fn doctor(invocation: &Invocation) -> Result<(), String> {
    if invocation.options.dry_run {
        return Err("doctor reads host diagnostics; --dry-run is unsupported".into());
    }
    let address = invocation
        .options
        .address
        .as_deref()
        .map(BluetoothAddress::parse)
        .transpose()
        .map_err(|e| e.to_string())?;
    let model = invocation
        .options
        .model
        .as_deref()
        .and_then(crate::models::Model::parse)
        .or_else(|| {
            address
                .as_ref()
                .and_then(|a| crate::device::DeviceRegistry::load().model_of(a))
        });
    let report = crate::diagnostics::collect(
        address.as_ref(),
        invocation.options.channel.unwrap_or(1),
        model,
    )
    .json();
    if invocation.options.json {
        println!("{report}");
    } else {
        println!(
            "{}",
            terminal_text(&serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?)
        );
    }
    Ok(())
}
