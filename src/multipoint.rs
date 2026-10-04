//! Headset multipoint: the verified dual-device toggle and the peer
//! list of phones/PCs attached to the headset itself (scaffolding).
//!
//! Multipoint peers live inside the headset. They are neither paired
//! host devices ([`crate::bluetooth::HostHeadset`]) nor this
//! application's own RFCOMM session; conflating them is the mistake
//! [`crate::bluetooth`] is documented against.
//!
//! Verification status on firmware 0.2.5:
//!
//! * **Dual toggle** — verified end to end: write instruction `0x06`,
//!   reported by device info at byte 5 (`settings::Setting::parse`
//!   key `dual`). Capability [`Capabilities::DUAL_DEVICE`].
//! * **Peer operations** — not verified. No Studio Pro capture of the
//!   official app proves the peer-list query or the
//!   disconnect/reconnect/switch writes, and a two-radio toggle proves
//!   nothing about a peer-management protocol. The capability bits
//!   ([`Capabilities::MULTIPOINT_PEERS`],
//!   [`Capabilities::PEER_DISCONNECT`], [`Capabilities::PEER_RECONNECT`],
//!   [`Capabilities::PEER_SWITCH`]) stay unset, every operation fails
//!   with [`UnsupportedFeature`] before any bytes are written, and no
//!   `peers` subcommand exists to reach them.

use crate::{bluetooth::BluetoothAddress, models::Capabilities, settings::StudioProState};
use std::{error::Error, fmt, io};

/// One multipoint peer slot of the headset.
///
/// The shape exists so a future verified capture has a place to land.
/// Nothing constructs it from unverified bytes: no Studio Pro frame has
/// been captured that encodes a peer identity yet.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultipointPeer {
    /// Peer address as the headset reports it once a verified parser
    /// exists.
    pub address: BluetoothAddress,
    /// Link state as the headset reports it.
    pub connected: bool,
}

/// Session view of multipoint state. Every field is tri-state: `None`
/// means "never queried", never "off" and never "no peers", so an
/// unknown value can never be displayed as a definite answer.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MultipointState {
    /// The dual-device radio toggle (instruction `0x06`). `None` when
    /// device info has not been read this session, or when the firmware
    /// answered with an out-of-range byte.
    pub dual: Option<bool>,
    /// Peer list. Stays `None` because the query itself is unverified;
    /// when a verified query exists, `Some(vec![])` will mean "queried,
    /// no peers" — a different answer from "never asked".
    pub peers: Option<Vec<MultipointPeer>>,
}

impl MultipointState {
    /// Dual toggle derived from an already-parsed device-info payload.
    /// The peer list is always left unknown: device info does not carry
    /// it and no verified query exists.
    pub fn from_device_info(info: &StudioProState) -> Self {
        Self {
            dual: info.value("dual").map(|value| value == "on"),
            peers: None,
        }
    }
}

/// Peer-management operations that exist as API surface but are not
/// verified on Studio Pro. An operation fails with this marker before
/// any frame is assembled, so unverified bytes are never written to a
/// live headset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnsupportedFeature {
    PeerList,
    PeerDisconnect,
    PeerReconnect,
    PeerSwitch,
}

impl UnsupportedFeature {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PeerList => "multipoint peer list",
            Self::PeerDisconnect => "multipoint peer disconnect",
            Self::PeerReconnect => "multipoint peer reconnect",
            Self::PeerSwitch => "multipoint peer switch",
        }
    }
    /// The capability bit that must be verified before the operation
    /// may send anything. All four bits are unset on every model today.
    pub const fn required_capability(self) -> u32 {
        match self {
            Self::PeerList => Capabilities::MULTIPOINT_PEERS,
            Self::PeerDisconnect => Capabilities::PEER_DISCONNECT,
            Self::PeerReconnect => Capabilities::PEER_RECONNECT,
            Self::PeerSwitch => Capabilities::PEER_SWITCH,
        }
    }
    /// Refuse the operation unless the model's capabilities verify it.
    /// With no Studio Pro captures, this fails for every variant.
    pub fn check(self, capabilities: Capabilities) -> Result<(), Self> {
        if capabilities.contains(self.required_capability()) {
            Ok(())
        } else {
            Err(self)
        }
    }
}

impl fmt::Display for UnsupportedFeature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} is not verified for Studio Pro", self.as_str())
    }
}

impl Error for UnsupportedFeature {}

impl From<UnsupportedFeature> for io::Error {
    fn from(feature: UnsupportedFeature) -> Self {
        io::Error::new(io::ErrorKind::Unsupported, feature.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info_with_dual(byte: u8) -> StudioProState {
        let mut raw = vec![0u8; 26];
        raw[5] = byte;
        StudioProState::new(raw).expect("26-byte payload passes")
    }

    #[test]
    fn dual_is_tri_state_and_peers_stay_unknown() {
        assert_eq!(
            MultipointState::from_device_info(&info_with_dual(1)),
            MultipointState {
                dual: Some(true),
                peers: None,
            }
        );
        assert_eq!(
            MultipointState::from_device_info(&info_with_dual(0)),
            MultipointState {
                dual: Some(false),
                peers: None,
            }
        );
        // Out-of-range bytes are unknown, never reported as off.
        assert_eq!(
            MultipointState::from_device_info(&info_with_dual(7)).dual,
            None
        );
        assert_eq!(MultipointState::default().dual, None);
        assert_eq!(MultipointState::default().peers, None);
    }

    #[test]
    fn peer_operations_fail_before_sending_anything() {
        let caps = crate::models::Model::StudioPro.capabilities();
        for feature in [
            UnsupportedFeature::PeerList,
            UnsupportedFeature::PeerDisconnect,
            UnsupportedFeature::PeerReconnect,
            UnsupportedFeature::PeerSwitch,
        ] {
            assert_eq!(feature.check(caps), Err(feature));
            let error: io::Error = feature.into();
            assert_eq!(error.kind(), io::ErrorKind::Unsupported);
            assert!(error.to_string().contains(feature.as_str()));
        }
    }

    #[test]
    fn capability_gates_match_operation_names() {
        assert_eq!(
            UnsupportedFeature::PeerList.required_capability(),
            Capabilities::MULTIPOINT_PEERS
        );
        assert_eq!(
            UnsupportedFeature::PeerDisconnect.required_capability(),
            Capabilities::PEER_DISCONNECT
        );
        assert_eq!(
            UnsupportedFeature::PeerReconnect.required_capability(),
            Capabilities::PEER_RECONNECT
        );
        assert_eq!(
            UnsupportedFeature::PeerSwitch.required_capability(),
            Capabilities::PEER_SWITCH
        );
        // Capability bits alone do not make peer management available.
        let mut unlocked = Capabilities::new();
        unlocked.insert(Capabilities::MULTIPOINT_PEERS);
        unlocked.insert(Capabilities::PEER_DISCONNECT);
        unlocked.insert(Capabilities::PEER_RECONNECT);
        unlocked.insert(Capabilities::PEER_SWITCH);
        assert_eq!(UnsupportedFeature::PeerList.check(unlocked), Ok(()));
        assert_eq!(UnsupportedFeature::PeerSwitch.check(unlocked), Ok(()));
    }

    #[test]
    fn dual_device_capability_does_not_imply_peer_management() {
        let caps = crate::models::Model::StudioPro.capabilities();
        assert!(caps.contains(Capabilities::DUAL_DEVICE));
        assert!(!caps.peer_management_verified());
    }
}
