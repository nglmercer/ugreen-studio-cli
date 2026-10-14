//! BlueZ MediaTransport observations; profile UUID controls interpretation.
//! Sources: BlueZ doc/org.bluez.MediaTransport.rst and profiles/audio/a2dp-codecs.h.
use super::{Activity, AudioStatus, Codec, Profile};
use crate::bluetooth::BluetoothAddress;
use std::time::SystemTime;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MediaTransport {
    pub path: String,
    pub address: BluetoothAddress,
    pub profile_uuid: Option<String>,
    pub codec_id: Option<u8>,
    pub configuration: Option<Vec<u8>>,
    pub state: Option<String>,
}

/// A2DP codec identifiers are profile-specific. 0xff alone never means LDAC.
/// Vendor configuration carries little-endian company and codec identifiers.
pub fn a2dp_codec(id: u8, configuration: &[u8]) -> Codec {
    match id {
        0x00 => Codec::Sbc,
        0x02 => Codec::Aac,
        0xff if configuration.len() >= 6 => {
            let company = u32::from_le_bytes(configuration[..4].try_into().expect("four bytes"));
            let codec = u16::from_le_bytes(configuration[4..6].try_into().expect("two bytes"));
            match (company, codec) {
                (0x0000012d, 0x00aa) => Codec::Ldac,
                (0x0000004f, 0x0001) => Codec::Aptx,
                (0x000000d7, 0x0024) => Codec::AptxHd,
                (0x0000000a, 0x0002) => Codec::AptxLl,
                _ => Codec::Unknown(format!("vendor:{company:08x}:{codec:04x}")),
            }
        }
        0xff => Codec::Unknown("vendor:missing-identifiers".into()),
        _ => Codec::Unknown(format!("a2dp:{id:02x}")),
    }
}
impl MediaTransport {
    pub fn observe(&self, observed_at: SystemTime) -> AudioStatus {
        let profile = match self
            .profile_uuid
            .as_deref()
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some(
                "0000110a-0000-1000-8000-00805f9b34fb" | "0000110b-0000-1000-8000-00805f9b34fb",
            ) => Profile::A2dp,
            Some(
                "0000111e-0000-1000-8000-00805f9b34fb" | "0000111f-0000-1000-8000-00805f9b34fb",
            ) => Profile::Hfp,
            Some(
                "00001108-0000-1000-8000-00805f9b34fb" | "00001112-0000-1000-8000-00805f9b34fb",
            ) => Profile::Hsp,
            _ => Profile::Unknown,
        };
        let codec = self.codec_id.map(|id| {
            if profile == Profile::A2dp {
                a2dp_codec(id, self.configuration.as_deref().unwrap_or_default())
            } else {
                Codec::Unknown(format!("profile-specific:{id:02x}"))
            }
        });
        let activity = match self.state.as_deref() {
            Some("active") => Activity::Active,
            Some("idle") => Activity::Idle,
            // Pending means not acquired; it does not establish an active OS route.
            _ => Activity::Unknown,
        };
        AudioStatus {
            address: self.address.clone(),
            profile,
            codec,
            activity,
            source: "bluez_media_transport",
            observed_at,
            transport_id: 0,
            transport_path: Some(self.path.clone()),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vendor_company_and_codec_are_both_required() {
        assert_eq!(a2dp_codec(0, &[]), Codec::Sbc);
        assert_eq!(a2dp_codec(2, &[]), Codec::Aac);
        assert_eq!(a2dp_codec(0xff, &[0x2d, 1, 0, 0, 0xaa, 0]), Codec::Ldac);
        assert!(matches!(
            a2dp_codec(0xff, &[0x2d, 1, 0, 0, 0xbb, 0]),
            Codec::Unknown(_)
        ));
        assert!(matches!(
            a2dp_codec(0xff, &[0x2d, 1, 0, 0, 0xaa]),
            Codec::Unknown(_)
        ));
        assert!(matches!(
            a2dp_codec(0xff, &[0x4f, 0, 0, 0, 0xaa, 0]),
            Codec::Unknown(_)
        ));
        assert_eq!(a2dp_codec(0xff, &[0x4f, 0, 0, 0, 1, 0]), Codec::Aptx);
    }
    #[test]
    fn profile_and_acquisition_control_interpretation() {
        let mut transport = MediaTransport {
            path: "/transport".into(),
            address: BluetoothAddress::parse("AA:BB:CC:DD:EE:FF").unwrap(),
            profile_uuid: None,
            codec_id: Some(0),
            configuration: None,
            state: Some("active".into()),
        };
        let observation = transport.observe(SystemTime::UNIX_EPOCH);
        assert_eq!(observation.profile, Profile::Unknown);
        assert!(matches!(observation.codec, Some(Codec::Unknown(_))));
        transport.profile_uuid = Some("0000110b-0000-1000-8000-00805f9b34fb".into());
        transport.state = Some("pending".into());
        let observation = transport.observe(SystemTime::UNIX_EPOCH);
        assert_eq!(observation.codec, Some(Codec::Sbc));
        assert_eq!(observation.activity, Activity::Unknown);
    }
}
