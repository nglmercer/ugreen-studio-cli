//! Host-side headset representation: what the operating system knows about
//! a paired/connected Classic Bluetooth device. This is NOT a multipoint
//! peer and NOT the RFCOMM control session; see `crate::multipoint` and
//! `crate::bluetooth::Connection` for those separate concepts.

use crate::bluetooth::BluetoothAddress;
use crate::models::Model;

/// Whether the OS currently holds a link to the device.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostConnectionState {
    Disconnected,
    Connected,
    /// The platform could not report a definitive state.
    Unknown,
}

impl HostConnectionState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Connected => "connected",
            Self::Disconnected => "disconnected",
            Self::Unknown => "unknown",
        }
    }
    pub fn is_connected(self) -> bool {
        matches!(self, Self::Connected)
    }
}

impl core::fmt::Display for HostConnectionState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One headset known to the host OS, keyed by its platform-stable address.
///
/// `paired` and `connected` describe the OS Bluetooth link only. The
/// vendor RFCOMM control connection of this application is a third,
/// independent state tracked by the engine, never folded into this struct.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostHeadset {
    pub address: BluetoothAddress,
    pub name: String,
    /// The OS reports the device as paired/bonded.
    pub paired: bool,
    /// The OS link state at enumeration time.
    pub connected: HostConnectionState,
    /// Whether a vendor control channel is reachable. Enumeration never
    /// probes RFCOMM sockets, so this stays `None` until a control
    /// connection has actually been attempted in this process.
    pub control_available: Option<bool>,
    /// Remembered model. Enumeration never guesses a model from the
    /// device name; only the registry or an explicit user choice fills it.
    pub model: Option<Model>,
}

impl HostHeadset {
    /// A paired entry with unknown link state; the common listing case.
    pub fn paired(address: BluetoothAddress, name: impl Into<String>) -> Self {
        Self {
            address,
            name: name.into(),
            paired: true,
            connected: HostConnectionState::Unknown,
            control_available: None,
            model: None,
        }
    }
    pub fn with_connection(mut self, state: HostConnectionState) -> Self {
        self.connected = state;
        self
    }
    pub fn with_model(mut self, model: Model) -> Self {
        self.model = Some(model);
        self
    }
    pub fn is_reachable(&self) -> bool {
        self.paired || self.connected.is_connected()
    }
}

/// Deterministic picker order: the current target first, then connected,
/// disconnected and unknown devices; ties break by name, then address.
pub fn sort_devices(devices: &mut [HostHeadset], selected: Option<&BluetoothAddress>) {
    let rank = |device: &HostHeadset| -> u8 {
        if selected.is_some_and(|address| *address == device.address) {
            0
        } else {
            match device.connected {
                HostConnectionState::Connected => 1,
                HostConnectionState::Disconnected => 2,
                HostConnectionState::Unknown => 3,
            }
        }
    };
    devices.sort_by(|a, b| {
        rank(a)
            .cmp(&rank(b))
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            .then_with(|| a.address.cmp(&b.address))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paired_constructor_defaults_are_honest() {
        let headset = HostHeadset::paired(
            BluetoothAddress::parse("AA:BB:CC:DD:EE:FF").unwrap(),
            "Studio Pro",
        );
        assert!(headset.paired);
        assert_eq!(headset.connected, HostConnectionState::Unknown);
        assert!(!headset.connected.is_connected());
        assert_eq!(headset.control_available, None);
        assert_eq!(headset.model, None);
        assert!(headset.is_reachable());
    }

    #[test]
    fn unreachable_device_is_neither_paired_nor_connected() {
        let headset = HostHeadset {
            address: BluetoothAddress::parse("AA:BB:CC:DD:EE:FF").unwrap(),
            name: "Gone".into(),
            paired: false,
            connected: HostConnectionState::Disconnected,
            control_available: None,
            model: None,
        };
        assert!(!headset.is_reachable());
        assert_eq!(headset.connected.as_str(), "disconnected");
    }

    #[test]
    fn sort_devices_orders_target_then_link_state_then_name() {
        let unknown = HostHeadset::paired(
            BluetoothAddress::parse("00:00:00:00:00:03").unwrap(),
            "Unknown",
        );
        let disconnected = HostHeadset::paired(
            BluetoothAddress::parse("00:00:00:00:00:02").unwrap(),
            "Disconnected",
        )
        .with_connection(HostConnectionState::Disconnected);
        let connected = HostHeadset::paired(
            BluetoothAddress::parse("00:00:00:00:00:01").unwrap(),
            "Connected",
        )
        .with_connection(HostConnectionState::Connected);
        let selected = BluetoothAddress::parse("00:00:00:00:00:04").unwrap();
        let target = HostHeadset::paired(selected.clone(), "Target")
            .with_connection(HostConnectionState::Disconnected);

        // Unsorted input: target and disconnected start last.
        let mut devices = vec![
            unknown.clone(),
            disconnected.clone(),
            connected.clone(),
            target.clone(),
        ];
        sort_devices(&mut devices, Some(&selected));
        assert_eq!(devices[0].name, "Target");
        assert_eq!(devices[1].name, "Connected");
        assert_eq!(devices[2].name, "Disconnected");
        assert_eq!(devices[3].name, "Unknown");

        // Without a target the same link-state order holds, and equal
        // states fall back to name (then address) so order is stable.
        let mut devices = vec![unknown, disconnected, connected, target];
        sort_devices(&mut devices, None);
        assert_eq!(
            devices
                .iter()
                .map(|device| device.name.as_str())
                .collect::<Vec<_>>(),
            ["Connected", "Disconnected", "Target", "Unknown"]
        );
    }
}
