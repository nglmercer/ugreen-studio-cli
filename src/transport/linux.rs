//! Linux ABI references: BlueZ lib/bluetooth/rfcomm.h and Linux asm-generic
//! socket/fcntl headers. No libbluetooth linkage is needed for RFCOMM sockets.
//! <https://github.com/bluez/bluez/blob/master/lib/bluetooth/rfcomm.h>
//! <https://github.com/bluez/bluez/blob/master/doc/bluetoothctl.rst>

use super::{format_address, parse_address, remaining, Device};
use std::ffi::{c_int, c_short, c_ulong, c_void};
use std::io::{self, Read};
use std::mem::size_of;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::process::{Child, Command, Stdio};
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
const F_GETFL: c_int = 3;
const F_SETFL: c_int = 4;

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
    fn fcntl(fd: c_int, command: c_int, ...) -> c_int;
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

fn nonblocking(fd: RawFd) -> io::Result<()> {
    // SAFETY: F_GETFL takes no variadic argument and does not change ownership.
    let flags = unsafe { fcntl(fd, F_GETFL) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: F_SETFL takes one c_int argument; fd remains owned by its pipe.
    if unsafe { fcntl(fd, F_SETFL, flags | SOCK_NONBLOCK) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Always kill/reap a child if an early error interrupts output collection.
struct ChildGuard(Child);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}

const MAX_COMMAND_OUTPUT: usize = 1024 * 1024;

fn drain_pipe(reader: &mut impl Read, output: &mut Vec<u8>) -> io::Result<bool> {
    let mut buffer = [0; 4096];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => return Ok(true),
            Ok(count) => {
                if output.len() + count > MAX_COMMAND_OUTPUT {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "bluetoothctl output exceeded 1 MiB",
                    ));
                }
                output.extend_from_slice(&buffer[..count]);
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => return Ok(false),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    }
}

pub(super) fn list_paired() -> io::Result<Vec<Device>> {
    check_abi()?;
    // This command only reads BlueZ's paired-device cache. In particular, it
    // does not run `scan`, `pair`, `connect`, `agent`, or `power` commands.
    let mut command = Command::new("bluetoothctl");
    command
        .args(["--timeout", "5", "devices", "Paired"])
        .env("LC_ALL", "C");
    collect_devices(command, Duration::from_secs(6))
}

fn collect_devices(mut command: Command, timeout: Duration) -> io::Result<Vec<Device>> {
    let child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            io::Error::new(
                error.kind(),
                format!(
                    "could not run bluetoothctl (install your distribution's BlueZ tools): {error}"
                ),
            )
        })?;
    let mut child = ChildGuard(child);
    let mut stdout = child
        .0
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("missing bluetoothctl stdout"))?;
    let mut stderr = child
        .0
        .stderr
        .take()
        .ok_or_else(|| io::Error::other("missing bluetoothctl stderr"))?;
    nonblocking(stdout.as_raw_fd())?;
    nonblocking(stderr.as_raw_fd())?;
    let deadline = Instant::now() + timeout;
    let mut out = Vec::new();
    let mut err = Vec::new();
    let mut exit = None;
    loop {
        let out_done = drain_pipe(&mut stdout, &mut out)?;
        let err_done = drain_pipe(&mut stderr, &mut err)?;
        if exit.is_none() {
            exit = child.0.try_wait()?;
        }
        if let Some(status) = exit {
            if out_done && err_done {
                let stdout = String::from_utf8_lossy(&out);
                let stderr = String::from_utf8_lossy(&err);
                if !status.success() {
                    return Err(io::Error::other(format!(
                        "bluetoothctl failed ({status}): {} {}",
                        stderr.trim(),
                        stdout.trim()
                    )));
                }
                if !stderr.trim().is_empty() {
                    return Err(io::Error::other(format!(
                        "bluetoothctl reported: {}",
                        stderr.trim()
                    )));
                }
                return parse_devices(&stdout);
            }
        }
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "bluetoothctl timed out; check that the BlueZ service is running",
            ));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
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

fn parse_devices(output: &str) -> io::Result<Vec<Device>> {
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
            "unexpected bluetoothctl output: {line}; this command requires BlueZ support for 'devices Paired'")))?;
        let (address, name) = rest.split_once(' ').unwrap_or((rest, ""));
        let address = parse_address(address).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("bluetoothctl returned an invalid device address: {address}"),
            )
        })?;
        devices.push(Device {
            address: format_address(address),
            name: name.trim().to_owned(),
        });
    }
    Ok(devices)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
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
        let devices = parse_devices("\u{1b}[0;94mDevice\u{1b}[0m aa:bb:cc:dd:ee:ff UGREEN Studio Pro\nDevice 01:23:45:67:89:AB Headphones with spaces\n").unwrap();
        assert_eq!(devices[0].address, "AA:BB:CC:DD:EE:FF");
        assert_eq!(devices[0].name, "UGREEN Studio Pro");
        assert_eq!(devices[1].name, "Headphones with spaces");
        assert!(parse_devices("").unwrap().is_empty());
        assert!(parse_devices("Device invalid Name").is_err());
        assert!(parse_devices("Invalid command in menu main: devices").is_err());
        assert!(parse_devices("No default controller available").is_err());
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
        let devices = collect_devices(success, Duration::from_secs(2)).unwrap();
        assert_eq!(devices[0].name, "Test headphones");

        let mut failure = Command::new("sh");
        failure.args(["-c", "printf 'test failure' >&2; exit 3"]);
        assert!(collect_devices(failure, Duration::from_secs(2))
            .unwrap_err()
            .to_string()
            .contains("test failure"));

        let mut stalled = Command::new("sh");
        stalled.args(["-c", "exec sleep 60"]);
        let start = Instant::now();
        assert_eq!(
            collect_devices(stalled, Duration::from_millis(30))
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
