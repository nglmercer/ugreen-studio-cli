//! Minimal Win32 bindings checked against Microsoft's SDK declarations:
//! <https://learn.microsoft.com/windows/win32/bluetooth/bluetooth-and-connect>
//! <https://learn.microsoft.com/windows/win32/api/winsock2/nf-winsock2-select>
//! <https://github.com/microsoft/win32metadata/blob/main/generation/WinSDK/RecompiledIdlHeaders/um/ws2bth.h>
//! <https://github.com/microsoft/win32metadata/blob/main/generation/WinSDK/RecompiledIdlHeaders/um/bluetoothapis.h>

use super::{format_address, remaining, Device};
use std::ffi::{c_char, c_int, c_void};
use std::io;
use std::mem::{size_of, zeroed};
use std::ptr::{null, null_mut};
use std::time::Instant;

type Socket = usize;
const INVALID_SOCKET: Socket = !0;
const AF_BTH: c_int = 32;
const SOCK_STREAM: c_int = 1;
const BTHPROTO_RFCOMM: c_int = 3;
const FIONBIO: c_int = 0x8004667Eu32 as c_int;
const SOL_SOCKET: c_int = 0xFFFF;
const SO_ERROR: c_int = 0x1007;
const WSAEINTR: c_int = 10004;
const WSAEWOULDBLOCK: c_int = 10035;
const WSAEINPROGRESS: c_int = 10036;
const ERROR_NO_MORE_ITEMS: i32 = 259;
const WSA_FLAG_OVERLAPPED: u32 = 1;
const WSA_FLAG_NO_HANDLE_INHERIT: u32 = 0x80;

// WinSock2.h places these members in different orders on 32/64-bit Windows.
#[repr(C)]
struct WsaData {
    version: u16,
    high_version: u16,
    #[cfg(target_pointer_width = "64")]
    max_sockets: u16,
    #[cfg(target_pointer_width = "64")]
    max_udp_datagram: u16,
    #[cfg(target_pointer_width = "64")]
    vendor_info: *mut c_char,
    description: [c_char; 257],
    system_status: [c_char; 129],
    #[cfg(target_pointer_width = "32")]
    max_sockets: u16,
    #[cfg(target_pointer_width = "32")]
    max_udp_datagram: u16,
    #[cfg(target_pointer_width = "32")]
    vendor_info: *mut c_char,
}

// ws2bth.h uses pshpack1.h, even on Win64. repr(C) alone would be WRONG:
// btAddr must start at byte 2 and the entire address must be 30 bytes.
#[repr(C, packed)]
struct SockAddrBth {
    family: u16,
    address: u64,
    service_class_id: [u8; 16],
    port: u32,
}

#[repr(C)]
struct FdSet {
    count: u32,
    sockets: [Socket; 64],
}
impl FdSet {
    fn one(socket: Socket) -> Self {
        let mut sockets = [0; 64];
        sockets[0] = socket;
        Self { count: 1, sockets }
    }
}

#[repr(C)]
struct TimeVal {
    seconds: i32,
    microseconds: i32,
}

#[link(name = "ws2_32")]
unsafe extern "system" {
    fn WSAStartup(version: u16, data: *mut WsaData) -> c_int;
    fn WSACleanup() -> c_int;
    fn WSAGetLastError() -> c_int;
    fn WSASocketW(
        family: c_int,
        kind: c_int,
        protocol: c_int,
        protocol_info: *const c_void,
        group: u32,
        flags: u32,
    ) -> Socket;
    fn closesocket(socket: Socket) -> c_int;
    fn ioctlsocket(socket: Socket, command: c_int, value: *mut u32) -> c_int;
    fn connect(socket: Socket, address: *const c_void, length: c_int) -> c_int;
    fn getsockopt(
        socket: Socket,
        level: c_int,
        option: c_int,
        value: *mut c_char,
        length: *mut c_int,
    ) -> c_int;
    fn select(
        ignored: c_int,
        read: *mut FdSet,
        write: *mut FdSet,
        error: *mut FdSet,
        timeout: *const TimeVal,
    ) -> c_int;
    fn recv(socket: Socket, buffer: *mut c_char, length: c_int, flags: c_int) -> c_int;
    fn send(socket: Socket, buffer: *const c_char, length: c_int, flags: c_int) -> c_int;
}

#[derive(Debug)]
struct Winsock;
impl Winsock {
    fn start() -> io::Result<Self> {
        // SAFETY: this C output structure consists only of integers/arrays/raw
        // pointers for which the zero bit pattern is valid; correct ABI below.
        let mut data: WsaData = unsafe { zeroed() };
        // SAFETY: data is writable with the WSADATA layout and exact size.
        let result = unsafe { WSAStartup(0x0202, &mut data) };
        if result != 0 {
            return Err(io::Error::from_raw_os_error(result));
        }
        let guard = Self;
        if data.version != 0x0202 {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Winsock 2.2 is required for Bluetooth RFCOMM",
            ));
        }
        Ok(guard)
    }
}
impl Drop for Winsock {
    fn drop(&mut self) {
        // SAFETY: each instance balances exactly one successful WSAStartup;
        // the owning Connection closes its socket before this field is dropped.
        unsafe {
            WSACleanup();
        }
    }
}

#[derive(Debug)]
pub(super) struct Connection {
    socket: Socket,
    _winsock: Winsock,
}

impl Drop for Connection {
    fn drop(&mut self) {
        // SAFETY: socket is live and exclusively owned. No linger option is set,
        // so closing does not synchronously wait for outgoing data to drain.
        unsafe {
            closesocket(self.socket);
        }
    }
}

fn last_socket_error() -> io::Error {
    // SAFETY: querying thread-local Winsock error has no pointer arguments.
    io::Error::from_raw_os_error(unsafe { WSAGetLastError() })
}

impl Connection {
    pub(super) fn connect(address: [u8; 6], channel: u8, deadline: Instant) -> io::Result<Self> {
        remaining(deadline)?;
        let winsock = Winsock::start()?;
        // SAFETY: scalar parameters select a Classic Bluetooth stream; null
        // protocol_info requests the provider matching family/type/protocol.
        let raw = unsafe {
            WSASocketW(
                AF_BTH,
                SOCK_STREAM,
                BTHPROTO_RFCOMM,
                null(),
                0,
                WSA_FLAG_OVERLAPPED | WSA_FLAG_NO_HANDLE_INHERIT,
            )
        };
        if raw == INVALID_SOCKET {
            return Err(last_socket_error());
        }
        let connection = Self {
            socket: raw,
            _winsock: winsock,
        };
        let mut enabled: u32 = 1;
        // SAFETY: enabled is a valid writable ULONG, and raw is a live socket.
        if unsafe { ioctlsocket(raw, FIONBIO, &mut enabled) } != 0 {
            return Err(last_socket_error());
        }
        let address = address
            .into_iter()
            .fold(0u64, |value, octet| (value << 8) | u64::from(octet));
        let target = SockAddrBth {
            family: AF_BTH as u16,
            address,
            service_class_id: [0; 16],
            port: u32::from(channel),
        };
        remaining(deadline)?;
        // SAFETY: target is initialized and has the SDK's packed SOCKADDR_BTH
        // layout. Windows treats it as a byte-addressed sockaddr input buffer.
        let result = unsafe {
            connect(
                raw,
                (&target as *const SockAddrBth).cast(),
                size_of::<SockAddrBth>() as c_int,
            )
        };
        if result != 0 {
            let error = last_socket_error();
            if !matches!(
                error.raw_os_error(),
                Some(WSAEWOULDBLOCK | WSAEINPROGRESS | WSAEINTR)
            ) {
                return Err(error);
            }
            wait_ready(raw, false, deadline)?;
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
            let length = buf.len().min(c_int::MAX as usize) as c_int;
            // SAFETY: buf is writable for length bytes and socket is live.
            let result = unsafe { recv(self.socket, buf.as_mut_ptr().cast(), length, 0) };
            if result >= 0 {
                return Ok(result as usize);
            }
            let error = last_socket_error();
            match error.raw_os_error() {
                Some(WSAEINTR) => continue,
                Some(WSAEWOULDBLOCK) => wait_ready(self.socket, true, deadline)?,
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
            let length = buf.len().min(c_int::MAX as usize) as c_int;
            // SAFETY: buf is readable for length bytes and socket is live.
            let result = unsafe { send(self.socket, buf.as_ptr().cast(), length, 0) };
            if result >= 0 {
                return Ok(result as usize);
            }
            let error = last_socket_error();
            match error.raw_os_error() {
                Some(WSAEINTR) => continue,
                Some(WSAEWOULDBLOCK) => wait_ready(self.socket, false, deadline)?,
                _ => return Err(error),
            }
        }
    }
}

fn wait_ready(socket: Socket, reading: bool, deadline: Instant) -> io::Result<()> {
    loop {
        let duration = remaining(deadline)?;
        let timeout = TimeVal {
            seconds: duration.as_secs().min(i32::MAX as u64) as i32,
            microseconds: duration.subsec_micros() as i32,
        };
        let mut ready = FdSet::one(socket);
        let mut errors = FdSet::one(socket);
        let (read, write) = if reading {
            (&mut ready as *mut FdSet, null_mut())
        } else {
            (null_mut(), &mut ready as *mut FdSet)
        };
        // SAFETY: all non-null sets are initialized SDK-layout fd_sets with one
        // live socket. timeout points to a valid timeval for this call.
        let result = unsafe { select(0, read, write, &mut errors, &timeout) };
        if result > 0 {
            if errors.count != 0 {
                socket_error(socket)?;
            }
            if ready.count != 0 {
                return Ok(());
            }
        } else if result < 0 {
            let error = last_socket_error();
            if error.raw_os_error() != Some(WSAEINTR) {
                return Err(error);
            }
        }
        // Recompute the remaining time after spurious readiness/EINTR. A
        // long timeout can also exceed the single-call timeval seconds limit.
    }
}

fn socket_error(socket: Socket) -> io::Result<()> {
    let mut value: c_int = 0;
    let mut length = size_of::<c_int>() as c_int;
    // SAFETY: correctly sized writable integer output and live socket.
    if unsafe {
        getsockopt(
            socket,
            SOL_SOCKET,
            SO_ERROR,
            (&mut value as *mut c_int).cast(),
            &mut length,
        )
    } != 0
    {
        return Err(last_socket_error());
    }
    if value != 0 {
        return Err(io::Error::from_raw_os_error(value));
    }
    Ok(())
}

#[repr(C, align(8))]
struct BluetoothAddress {
    value: u64,
}

#[repr(C)]
struct BluetoothDeviceInfo {
    size: u32,
    address: BluetoothAddress,
    class_of_device: u32,
    connected: i32,
    remembered: i32,
    authenticated: i32,
    last_seen: [u16; 8],
    last_used: [u16; 8],
    name: [u16; 248],
}

#[repr(C)]
struct BluetoothSearchParams {
    size: u32,
    authenticated: i32,
    remembered: i32,
    unknown: i32,
    connected: i32,
    issue_inquiry: i32,
    timeout_multiplier: u8,
    radio: *mut c_void,
}

#[link(name = "bthprops")]
unsafe extern "system" {
    fn BluetoothFindFirstDevice(
        params: *const BluetoothSearchParams,
        device: *mut BluetoothDeviceInfo,
    ) -> *mut c_void;
    fn BluetoothFindNextDevice(handle: *mut c_void, device: *mut BluetoothDeviceInfo) -> i32;
    fn BluetoothFindDeviceClose(handle: *mut c_void) -> i32;
}

struct DeviceFind(*mut c_void);
impl Drop for DeviceFind {
    fn drop(&mut self) {
        // SAFETY: this is the valid, exclusively owned enumeration handle
        // returned by BluetoothFindFirstDevice. It is closed exactly once.
        unsafe {
            BluetoothFindDeviceClose(self.0);
        }
    }
}

pub(super) fn list_paired() -> io::Result<Vec<Device>> {
    let params = BluetoothSearchParams {
        size: size_of::<BluetoothSearchParams>() as u32,
        authenticated: 1,
        remembered: 0,
        unknown: 0,
        connected: 0,
        issue_inquiry: 0,
        timeout_multiplier: 0,
        radio: null_mut(),
    };
    // SAFETY: all fields are integers/arrays with a valid zero representation.
    let mut info: BluetoothDeviceInfo = unsafe { zeroed() };
    info.size = size_of::<BluetoothDeviceInfo>() as u32;
    // SAFETY: both structures have the documented ABI and correct dwSize.
    // fIssueInquiry=FALSE forbids active discovery; only authenticated cached
    // Classic Bluetooth devices are requested across existing local radios.
    let handle = unsafe { BluetoothFindFirstDevice(&params, &mut info) };
    if handle.is_null() {
        let error = io::Error::last_os_error();
        if error.raw_os_error() == Some(ERROR_NO_MORE_ITEMS) {
            return Ok(Vec::new());
        }
        return Err(error);
    }
    let handle = DeviceFind(handle);
    let mut devices = Vec::new();
    loop {
        // Defence in depth: never label an unpaired cached device as paired.
        if info.authenticated != 0 {
            let bytes = info.address.value.to_be_bytes();
            let address = [bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]];
            let end = info
                .name
                .iter()
                .position(|ch| *ch == 0)
                .unwrap_or(info.name.len());
            devices.push(Device {
                address: format_address(address),
                name: String::from_utf16_lossy(&info.name[..end]),
            });
        }
        info.size = size_of::<BluetoothDeviceInfo>() as u32;
        // SAFETY: live enumeration handle and properly sized writable output.
        if unsafe { BluetoothFindNextDevice(handle.0, &mut info) } == 0 {
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(ERROR_NO_MORE_ITEMS) {
                break;
            }
            return Err(error);
        }
    }
    Ok(devices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layouts_match_windows_sdk_without_bluetooth_calls() {
        assert_eq!(size_of::<SockAddrBth>(), 30);
        assert_eq!(std::mem::offset_of!(SockAddrBth, address), 2);
        assert_eq!(std::mem::offset_of!(SockAddrBth, port), 26);
        assert_eq!(size_of::<BluetoothDeviceInfo>(), 560);
        assert_eq!(std::mem::offset_of!(BluetoothDeviceInfo, address), 8);
        assert_eq!(std::mem::offset_of!(BluetoothDeviceInfo, name), 64);
        assert_eq!(size_of::<TimeVal>(), 8);
        if cfg!(target_pointer_width = "64") {
            assert_eq!(size_of::<WsaData>(), 408);
            assert_eq!(size_of::<FdSet>(), 520);
            assert_eq!(size_of::<BluetoothSearchParams>(), 40);
        } else {
            assert_eq!(size_of::<WsaData>(), 400);
            assert_eq!(size_of::<FdSet>(), 260);
            assert_eq!(size_of::<BluetoothSearchParams>(), 32);
        }
    }
}
