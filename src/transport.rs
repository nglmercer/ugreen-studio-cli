//! Native Bluetooth Classic RFCOMM transport, with no background discovery.
//!
//! Each connect/read/write operation has a deadline. `read_exact` and
//! `write_all` also share a single deadline across their partial operations.
//! OS scheduling can cause a deadline to be exceeded by a small amount.

use std::io::{self, Read, Write};
use std::time::{Duration, Instant};

#[cfg(target_os = "linux")]
#[path = "transport/linux.rs"]
mod platform;
#[cfg(target_os = "windows")]
#[path = "transport/windows.rs"]
mod platform;
#[cfg(not(any(target_os = "linux", target_os = "windows")))]
#[path = "transport/unsupported.rs"]
mod platform;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Device {
    pub address: String,
    pub name: String,
}

#[derive(Debug)]
pub struct Connection {
    inner: platform::Connection,
    timeout: Duration,
}

impl Connection {
    /// Connect to an already-paired Classic Bluetooth device on channel 1–30.
    /// Accepts exactly six colon-separated hexadecimal address octets.
    /// Does not scan, pair, or change adapter settings.
    pub fn connect(address: &str, channel: u8, timeout: Duration) -> io::Result<Self> {
        let address = parse_address(address)?;
        if !(1..=30).contains(&channel) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "RFCOMM channel must be in 1..=30",
            ));
        }
        let deadline = deadline(timeout)?;
        Ok(Self {
            inner: platform::Connection::connect(address, channel, deadline)?,
            timeout,
        })
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

/// Read only the OS's cached paired-device list. Never performs discovery,
/// pairing, or connection attempts. Linux requires BlueZ's `bluetoothctl`.
pub fn list_paired() -> io::Result<Vec<Device>> {
    let mut devices = platform::list_paired()?;
    devices.sort_by(|a, b| a.address.cmp(&b.address));
    devices.dedup_by(|a, b| a.address == b.address);
    Ok(devices)
}

fn parse_address(address: &str) -> io::Result<[u8; 6]> {
    let invalid = || {
        io::Error::new(io::ErrorKind::InvalidInput,
        "Bluetooth address must contain exactly six colon-separated hex octets, e.g. AA:BB:CC:DD:EE:FF")
    };
    let bytes = address.as_bytes();
    if bytes.len() != 17 {
        return Err(invalid());
    }
    let mut result = [0; 6];
    for (index, octet) in result.iter_mut().enumerate() {
        let offset = index * 3;
        if index != 5 && bytes[offset + 2] != b':' {
            return Err(invalid());
        }
        let high = hex_digit(bytes[offset]).ok_or_else(invalid)?;
        let low = hex_digit(bytes[offset + 1]).ok_or_else(invalid)?;
        *octet = high * 16 + low;
    }
    Ok(result)
}

fn hex_digit(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn format_address(address: [u8; 6]) -> String {
    format!(
        "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        address[0], address[1], address[2], address[3], address[4], address[5]
    )
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

    #[test]
    fn address_is_strict_and_case_insensitive() {
        let bytes = [0x01, 0x23, 0xab, 0xcd, 0xef, 0x45];
        assert_eq!(parse_address("01:23:aB:cD:eF:45").unwrap(), bytes);
        assert_eq!(format_address(bytes), "01:23:AB:CD:EF:45");
        for invalid in [
            "",
            "1:23:AB:CD:EF:45",
            "01-23-AB-CD-EF-45",
            "01:23:AB:CD:EF:4G",
            " 01:23:AB:CD:EF:45",
            "01:23:AB:CD:EF:45\n",
            "01:23:AB:CD:EF:45:67",
            "é1:23:AB:CD:EF:4",
        ] {
            assert_eq!(
                parse_address(invalid).unwrap_err().kind(),
                io::ErrorKind::InvalidInput
            );
        }
    }

    #[test]
    fn invalid_connect_arguments_never_reach_the_os() {
        let t = Duration::from_secs(1);
        for (address, channel, timeout) in [
            ("bad", 1, t),
            ("01:23:45:67:89:AB", 0, t),
            ("01:23:45:67:89:AB", 31, t),
            ("01:23:45:67:89:AB", 1, Duration::ZERO),
        ] {
            assert_eq!(
                Connection::connect(address, channel, timeout)
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
}
