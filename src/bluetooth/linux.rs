//! Linux ABI references: BlueZ lib/bluetooth/rfcomm.h and Linux asm-generic
//! socket/fcntl headers. No libbluetooth linkage is needed for RFCOMM sockets.
//! <https://github.com/bluez/bluez/blob/master/lib/bluetooth/rfcomm.h>
//! <https://github.com/bluez/bluez/blob/master/doc/bluetoothctl.rst>

use super::{remaining, BluetoothAddress, HostConnectionState, HostHeadset};
use crate::process::collect_output;
use std::ffi::{c_int, c_short, c_ulong, c_void};
use std::io;
use std::mem::size_of;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::process::Command;
use std::time::{Duration, Instant};

const AF_BLUETOOTH: c_int = 31;
const SOCK_STREAM: c_int = 1;
const SOCK_NONBLOCK: c_int = 0x800;
const SOCK_CLOEXEC: c_int = 0x80000;
const BTPROTO_RFCOMM: c_int = 3;
const SOL_SOCKET: c_int = 1;
const SO_ERROR: c_int = 4;
const EINPROGRESS: c_int = 115;
const MSG_NOSIGNAL: c_int = 0x4000;
const POLLIN: c_short = 1;
const POLLOUT: c_short = 4;
const POLLNVAL: c_short = 0x20;

#[repr(C)]
struct SockAddrRc {
    family: u16,
    address: [u8; 6],
    channel: u8,
    // Explicit padding keeps every byte passed to connect initialized.
    padding: u8,
}

#[repr(C)]
struct PollFd {
    fd: c_int,
    events: c_short,
    revents: c_short,
}

unsafe extern "C" {
    fn socket(domain: c_int, kind: c_int, protocol: c_int) -> c_int;
    fn connect(fd: c_int, address: *const c_void, length: u32) -> c_int;
    fn getsockopt(
        fd: c_int,
        level: c_int,
        option: c_int,
        value: *mut c_void,
        length: *mut u32,
    ) -> c_int;
    fn poll(fds: *mut PollFd, count: c_ulong, timeout_ms: c_int) -> c_int;
    fn recv(fd: c_int, buffer: *mut c_void, length: usize, flags: c_int) -> isize;
    fn send(fd: c_int, buffer: *const c_void, length: usize, flags: c_int) -> isize;
}

// These Linux architectures share the constants above. Do not silently apply
// the asm-generic ABI to MIPS/SPARC, whose socket/errno constants differ.
fn check_abi() -> io::Result<()> {
    if cfg!(any(
        target_arch = "x86",
        target_arch = "x86_64",
        target_arch = "arm",
        target_arch = "aarch64",
        target_arch = "riscv32",
        target_arch = "riscv64",
        target_arch = "powerpc",
        target_arch = "powerpc64",
        target_arch = "s390x",
        target_arch = "loongarch64"
    )) {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "this Linux CPU architecture is not yet supported by the native RFCOMM bindings",
        ))
    }
}

#[derive(Debug)]
pub(super) struct Connection {
    fd: OwnedFd,
}

impl Connection {
    pub(super) fn connect(
        mut address: [u8; 6],
        channel: u8,
        deadline: Instant,
    ) -> io::Result<Self> {
        check_abi()?;
        remaining(deadline)?;
        // SAFETY: socket takes integer arguments and returns a fresh owned fd.
        let raw = unsafe {
            socket(
                AF_BLUETOOTH,
                SOCK_STREAM | SOCK_NONBLOCK | SOCK_CLOEXEC,
                BTPROTO_RFCOMM,
            )
        };
        if raw < 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: raw is a newly created, valid descriptor owned only here.
        let connection = Self {
            fd: unsafe { OwnedFd::from_raw_fd(raw) },
        };
        address.reverse(); // BlueZ bdaddr_t stores the least significant octet first.
        let target = SockAddrRc {
            family: AF_BLUETOOTH as u16,
            address,
            channel,
            padding: 0,
        };
        // SAFETY: target has the sockaddr_rc C layout and lives through the call.
        let result = unsafe {
            connect(
                raw,
                (&target as *const SockAddrRc).cast(),
                size_of::<SockAddrRc>() as u32,
            )
        };
        if result < 0 {
            let error = io::Error::last_os_error();
            if error.raw_os_error() != Some(EINPROGRESS)
                && error.kind() != io::ErrorKind::WouldBlock
                && error.kind() != io::ErrorKind::Interrupted
            {
                return Err(error);
            }
            wait_ready(raw, POLLOUT, deadline)?;
            socket_error(raw)?;
        }
        Ok(connection)
    }

    pub(super) fn read(&mut self, buf: &mut [u8], deadline: Instant) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        loop {
            remaining(deadline)?;
            // SAFETY: the fd is live; buf is writable for its exact length.
            let result =
                unsafe { recv(self.fd.as_raw_fd(), buf.as_mut_ptr().cast(), buf.len(), 0) };
            if result >= 0 {
                return Ok(result as usize);
            }
            let error = io::Error::last_os_error();
            match error.kind() {
                io::ErrorKind::Interrupted => continue,
                io::ErrorKind::WouldBlock => wait_ready(self.fd.as_raw_fd(), POLLIN, deadline)?,
                _ => return Err(error),
            }
        }
    }

    pub(super) fn write(&mut self, buf: &[u8], deadline: Instant) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        loop {
            remaining(deadline)?;
            // SAFETY: the fd is live; buf is readable for its exact length.
            // MSG_NOSIGNAL avoids terminating the process on a disconnected peer.
            let result = unsafe {
                send(
                    self.fd.as_raw_fd(),
                    buf.as_ptr().cast(),
                    buf.len(),
                    MSG_NOSIGNAL,
                )
            };
            if result >= 0 {
                return Ok(result as usize);
            }
            let error = io::Error::last_os_error();
            match error.kind() {
                io::ErrorKind::Interrupted => continue,
                io::ErrorKind::WouldBlock => wait_ready(self.fd.as_raw_fd(), POLLOUT, deadline)?,
                _ => return Err(error),
            }
        }
    }
}

fn wait_ready(fd: RawFd, events: c_short, deadline: Instant) -> io::Result<()> {
    loop {
        let duration = remaining(deadline)?;
        // Round up to avoid a zero-millisecond busy loop for short timeouts.
        let millis = duration
            .as_millis()
            .saturating_add(u128::from(duration.subsec_nanos() % 1_000_000 != 0))
            .min(c_int::MAX as u128) as c_int;
        let mut descriptor = PollFd {
            fd,
            events,
            revents: 0,
        };
        // SAFETY: descriptor is initialized, writable, and describes one live fd.
        let result = unsafe { poll(&mut descriptor, 1, millis) };
        if result > 0 {
            if descriptor.revents & POLLNVAL != 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "invalid RFCOMM socket descriptor",
                ));
            }
            // POLLHUP/POLLERR also wake the caller, which retrieves EOF or the
            // concrete socket error with recv/send/getsockopt.
            return Ok(());
        }
        if result < 0 {
            let error = io::Error::last_os_error();
            if error.kind() != io::ErrorKind::Interrupted {
                return Err(error);
            }
        }
        // A capped poll can expire before a very long requested deadline.
    }
}

fn socket_error(fd: RawFd) -> io::Result<()> {
    let mut value: c_int = 0;
    let mut length = size_of::<c_int>() as u32;
    // SAFETY: value/length point to correctly sized writable initialized values.
    let result = unsafe {
        getsockopt(
            fd,
            SOL_SOCKET,
            SO_ERROR,
            (&mut value as *mut c_int).cast(),
            &mut length,
        )
    };
    if result < 0 {
        return Err(io::Error::last_os_error());
    }
    if value != 0 {
        return Err(io::Error::from_raw_os_error(value));
    }
    Ok(())
}

/// List the OS's cached paired and connected devices.
///
/// The BlueZ D-Bus API is preferred because it returns typed properties
/// (Paired/Connected) in one round trip. If the system bus is
/// unavailable, a bounded `bluetoothctl` fallback runs two read-only
/// cache queries. Neither path ever runs `scan`, `pair`, `connect`,
/// `agent`, or `power` commands, and no shell is involved.
pub(super) fn list_devices() -> io::Result<Vec<HostHeadset>> {
    check_abi()?;
    match super::dbus::list_devices() {
        Ok(devices) => Ok(devices),
        Err(_) => bluetoothctl_devices(),
    }
}

fn bluetoothctl_devices() -> io::Result<Vec<HostHeadset>> {
    let paired = bluetoothctl_query("Paired")?;
    // Older bluetoothctl builds lack the `Connected` filter. Then the
    // link state stays Unknown instead of failing the whole listing.
    let connected = bluetoothctl_query("Connected").unwrap_or_default();
    Ok(merge_paired_connected(paired, connected))
}

/// Pure merge of the two cached listings. The paired listing supplies
/// names and pairing truth; the connected listing only adds link state.
fn merge_paired_connected(
    paired: Vec<(BluetoothAddress, String)>,
    connected: Vec<(BluetoothAddress, String)>,
) -> Vec<HostHeadset> {
    let mut devices: Vec<HostHeadset> = Vec::new();
    for (address, name) in paired {
        let state = if connected.iter().any(|(a, _)| *a == address) {
            HostConnectionState::Connected
        } else {
            HostConnectionState::Disconnected
        };
        devices.push(HostHeadset::paired(address, name).with_connection(state));
    }
    for (address, name) in connected {
        if !devices.iter().any(|d| d.address == address) {
            // Connected but not reported as paired: never claim pairing.
            devices.push(HostHeadset {
                address,
                name,
                paired: false,
                connected: HostConnectionState::Connected,
                control_available: None,
                model: None,
            });
        }
    }
    devices
}

fn bluetoothctl_query(filter: &str) -> io::Result<Vec<(BluetoothAddress, String)>> {
    // This command only reads BlueZ's device cache; the filter selects
    // which cached entries are printed. In particular it never runs
    // `scan`, `pair`, `connect`, `agent`, or `power` commands.
    let mut command = Command::new("bluetoothctl");
    command
        .args(["--timeout", "5", "devices", filter])
        .env("LC_ALL", "C");
    let output = collect_output(command, Duration::from_secs(6))?;
    parse_device_lines(&output)
}

fn strip_ansi(line: &str) -> String {
    let mut result = String::new();
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            for code in chars.by_ref() {
                if ('@'..='~').contains(&code) {
                    break;
                }
            }
        } else {
            result.push(ch);
        }
    }
    result
}

/// Strict parser for `bluetoothctl devices <filter>` output. Locale is
/// pinned to `C`, ANSI escapes are stripped, and any line that is not a
/// device record is an error rather than silently ignored.
fn parse_device_lines(output: &str) -> io::Result<Vec<(BluetoothAddress, String)>> {
    let mut devices = Vec::new();
    for line in output.lines() {
        let clean = strip_ansi(line);
        let line = clean.trim();
        if line.is_empty() {
            continue;
        }
        if line.contains("No default controller")
            || line.contains("Waiting to connect to bluetoothd")
        {
            return Err(io::Error::new(io::ErrorKind::NotConnected,
                format!("BlueZ is unavailable: {line}; check the Bluetooth adapter and bluetooth service")));
        }
        let rest = line.strip_prefix("Device ").ok_or_else(|| io::Error::other(format!(
            "unexpected bluetoothctl output: {line}; this command requires BlueZ support for 'devices'")))?;
        let (address, name) = rest.split_once(' ').unwrap_or((rest, ""));
        let address = BluetoothAddress::parse(address).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("bluetoothctl returned an invalid device address: {address}"),
            )
        })?;
        devices.push((address, name.trim().to_owned()));
    }
    Ok(devices)
}

/// Reset cleanup uses a typed BlueZ RemoveDevice call; no shell fallback.
pub(super) fn unpair(address: &super::BluetoothAddress) -> io::Result<()> {
    super::dbus::unpair(address)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;

    fn local_pair() -> (Connection, UnixStream) {
        let (local, peer) = UnixStream::pair().unwrap();
        local.set_nonblocking(true).unwrap();
        (Connection { fd: local.into() }, peer)
    }

    #[test]
    fn address_layout_matches_bluez() {
        assert_eq!(size_of::<SockAddrRc>(), 10);
        assert_eq!(std::mem::offset_of!(SockAddrRc, address), 2);
        assert_eq!(std::mem::offset_of!(SockAddrRc, channel), 8);
        assert_eq!(size_of::<PollFd>(), 8);
    }

    #[test]
    fn cached_device_output_is_parsed_without_discovery() {
        let devices = parse_device_lines("\u{1b}[0;94mDevice\u{1b}[0m aa:bb:cc:dd:ee:ff UGREEN Studio Pro\nDevice 01:23:45:67:89:AB Headphones with spaces\n").unwrap();
        assert_eq!(devices[0].0.as_str(), "AA:BB:CC:DD:EE:FF");
        assert_eq!(devices[0].1, "UGREEN Studio Pro");
        assert_eq!(devices[1].1, "Headphones with spaces");
        assert!(parse_device_lines("").unwrap().is_empty());
        assert!(parse_device_lines("Device invalid Name").is_err());
        assert!(parse_device_lines("Invalid command in menu main: devices").is_err());
        assert!(parse_device_lines("No default controller available").is_err());
    }

    #[test]
    fn paired_and_connected_lists_merge_without_faking_pairing() {
        let address = |a: &str| BluetoothAddress::parse(a).unwrap();
        let paired = vec![
            (address("AA:AA:AA:AA:AA:01"), "Studio Pro".to_string()),
            (address("AA:AA:AA:AA:AA:02"), "Old name".to_string()),
        ];
        let connected = vec![
            (address("AA:AA:AA:AA:AA:02"), "Renamed".to_string()),
            (address("AA:AA:AA:AA:AA:03"), "Unpaired device".to_string()),
        ];
        let devices = merge_paired_connected(paired, connected);
        assert_eq!(devices.len(), 3);
        assert!(devices[0].paired);
        assert_eq!(devices[0].connected, HostConnectionState::Disconnected);
        assert!(devices[1].paired);
        assert_eq!(devices[1].connected, HostConnectionState::Connected);
        // The paired listing's name wins; identity is the address.
        assert_eq!(devices[1].name, "Old name");
        assert!(!devices[2].paired);
        assert_eq!(devices[2].connected, HostConnectionState::Connected);
    }

    #[test]
    fn local_stream_io_and_eof_need_no_bluetooth_hardware() {
        let (mut connection, mut peer) = local_pair();
        let deadline = Instant::now() + Duration::from_secs(1);
        peer.write_all(b"reply").unwrap();
        let mut buf = [0; 5];
        assert_eq!(connection.read(&mut buf, deadline).unwrap(), 5);
        assert_eq!(&buf, b"reply");
        assert_eq!(connection.write(b"ok", deadline).unwrap(), 2);
        let mut response = [0; 2];
        peer.read_exact(&mut response).unwrap();
        assert_eq!(&response, b"ok");
        drop(peer);
        assert_eq!(connection.read(&mut buf, deadline).unwrap(), 0);
        assert!(connection.write(b"closed", deadline).is_err());
    }

    #[test]
    fn local_idle_read_times_out() {
        let (mut connection, _peer) = local_pair();
        let start = Instant::now();
        let error = connection
            .read(&mut [0; 8], start + Duration::from_millis(25))
            .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert!(start.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn local_blocked_write_times_out() {
        let (mut connection, _peer) = local_pair();
        let deadline = Instant::now() + Duration::from_millis(30);
        let buffer = [0u8; 65536];
        loop {
            match connection.write(&buffer, deadline) {
                Ok(_) => continue,
                Err(error) => {
                    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
                    break;
                }
            }
        }
    }

    #[test]
    fn subprocess_collection_is_bounded_and_reports_failures() {
        // Only local shell fixtures are run; never invoke bluetoothctl in tests.
        let mut success = Command::new("sh");
        success.args(["-c", "printf 'Device AA:BB:CC:DD:EE:FF Test headphones\\n'"]);
        let output = collect_output(success, Duration::from_secs(2)).unwrap();
        assert!(output.contains("Test headphones"));

        let mut failure = Command::new("sh");
        failure.args(["-c", "printf 'test failure' >&2; exit 3"]);
        assert!(collect_output(failure, Duration::from_secs(2))
            .unwrap_err()
            .to_string()
            .contains("test failure"));

        let mut stalled = Command::new("sh");
        stalled.args(["-c", "exec sleep 60"]);
        let start = Instant::now();
        assert_eq!(
            collect_output(stalled, Duration::from_millis(30))
                .unwrap_err()
                .kind(),
            io::ErrorKind::TimedOut
        );
        assert!(start.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn complete_reads_share_one_deadline() {
        let (inner, mut peer) = local_pair();
        let mut connection = super::super::Connection {
            inner,
            timeout: Duration::from_millis(30),
        };
        peer.write_all(b"partial").unwrap();
        let mut response = [0; 8];
        let error = connection.read_exact(&mut response).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert_eq!(&response[..7], b"partial");
    }
}
