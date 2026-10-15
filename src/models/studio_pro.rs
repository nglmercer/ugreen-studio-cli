//! UGREEN Studio Pro (HP206) capability set.
//!
//! Verified on firmware 0.2.5 against the settings table in
//! `docs/protocol.md`: ANC, EQ, game mode, spatial audio,
//! dual-device toggle, wind reduction, prompt mode,
//! volume-hold actions, battery and firmware readback.
//!
//! Multipoint peer operations (`MULTIPOINT_PEERS`,
//! `PEER_DISCONNECT`, `PEER_RECONNECT`, `PEER_SWITCH`) remain
//! unset until official-app Studio Pro traffic proves the packet
//! formats. Dual-device capability only proves a two-radio
//! toggle, not a peer-management protocol. Every other advanced
//! feature bit stays unset pending the same capture evidence.

use super::Capabilities;

/// Exact firmware scope of the historical verified setting evidence.
pub const VERIFIED_FIRMWARE: &str = "0.2.5";

/// Verified Studio Pro capabilities (HP206, firmware 0.2.5):
/// the nine verified settings and nothing else.
pub const fn capabilities() -> Capabilities {
    let mut caps = Capabilities::new();
    caps.insert(Capabilities::ANC);
    caps.insert(Capabilities::EQ);
    caps.insert(Capabilities::GAME_MODE);
    caps.insert(Capabilities::SPATIAL);
    caps.insert(Capabilities::DUAL_DEVICE);
    caps.insert(Capabilities::WIND_REDUCTION);
    caps.insert(Capabilities::PROMPTS);
    caps.insert(Capabilities::VOLUME_UP_ACTION);
    caps.insert(Capabilities::VOLUME_DOWN_ACTION);
    caps
}
