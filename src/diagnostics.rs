//! Read-only host diagnostics. Never opens RFCOMM, scans, pairs or powers adapters.
use crate::{
    audio::{AudioStatus, AudioStatusProvider, OsAudio},
    bluetooth::{self, BluetoothAddress, HostHeadset, HostSnapshot},
    models::Model,
};
use serde_json::{json, Value};
use std::io;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticFailure {
    pub kind: io::ErrorKind,
    pub message: String,
}
impl From<io::Error> for DiagnosticFailure {
    fn from(error: io::Error) -> Self {
        Self {
            kind: error.kind(),
            message: error.to_string(),
        }
    }
}
impl DiagnosticFailure {
    fn json(&self) -> Value {
        json!({"status":"unavailable","error_kind":format!("{:?}",self.kind),"reason":self.message})
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SocketCollision {
    NotRequested,
    NoObservedSocket,
    PossibleCollision { matching_sockets: usize },
    Unavailable(DiagnosticFailure),
}
#[derive(Clone, Debug)]
pub struct DoctorReport {
    pub address: Option<BluetoothAddress>,
    pub channel: u8,
    pub model: Option<Model>,
    pub host: Result<HostSnapshot, DiagnosticFailure>,
    pub devices: Vec<HostHeadset>,
    pub device_error: Option<DiagnosticFailure>,
    pub audio: Option<Result<Vec<AudioStatus>, DiagnosticFailure>>,
    pub collision: SocketCollision,
}
pub fn collect(
    address: Option<&BluetoothAddress>,
    channel: u8,
    model: Option<Model>,
) -> DoctorReport {
    let host = bluetooth::host_snapshot().map_err(DiagnosticFailure::from);
    let (devices, device_error) = match &host {
        Ok(snapshot) => (snapshot.devices.clone(), None),
        Err(_) => match bluetooth::list_devices() {
            Ok(v) => (v, None),
            Err(e) => (vec![], Some(e.into())),
        },
    };
    let audio = address.map(|a| OsAudio.observe(a).map_err(DiagnosticFailure::from));
    let collision = match address {
        None => SocketCollision::NotRequested,
        Some(address) => check_socket_collision(address, channel),
    };
    DoctorReport {
        address: address.cloned(),
        channel,
        model,
        host,
        devices,
        device_error,
        audio,
        collision,
    }
}
impl DoctorReport {
    pub fn json(&self) -> Value {
        self.json_with_session(None, false)
    }
    /// Host-only diagnostics cannot infer firmware. A TUI may add an observed
    /// firmware from its same-target active session without opening another socket.
    pub fn json_with_session(&self, firmware: Option<&str>, connected: bool) -> Value {
        let firmware = if connected { firmware } else { None };
        let write_capabilities = crate::models::capabilities_for(self.model, firmware).bits();
        let selected = self
            .address
            .as_ref()
            .and_then(|a| self.devices.iter().find(|d| &d.address == a));
        let adapters = match &self.host {
            Ok(snapshot) => {
                json!({"status":if snapshot.adapters.is_empty(){"no_controller"}else{"available"},"system_bus_read":"authorized","host_write_permissions":"not_probed","adapters":snapshot.adapters.iter().map(|a|json!({"path":a.path,"address":a.address.as_ref().map(|a|a.as_str()),"name":a.name,"powered":a.powered,"discovering":a.discovering})).collect::<Vec<_>>()})
            }
            Err(error) => error.json(),
        };
        let audio = self.audio.as_ref().map(|result| match result {
            Ok(v) => json!({"observations":v.iter().map(AudioStatus::json).collect::<Vec<_>>()}),
            Err(e) => e.json(),
        });
        let collision = match &self.collision {
            SocketCollision::NotRequested => json!({"status":"not_requested"}),
            SocketCollision::NoObservedSocket => {
                json!({"status":"no_socket_observed","source":"linux_debugfs_rfcomm","collision_excluded":false})
            }
            SocketCollision::PossibleCollision { matching_sockets } => {
                json!({"status":"possible_collision","source":"linux_debugfs_rfcomm","matching_sockets":matching_sockets})
            }
            SocketCollision::Unavailable(error) => error.json(),
        };
        json!({"schema_version":1,"kind":"doctor","host_enumeration_error":self.device_error.as_ref().map(|e|&e.message),"device_count":self.devices.len(),"selected_address":self.address.as_ref().map(|a|a.as_str()),"selected_paired":selected.map(|d|d.paired),"selected_host_link":selected.map(|d|d.connected.as_str()),"selected_model":self.model.map(Model::as_str),"rfcomm_channel":self.channel,"adapters":adapters,"audio":audio,"rfcomm_collision_check":collision,"firmware_evidence":if firmware.is_some(){"session_read"}else{"unknown_until_read"},"observed_firmware":firmware,"control_session":if connected{"connected"}else{"disconnected"},"write_capabilities":write_capabilities,"hardware_modified":false})
    }
}
fn check_socket_collision(address: &BluetoothAddress, channel: u8) -> SocketCollision {
    #[cfg(target_os = "linux")]
    {
        use std::io::Read;
        let result = (|| {
            let file = std::fs::File::open("/sys/kernel/debug/bluetooth/rfcomm")?;
            let mut text = String::new();
            file.take(1024 * 1024 + 1).read_to_string(&mut text)?;
            parse_socket_snapshot(&text, address, channel)
        })();
        match result {
            Ok(0) => SocketCollision::NoObservedSocket,
            Ok(matching_sockets) => SocketCollision::PossibleCollision { matching_sockets },
            Err(e) => SocketCollision::Unavailable(e.into()),
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (address, channel);
        SocketCollision::Unavailable(
            io::Error::new(
                io::ErrorKind::Unsupported,
                "read-only RFCOMM socket inventory unavailable on this platform",
            )
            .into(),
        )
    }
}
/// Linux rfcomm_sock_debugfs_show emits source, destination, state, channel.
/// No observed socket is not proof that the channel is free on the headset.
pub fn parse_socket_snapshot(
    text: &str,
    address: &BluetoothAddress,
    channel: u8,
) -> io::Result<usize> {
    if text.len() > 1024 * 1024 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "RFCOMM inventory exceeds 1 MiB",
        ));
    }
    let mut matching = 0;
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.len() != 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unrecognized RFCOMM inventory layout",
            ));
        }
        let _source = BluetoothAddress::parse(fields[0])?;
        let destination = BluetoothAddress::parse(fields[1])?;
        let _state: u8 = fields[2].parse().map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "invalid RFCOMM socket state")
        })?;
        let socket_channel: u8 = fields[3]
            .parse()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid RFCOMM channel"))?;
        if socket_channel > 30 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid RFCOMM channel",
            ));
        }
        if &destination == address && socket_channel == channel {
            matching += 1;
        }
    }
    Ok(matching)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn socket_inventory_matches_exact_destination_and_channel() {
        let address = BluetoothAddress::parse("AA:BB:CC:DD:EE:FF").unwrap();
        let text="00:11:22:33:44:55 AA:BB:CC:DD:EE:FF 1 1\n00:11:22:33:44:55 AA:BB:CC:DD:EE:FF 1 2\n00:11:22:33:44:55 11:22:33:44:55:66 1 1\n";
        assert_eq!(parse_socket_snapshot(text, &address, 1).unwrap(), 1);
        for text in [
            "invalid",
            "00:11:22:33:44:55 AA:BB:CC:DD:EE:FF x 1",
            "00:11:22:33:44:55 AA:BB:CC:DD:EE:FF 1 31",
        ] {
            assert!(parse_socket_snapshot(text, &address, 1).is_err());
        }
        assert!(parse_socket_snapshot(&" ".repeat(1024 * 1024 + 1), &address, 1).is_err());
    }
    #[test]
    fn no_controller_and_unknown_permissions_remain_explicit() {
        let report = DoctorReport {
            address: None,
            channel: 1,
            model: None,
            host: Ok(HostSnapshot::default()),
            devices: vec![],
            device_error: None,
            audio: None,
            collision: SocketCollision::NotRequested,
        };
        let json = report.json();
        assert_eq!(json["adapters"]["status"], "no_controller");
        assert_eq!(json["adapters"]["host_write_permissions"], "not_probed");
        assert_eq!(json["write_capabilities"], 0);
        assert_eq!(json["hardware_modified"], false);
    }
    #[test]
    fn session_firmware_context_does_not_change_host_only_diagnostics() {
        let report = DoctorReport {
            address: Some(BluetoothAddress::parse("AA:BB:CC:DD:EE:FF").unwrap()),
            channel: 1,
            model: Some(Model::StudioPro),
            host: Ok(HostSnapshot::default()),
            devices: vec![],
            device_error: None,
            audio: None,
            collision: SocketCollision::Unavailable(
                io::Error::from(io::ErrorKind::PermissionDenied).into(),
            ),
        };
        assert_eq!(report.json()["firmware_evidence"], "unknown_until_read");
        assert_eq!(
            report.json_with_session(Some("0.2.5"), false)["write_capabilities"],
            0
        );
        let active = report.json_with_session(Some("0.2.5"), true);
        assert_eq!(active["firmware_evidence"], "session_read");
        assert_eq!(active["observed_firmware"], "0.2.5");
        assert_eq!(
            active["write_capabilities"],
            Model::StudioPro.capabilities().bits()
        );
        assert_eq!(
            active["rfcomm_collision_check"]["error_kind"],
            "PermissionDenied"
        );
        assert_eq!(
            report.json_with_session(Some("0.2.6"), true)["write_capabilities"],
            0
        );
    }
}
