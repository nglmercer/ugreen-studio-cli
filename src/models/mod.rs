//! Model identification and per-model capability reporting.
//!
//! UGREEN reuses instruction IDs for different functions across models
//! (see `docs/protocol.md`), so a model is only ever explicitly chosen by
//! the user, or remembered in the registry from an earlier explicit
//! choice. It is never inferred from a device name, response bytes or
//! instruction numbers alone.

pub mod studio_pro;

/// Verified capability bits. A capability is set only when this project
/// has verified it on the named model/firmware; unverified features stay
/// unset so no control is ever rendered or exposed for them.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Capabilities(u32);

impl Capabilities {
    pub const ANC: u32 = 1 << 0;
    pub const EQ: u32 = 1 << 1;
    pub const GAME_MODE: u32 = 1 << 2;
    pub const SPATIAL: u32 = 1 << 3;
    /// Verified dual-connection toggle (`dual`, instruction `0x06`).
    pub const DUAL_DEVICE: u32 = 1 << 4;
    /// Peer list query/response: set only after Studio Pro captures.
    pub const MULTIPOINT_PEERS: u32 = 1 << 5;
    /// Peer disconnect write: set only after Studio Pro captures.
    pub const PEER_DISCONNECT: u32 = 1 << 6;
    /// Peer reconnect write: set only after Studio Pro captures.
    pub const PEER_RECONNECT: u32 = 1 << 7;
    /// Active-peer switch write: set only after Studio Pro captures.
    pub const PEER_SWITCH: u32 = 1 << 8;

    pub const fn new() -> Self {
        Self(0)
    }
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }
    pub const fn bits(self) -> u32 {
        self.0
    }
    pub const fn contains(self, flag: u32) -> bool {
        self.0 & flag == flag
    }
    pub const fn insert(&mut self, flag: u32) {
        self.0 |= flag;
    }
    /// Multipoint peer management verified end to end (read + write).
    pub const fn peer_management_verified(self) -> bool {
        self.contains(Self::MULTIPOINT_PEERS)
            && self.contains(Self::PEER_DISCONNECT)
            && self.contains(Self::PEER_RECONNECT)
            && self.contains(Self::PEER_SWITCH)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Model {
    StudioPro,
}

impl Model {
    pub const ALL: [Model; 1] = [Model::StudioPro];

    /// Accept only identifiers this project has verified mappings for.
    /// `HiTune Max5c` is deliberately not accepted: its instruction IDs
    /// collide with different Studio Pro functions.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "studio-pro" => Some(Self::StudioPro),
            _ => None,
        }
    }
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StudioPro => "studio-pro",
        }
    }
    pub const fn capabilities(self) -> Capabilities {
        match self {
            Self::StudioPro => studio_pro::capabilities(),
        }
    }
}

impl core::fmt::Display for Model {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_verified_models_parse() {
        assert_eq!(Model::parse("studio-pro"), Some(Model::StudioPro));
        for unsupported in ["max5c", "hitune-max5c", "MAX5C", "", "studio pro"] {
            assert_eq!(Model::parse(unsupported), None, "{unsupported}");
        }
    }

    #[test]
    fn studio_pro_reports_only_verified_capabilities() {
        let caps = Model::StudioPro.capabilities();
        assert!(caps.contains(Capabilities::ANC));
        assert!(caps.contains(Capabilities::EQ));
        assert!(caps.contains(Capabilities::GAME_MODE));
        assert!(caps.contains(Capabilities::SPATIAL));
        assert!(caps.contains(Capabilities::DUAL_DEVICE));
        assert!(!caps.contains(Capabilities::MULTIPOINT_PEERS));
        assert!(!caps.contains(Capabilities::PEER_DISCONNECT));
        assert!(!caps.contains(Capabilities::PEER_RECONNECT));
        assert!(!caps.contains(Capabilities::PEER_SWITCH));
        assert!(!caps.peer_management_verified());
    }
}
