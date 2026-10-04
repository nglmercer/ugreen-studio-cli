//! Host-side Bluetooth: device enumeration, identity types and the RFCOMM
//! control transport.
//!
//! Three layers exist and must never be conflated:
//!
//! 1. **OS Bluetooth connection** — pairing/link state owned by the OS
//!    (`HostHeadset`, `list_devices`).
//! 2. **RFCOMM vendor control connection** — this application's own
//!    socket to the headset's vendor service (`Connection`,
//!    `connect_control`).
//! 3. **Headset multipoint peers** — other phones/PCs connected to the
//!    headset itself (`crate::multipoint`).
//!
//! Enumeration only reads the OS's cached device list. It never scans,
//! pairs, powers the adapter, or opens RFCOMM sockets.

mod address;
mod device;

pub use address::BluetoothAddress;
pub use device::{HostConnectionState, HostHeadset};

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod platform;
#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod platform;
#[cfg(not(any(target_os = "linux", target_os = "windows")))]
#[path = "unsupported.rs"]
mod platform;

/// Minimal system-bus client used for BlueZ enumeration on Linux.
#[cfg(target_os = "linux")]
mod dbus;

use std::io::{self, Read, Write};
use std::time::{Duration, Instant};

/// Platform-independent access to host headsets and the control transport.
///
/// Platform implementations stay separate; vendor protocol behaviour never
/// leaks into this trait.
pub trait BluetoothBackend {
    /// List the OS's cached paired/connected headsets without discovery.
    fn list_devices(&self) -> io::Result<Vec<HostHeadset>>;
    /// Open this application's RFCOMM control channel to one address.
    fn connect_control(
        &self,
        address: &BluetoothAddress,
        channel: u8,
        timeout: Duration,
    ) -> io::Result<Connection>;
}

/// The production backend: native RFCOMM plus native enumeration.
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeBackend;

impl BluetoothBackend for NativeBackend {
    fn list_devices(&self) -> io::Result<Vec<HostHeadset>> {
        list_devices()
    }
    fn connect_control(
        &self,
        address: &BluetoothAddress,
        channel: u8,
        timeout: Duration,
    ) -> io::Result<Connection> {
        Connection::connect(address, channel, timeout)
    }
}

/// One live RFCOMM control socket to one headset.
///
/// Each read/write operation has a deadline. `read_exact` and `write_all`
/// share a single deadline across their partial operations. OS scheduling
/// can cause a deadline to be exceeded by a small amount.
#[derive(Debug)]
pub struct Connection {
    inner: platform::Connection,
    timeout: Duration,
}

impl Connection {
    /// Connect to an already-paired Classic Bluetooth device on channel 1–30.
    /// Does not scan, pair, or change adapter settings.
    pub fn connect(address: &BluetoothAddress, channel: u8, timeout: Duration) -> io::Result<Self> {
        if !(1..=30).contains(&channel) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "RFCOMM channel must be in 1..=30",
            ));
        }
        let deadline = deadline(timeout)?;
        Ok(Self {
            inner: platform::Connection::connect(address.octets(), channel, deadline)?,
            timeout,
        })
    }
    /// The address this control socket was opened to.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

impl Read for Connection {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf, deadline(self.timeout)?)
    }

    fn read_exact(&mut self, mut buf: &mut [u8]) -> io::Result<()> {
        let deadline = deadline(self.timeout)?;
        while !buf.is_empty() {
            let count = self.inner.read(buf, deadline)?;
            if count == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "RFCOMM peer closed before the complete response arrived",
                ));
            }
            buf = &mut buf[count..];
        }
        Ok(())
    }
}

impl Write for Connection {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf, deadline(self.timeout)?)
    }

    fn write_all(&mut self, mut buf: &[u8]) -> io::Result<()> {
        let deadline = deadline(self.timeout)?;
        while !buf.is_empty() {
            let count = self.inner.write(buf, deadline)?;
            if count == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "RFCOMM socket accepted no data",
                ));
            }
            buf = &buf[count..];
        }
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        // No userspace output buffering. This is not a remote acknowledgement.
        Ok(())
    }
}

/// Read only the OS's cached device list (paired and/or currently
/// connected). Never performs discovery, pairing, or connection attempts.
/// Results are sorted by address and deduplicated by address, keeping the
/// first occurrence so repeated listings are stable.
pub fn list_devices() -> io::Result<Vec<HostHeadset>> {
    let mut devices = platform::list_devices()?;
    normalize(&mut devices);
    Ok(devices)
}

/// Sort by address and keep exactly one entry per address. The first
/// occurrence wins; names never create identity.
pub fn normalize(devices: &mut Vec<HostHeadset>) {
    devices.sort_by(|a, b| a.address.cmp(&b.address));
    devices.dedup_by(|a, b| a.address == b.address);
}

fn deadline(timeout: Duration) -> io::Result<Instant> {
    if timeout.is_zero() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "timeout must be greater than zero",
        ));
    }
    Instant::now()
        .checked_add(timeout)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "timeout is too large"))
}

fn remaining(deadline: Instant) -> io::Result<Duration> {
    deadline
        .checked_duration_since(Instant::now())
        .filter(|d| !d.is_zero())
        .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "RFCOMM operation timed out"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headset(
        address: &str,
        name: &str,
        paired: bool,
        connected: HostConnectionState,
    ) -> HostHeadset {
        HostHeadset {
            address: BluetoothAddress::parse(address).unwrap(),
            name: name.into(),
            paired,
            connected,
            control_available: None,
            model: None,
        }
    }

    #[test]
    fn invalid_connect_arguments_never_reach_the_os() {
        let t = Duration::from_secs(1);
        for (address, channel, timeout) in [
            ("01:23:45:67:89:AB", 0, t),
            ("01:23:45:67:89:AB", 31, t),
            ("01:23:45:67:89:AB", 1, Duration::ZERO),
        ] {
            let address = BluetoothAddress::parse(address).unwrap();
            assert_eq!(
                Connection::connect(&address, channel, timeout)
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::InvalidInput
            );
        }
    }

    #[test]
    fn expired_deadline_is_timeout() {
        assert_eq!(
            remaining(Instant::now() - Duration::from_secs(1))
                .unwrap_err()
                .kind(),
            io::ErrorKind::TimedOut
        );
        assert!(remaining(deadline(Duration::from_secs(1)).unwrap()).is_ok());
    }

    #[test]
    fn normalize_sorts_and_dedups_by_address() {
        let mut devices = vec![
            headset(
                "BB:BB:BB:BB:BB:BB",
                "second",
                true,
                HostConnectionState::Connected,
            ),
            headset(
                "AA:AA:AA:AA:AA:AA",
                "old name",
                true,
                HostConnectionState::Unknown,
            ),
            headset(
                "AA:AA:AA:AA:AA:AA",
                "renamed",
                false,
                HostConnectionState::Connected,
            ),
        ];
        normalize(&mut devices);
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].address.as_str(), "AA:AA:AA:AA:AA:AA");
        // Same address with a different name keeps exactly one entry.
        assert_eq!(devices[0].name, "old name");
        assert_eq!(devices[1].name, "second");
    }
}
