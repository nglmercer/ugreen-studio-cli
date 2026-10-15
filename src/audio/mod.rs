//! Observed OS audio transport, independent of vendor preferences and RFCOMM.
pub mod bluez;
use crate::bluetooth::BluetoothAddress;
use serde_json::{json, Value};
use std::{
    io,
    time::{SystemTime, UNIX_EPOCH},
};

pub const MAX_DUMP_BYTES: usize = 1024 * 1024;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Profile {
    A2dp,
    Hfp,
    Hsp,
    LeAudio,
    Unknown,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Codec {
    Sbc,
    SbcXq,
    Aac,
    Aptx,
    AptxHd,
    AptxLl,
    AptxAdaptive,
    Ldac,
    Lc3,
    Unknown(String),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Activity {
    Active,
    Idle,
    Unknown,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioStatus {
    pub address: BluetoothAddress,
    pub profile: Profile,
    pub codec: Option<Codec>,
    pub activity: Activity,
    pub source: &'static str,
    pub observed_at: SystemTime,
    pub transport_id: u64,
    pub transport_path: Option<String>,
}
pub trait AudioStatusProvider {
    fn observe(&self, address: &BluetoothAddress) -> io::Result<Vec<AudioStatus>>;
}
pub struct OsAudio;
impl AudioStatusProvider for OsAudio {
    fn observe(&self, address: &BluetoothAddress) -> io::Result<Vec<AudioStatus>> {
        #[cfg(target_os = "linux")]
        {
            let mut command = std::process::Command::new("pw-dump");
            command.arg("--no-colors");
            let pipewire =
                crate::process::collect_output(command, std::time::Duration::from_secs(3))
                    .and_then(|dump| parse_pipewire(dump.as_bytes(), address, SystemTime::now()));
            if pipewire
                .as_ref()
                .is_ok_and(|observations| observations.iter().any(|o| o.codec.is_some()))
            {
                return pipewire;
            }
            let bluez = crate::bluetooth::host_snapshot().map(|snapshot| {
                snapshot
                    .transports
                    .into_iter()
                    .filter(|t| &t.address == address)
                    .map(|t| t.observe(SystemTime::now()))
                    .collect::<Vec<_>>()
            });
            match (pipewire,bluez) {
                (Ok(mut nodes),Ok(transports)) => { nodes.extend(transports); Ok(nodes) }
                (Ok(nodes),Err(_)) => Ok(nodes),
                (Err(_),Ok(transports)) => Ok(transports),
                (Err(pipewire),Err(bluez)) => Err(io::Error::other(format!("PipeWire unavailable: {pipewire}; BlueZ transport fallback unavailable: {bluez}"))),
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = address;
            Err(io::Error::new(io::ErrorKind::Unsupported, "negotiated codec observation unavailable on this platform; inspect the OS audio diagnostics"))
        }
    }
}
/// Only address-matched BlueZ nodes qualify. A support list never determines a codec.
/// Conflicting observations remain separate rather than selecting an arbitrary node.
pub fn parse_pipewire(
    bytes: &[u8],
    address: &BluetoothAddress,
    observed_at: SystemTime,
) -> io::Result<Vec<AudioStatus>> {
    if bytes.len() > MAX_DUMP_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "PipeWire dump exceeds 1 MiB",
        ));
    }
    let root: Value =
        serde_json::from_slice(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let entries = root.as_array().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "PipeWire dump must be an array")
    })?;
    let mut result = Vec::new();
    for entry in entries {
        if entry["type"].as_str() != Some("PipeWire:Interface:Node") {
            continue;
        }
        let props = &entry["info"]["props"];
        let Some(raw_address) = props["api.bluez5.address"].as_str() else {
            continue;
        };
        if BluetoothAddress::parse(raw_address).ok().as_ref() != Some(address) {
            continue;
        }
        let profile = match props["api.bluez5.profile"].as_str() {
            Some("a2dp-sink" | "a2dp-source") => Profile::A2dp,
            Some("headset-head-unit" | "headset-audio-gateway" | "hfp-hf" | "hfp-ag") => {
                Profile::Hfp
            }
            Some("hsp-hs" | "hsp-ag") => Profile::Hsp,
            Some("bap-sink" | "bap-source") => Profile::LeAudio,
            _ => Profile::Unknown,
        };
        let activity = match entry["info"]["state"].as_str() {
            Some("running") => Activity::Active,
            Some("idle" | "suspended") => Activity::Idle,
            _ => Activity::Unknown,
        };
        // A node must carry its transport key and codec, not a support list.
        // PipeWire publishes an internal pointer here; its native protocol
        // redacts pointer values to "". An empty string is therefore valid
        // transport metadata, but must never be presented as a D-Bus path.
        // See PipeWire bluez5-device.c emit_node and protocol-native.c push_item.
        let codec = if props["api.bluez5.transport"].as_str().is_some() {
            props["api.bluez5.codec"].as_str().map(|name| match name {
                "sbc" => Codec::Sbc,
                "sbc_xq" => Codec::SbcXq,
                "aac" => Codec::Aac,
                "aptx" => Codec::Aptx,
                "aptx_hd" => Codec::AptxHd,
                "aptx_ll" => Codec::AptxLl,
                "aptx_adaptive" => Codec::AptxAdaptive,
                "ldac" => Codec::Ldac,
                "lc3" => Codec::Lc3,
                _ => Codec::Unknown(name.to_owned()),
            })
        } else {
            None
        };
        result.push(AudioStatus {
            address: address.clone(),
            profile,
            codec,
            activity,
            source: "pipewire_node_transport",
            observed_at,
            transport_id: entry["id"].as_u64().unwrap_or(0),
            transport_path: props["api.bluez5.transport"]
                .as_str()
                .filter(|path| path.starts_with("/org/bluez/"))
                .map(str::to_owned),
        });
    }
    result.sort_by_key(|status| status.transport_id);
    Ok(result)
}
impl AudioStatus {
    pub fn json(&self) -> Value {
        json!({"address": self.address.as_str(), "profile": format!("{:?}", self.profile),
            "codec": self.codec.as_ref().map(|v| format!("{v:?}")), "activity": format!("{:?}", self.activity),
            "source": self.source, "evidence": "os_observation", "transport_id": self.transport_id, "transport_path":self.transport_path,
            "observed_at_unix_ms": self.observed_at.duration_since(UNIX_EPOCH).ok().map(|v| v.as_millis() as u64)})
    }
}
/// Enumerate exact PipeWire sink names for explicit routing; no default fallback.
#[cfg(target_os = "linux")]
pub fn output_devices() -> io::Result<Vec<(String, String)>> {
    let mut command = std::process::Command::new("pw-dump");
    command.arg("--no-colors");
    let dump = crate::process::collect_output(command, std::time::Duration::from_secs(3))?;
    parse_output_devices(dump.as_bytes())
}
pub fn parse_output_devices(bytes: &[u8]) -> io::Result<Vec<(String, String)>> {
    if bytes.len() > MAX_DUMP_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "PipeWire dump exceeds 1 MiB",
        ));
    }
    let root: Value =
        serde_json::from_slice(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let nodes = root.as_array().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "PipeWire dump must be an array")
    })?;
    let mut outputs = Vec::new();
    for node in nodes {
        let props = &node["info"]["props"];
        if node["type"] != "PipeWire:Interface:Node" || props["media.class"] != "Audio/Sink" {
            continue;
        }
        let Some(name) = props["node.name"].as_str().filter(|name| !name.is_empty()) else {
            continue;
        };
        outputs.push((
            name.to_owned(),
            props["node.description"]
                .as_str()
                .unwrap_or(name)
                .to_owned(),
        ));
    }
    outputs.sort();
    outputs.dedup();
    Ok(outputs)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn address() -> BluetoothAddress {
        BluetoothAddress::parse("AA:BB:CC:DD:EE:FF").unwrap()
    }
    fn node(codec: &str) -> Value {
        json!({"id": 1, "type": "PipeWire:Interface:Node", "info": {"state": "running", "props": {"api.bluez5.address": address().as_str(), "api.bluez5.profile": "a2dp-sink", "api.bluez5.codec": codec, "api.bluez5.transport": "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF/sep1/fd0"}}})
    }
    #[test]
    fn codecs_are_observed_only_for_exact_address_and_transport() {
        for (name, expected) in [
            ("sbc", Codec::Sbc),
            ("aac", Codec::Aac),
            ("ldac", Codec::Ldac),
            ("0xff", Codec::Unknown("0xff".into())),
        ] {
            let mut other = node("aac");
            other["info"]["props"]["api.bluez5.address"] = json!("00:11:22:33:44:55");
            let dump = serde_json::to_vec(&json!([other, node(name)])).unwrap();
            let result = parse_pipewire(&dump, &address(), UNIX_EPOCH).unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].codec, Some(expected));
            assert_eq!(result[0].activity, Activity::Active);
        }
    }
    #[test]
    fn absent_transport_never_turns_preference_into_negotiation() {
        let mut n = node("ldac");
        n["info"]["props"]
            .as_object_mut()
            .unwrap()
            .remove("api.bluez5.transport");
        let dump = serde_json::to_vec(&json!([n])).unwrap();
        assert_eq!(
            parse_pipewire(&dump, &address(), UNIX_EPOCH).unwrap()[0].codec,
            None
        );
    }
    #[test]
    fn redacted_pipewire_pointer_preserves_codec_without_inventing_a_transport_path() {
        for value in ["", "pointer:0x1234"] {
            let mut n = node("sbc_xq");
            n["info"]["props"]["api.bluez5.transport"] = json!(value);
            let result = parse_pipewire(
                &serde_json::to_vec(&json!([n])).unwrap(),
                &address(),
                UNIX_EPOCH,
            )
            .unwrap();
            assert_eq!(result[0].codec, Some(Codec::SbcXq));
            assert!(result[0].transport_path.is_none());
        }
    }
    #[test]
    fn malformed_oversized_missing_and_conflicting_data() {
        for bytes in [
            b"{".to_vec(),
            b"{}".to_vec(),
            vec![b' '; MAX_DUMP_BYTES + 1],
        ] {
            assert!(parse_pipewire(&bytes, &address(), UNIX_EPOCH).is_err());
        }
        assert!(parse_pipewire(b"[{}]", &address(), UNIX_EPOCH)
            .unwrap()
            .is_empty());
        let dump = serde_json::to_vec(&json!([node("sbc"), node("aac")])).unwrap();
        assert_eq!(
            parse_pipewire(&dump, &address(), UNIX_EPOCH).unwrap().len(),
            2
        );
    }
}
