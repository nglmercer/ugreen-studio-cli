//! Shared application policy used by CLI, TUI and evidence-aware library calls.
//! Raw protocol requests remain a separate, caller-managed research interface.
use crate::{
    features,
    models::{self, Model},
    settings::Setting,
};
use std::io;

/// Versioned status representation; protocol keys/values are locale-independent.
/// Model is an explicit selection, not an identity inferred from a reply.
pub fn status_json(
    address: &crate::bluetooth::BluetoothAddress,
    model: Model,
    firmware: &str,
    info: &crate::settings::StudioProState,
) -> serde_json::Value {
    let values: serde_json::Map<String, serde_json::Value> = crate::settings::KEYS
        .iter()
        .map(|key| ((*key).into(), serde_json::json!(info.value(key))))
        .collect();
    serde_json::json!({"schema_version":1,"kind":"status","address":address.as_str(),
        "model":model.as_str(),"model_source":"user_selected","firmware":firmware,
        "battery_percent":info.battery(),"settings":values,
        "raw_device_info":crate::protocol::hex(&info.raw)})
}

/// Public Setting fields are retained for source compatibility. Validate all of
/// them against the canonical registry before permitting a setting operation.
pub fn validate_setting(setting: &Setting) -> io::Result<()> {
    let canonical = Setting::parse(&setting.key, &setting.value)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    if canonical != *setting {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "setting packet does not match its registered key/value",
        ));
    }
    Ok(())
}

/// A known firmware is insufficient if its current info payload cannot report
/// the requested field. Profiles validate every requested field before writing.
pub fn validate_preflight(
    info: &crate::settings::StudioProState,
    setting: &Setting,
) -> io::Result<()> {
    validate_setting(setting)?;
    if info.value(&setting.key).is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("setting '{}' missing from preflight", setting.key),
        ));
    }
    Ok(())
}

/// Historical model metadata is insufficient: authorization always uses the
/// observed exact firmware and the feature's required capability bit.
pub fn authorize_setting(
    model: Option<Model>,
    firmware: Option<&str>,
    setting: &Setting,
) -> io::Result<()> {
    validate_setting(setting)?;
    let spec = features::by_setting(&setting.key)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Unsupported, "setting is not registered"))?;
    authorize_capability(
        model,
        firmware,
        spec.capability.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Unsupported,
                "setting has no registered capability",
            )
        })?,
    )
}

/// Shared exact-firmware permission check for settings, queries and controls.
pub fn authorize_capability(
    model: Option<Model>,
    firmware: Option<&str>,
    required: u32,
) -> io::Result<()> {
    models::require_verified_firmware(model, firmware)?;
    if required == 0 || !models::capabilities_for(model, firmware).contains(required) {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "operation is not verified for the selected model and firmware",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_registered_value_requires_exact_firmware_and_canonical_packet() {
        for spec in crate::settings::SPECS {
            for (value, _) in spec.values {
                let setting = Setting::parse(spec.key, value).unwrap();
                assert!(authorize_setting(Some(Model::StudioPro), Some("0.2.5"), &setting).is_ok());
                for firmware in [None, Some("0.2.6"), Some("0.2.5 ")] {
                    assert!(authorize_setting(Some(Model::StudioPro), firmware, &setting).is_err());
                }
                assert!(authorize_setting(None, Some("0.2.5"), &setting).is_err());
                let mut forged = setting.clone();
                forged.instruction = 0x10;
                assert!(validate_setting(&forged).is_err());
                forged = setting.clone();
                forged.payload.push(0);
                assert!(validate_setting(&forged).is_err());
            }
        }
    }
    #[test]
    fn status_json_is_canonical_and_does_not_infer_model_identity() {
        let address = crate::bluetooth::BluetoothAddress::parse("aa:bb:cc:dd:ee:ff").unwrap();
        let mut raw = vec![0; 30];
        raw[0] = 255;
        let state = crate::settings::StudioProState::new(raw).unwrap();
        let value = status_json(&address, Model::StudioPro, "0.2.6", &state);
        assert_eq!(value["address"], "AA:BB:CC:DD:EE:FF");
        assert_eq!(value["model_source"], "user_selected");
        assert!(value["battery_percent"].is_null());
        assert_eq!(value["firmware"], "0.2.6");
        assert_eq!(
            value["settings"].as_object().unwrap().len(),
            crate::settings::KEYS.len()
        );
        assert!(authorize_capability(Some(Model::StudioPro), Some("0.2.5"), 0).is_err());
        assert!(authorize_capability(
            Some(Model::StudioPro),
            Some("0.2.5"),
            crate::models::Capabilities::MULTIPOINT_PEERS
        )
        .is_err());
    }
}
