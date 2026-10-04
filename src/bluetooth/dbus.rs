//! Minimal BlueZ enumeration over the D-Bus system bus.
//!
//! Why a hand-written client: the project keeps its dependency set at
//! ratatui/crossterm only, and this needs exactly two method calls
//! (`Hello`, `ObjectManager.GetManagedObjects`) with typed results —
//! no async runtime, no signal subscriptions, no reconnection logic.
//!
//! Safety bounds: the socket path is fixed, reads/writes carry a 5 s
//! timeout, message sizes are capped, the reply wait is bounded, and
//! nothing here ever writes to BlueZ (read-only `GetManagedObjects`).
//! Any failure propagates to the caller, which falls back to the
//! bounded `bluetoothctl` path.

use super::{BluetoothAddress, HostConnectionState, HostHeadset};
use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

const SYSTEM_BUS: &str = "/run/dbus/system_bus_socket";
const IO_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_FIELDS: usize = 128 * 1024;
const MAX_BODY: usize = 8 * 1024 * 1024;
const MAX_MESSAGES: usize = 64;

const MESSAGE_METHOD_RETURN: u8 = 2;
const MESSAGE_ERROR: u8 = 3;

const DEVICE_INTERFACE: &str = "org.bluez.Device1";

pub(super) fn list_devices() -> io::Result<Vec<HostHeadset>> {
    let mut bus = Bus::open()?;
    let objects = bus.get_managed_objects()?;
    let mut devices: Vec<HostHeadset> = objects
        .into_iter()
        .filter_map(ManagedObject::into_headset)
        .collect();
    super::normalize(&mut devices);
    Ok(devices)
}

struct Bus {
    stream: UnixStream,
    serial: u32,
}

impl Bus {
    fn open() -> io::Result<Self> {
        let stream = UnixStream::connect(SYSTEM_BUS)?;
        stream.set_read_timeout(Some(IO_TIMEOUT))?;
        stream.set_write_timeout(Some(IO_TIMEOUT))?;
        let mut bus = Self { stream, serial: 0 };
        bus.authenticate()?;
        // Every connection must announce itself before other calls work.
        bus.call(
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus",
            "Hello",
            "org.freedesktop.DBus",
            None,
        )?;
        bus.await_reply()?;
        Ok(bus)
    }

    /// SASL `EXTERNAL` handshake over the unix socket: the leading NUL
    /// byte, the process uid as a hex-encoded decimal string, then
    /// `BEGIN`.
    fn authenticate(&mut self) -> io::Result<()> {
        // SAFETY: getuid takes no arguments and returns the caller's uid.
        let uid = unsafe { getuid() };
        let mut command = String::from("AUTH EXTERNAL ");
        for byte in uid.to_string().bytes() {
            command.push(char::from(b"0123456789ABCDEF"[usize::from(byte >> 4)]));
            command.push(char::from(b"0123456789ABCDEF"[usize::from(byte & 15)]));
        }
        command.push_str("\r\n");
        self.stream.write_all(b"\0")?;
        self.stream.write_all(command.as_bytes())?;
        let line = read_line(&mut self.stream, 4096)?;
        if !line.starts_with("OK ") {
            return Err(io::Error::other(format!(
                "D-Bus authentication refused: {line}"
            )));
        }
        self.stream.write_all(b"BEGIN\r\n")
    }

    fn call(
        &mut self,
        path: &str,
        interface: &str,
        member: &str,
        destination: &str,
        signature: Option<&str>,
    ) -> io::Result<()> {
        self.serial = self
            .serial
            .checked_add(1)
            .ok_or_else(|| io::Error::other("D-Bus serial space exhausted during enumeration"))?;
        let message =
            build_method_call(self.serial, path, interface, member, destination, signature)?;
        self.stream.write_all(&message)
    }

    fn await_reply(&mut self) -> io::Result<Message> {
        for _ in 0..MAX_MESSAGES {
            let message = read_message(&mut self.stream)?;
            match message.kind {
                MESSAGE_METHOD_RETURN if message.reply_serial == Some(self.serial) => {
                    return Ok(message);
                }
                MESSAGE_ERROR if message.reply_serial == Some(self.serial) => {
                    return Err(io::Error::other(format!(
                        "D-Bus error: {}",
                        message.error_name.unwrap_or_else(|| "unknown".into())
                    )));
                }
                // Signals and unrelated replies are irrelevant to a
                // strictly sequential enumerator.
                _ => continue,
            }
        }
        Err(io::Error::other(
            "too many D-Bus messages before the method reply",
        ))
    }

    fn get_managed_objects(&mut self) -> io::Result<Vec<ManagedObject>> {
        self.call(
            "/",
            "org.freedesktop.DBus.ObjectManager",
            "GetManagedObjects",
            "org.bluez",
            None,
        )?;
        let reply = self.await_reply()?;
        parse_managed_objects(&reply.body, reply.little_endian)
    }
}

unsafe extern "C" {
    // SAFETY: getuid is POSIX, allocation-free and safe to call on any thread.
    fn getuid() -> u32;
}

// ---------------------------------------------------------------------------
// Marshalling (method calls we send)
// ---------------------------------------------------------------------------

fn align(buffer: &mut Vec<u8>, boundary: usize) {
    while !buffer.len().is_multiple_of(boundary) {
        buffer.push(0);
    }
}

fn push_u32_native(buffer: &mut Vec<u8>, value: u32) {
    buffer.extend_from_slice(&value.to_ne_bytes());
}

fn push_string(buffer: &mut Vec<u8>, value: &str) {
    align(buffer, 4);
    push_u32_native(buffer, value.len() as u32);
    buffer.extend_from_slice(value.as_bytes());
    buffer.push(0);
}

/// One `(byte code, variant)` struct entry inside the header array.
fn push_field_string(buffer: &mut Vec<u8>, code: u8, value: &str, signature: u8) {
    align(buffer, 8);
    buffer.push(code);
    // Variant containing a single string/object-path value.
    buffer.extend_from_slice(&[1, signature, 0]);
    push_string(buffer, value);
}

fn push_field_signature(buffer: &mut Vec<u8>, code: u8, value: &str) {
    align(buffer, 8);
    buffer.push(code);
    // Variant whose own signature is `g`.
    buffer.extend_from_slice(&[1, b'g', 0]);
    buffer.push(value.len() as u8);
    buffer.extend_from_slice(value.as_bytes());
    buffer.push(0);
}

fn build_method_call(
    serial: u32,
    path: &str,
    interface: &str,
    member: &str,
    destination: &str,
    signature: Option<&str>,
) -> io::Result<Vec<u8>> {
    if path.len() > 255 || interface.len() > 255 || member.len() > 255 || destination.len() > 255 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "D-Bus destination field too long",
        ));
    }
    let mut fields = Vec::new();
    push_field_string(&mut fields, 1, path, b'o'); // PATH
    push_field_string(&mut fields, 2, interface, b's');
    push_field_string(&mut fields, 3, member, b's');
    push_field_string(&mut fields, 6, destination, b's');
    if let Some(signature) = signature {
        push_field_signature(&mut fields, 8, signature);
    }
    let body: &[u8] = &[];
    let mut message = Vec::with_capacity(16 + fields.len() + body.len());
    message.push(if cfg!(target_endian = "little") {
        b'l'
    } else {
        b'B'
    });
    message.push(1); // METHOD_CALL
    message.push(0); // flags: a reply is expected
    message.push(1); // protocol version
    push_u32_native(&mut message, body.len() as u32);
    push_u32_native(&mut message, serial);
    push_u32_native(&mut message, fields.len() as u32);
    message.extend_from_slice(&fields);
    align(&mut message, 8);
    message.extend_from_slice(body);
    Ok(message)
}

// ---------------------------------------------------------------------------
// Unmarshalling (replies we read)
// ---------------------------------------------------------------------------

struct Message {
    kind: u8,
    reply_serial: Option<u32>,
    error_name: Option<String>,
    body: Vec<u8>,
    little_endian: bool,
}

fn u32_at(bytes: &[u8], little_endian: bool) -> u32 {
    let raw = [bytes[0], bytes[1], bytes[2], bytes[3]];
    if little_endian {
        u32::from_le_bytes(raw)
    } else {
        u32::from_be_bytes(raw)
    }
}

fn read_message(stream: &mut UnixStream) -> io::Result<Message> {
    let mut fixed = [0u8; 16];
    stream.read_exact(&mut fixed)?;
    let little_endian = match fixed[0] {
        b'l' => true,
        b'B' => false,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "D-Bus message has an invalid byte-order marker",
            ))
        }
    };
    let kind = fixed[1];
    let body_len = u32_at(&fixed[4..8], little_endian) as usize;
    let fields_len = u32_at(&fixed[12..16], little_endian) as usize;
    if fields_len > MAX_FIELDS || body_len > MAX_BODY {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "D-Bus message exceeds the size bound",
        ));
    }
    let mut fields = vec![0u8; fields_len];
    stream.read_exact(&mut fields)?;
    // The body begins at an 8-byte boundary measured from the message start.
    let header = 16 + fields_len;
    let padding = (8 - header % 8) % 8;
    let mut discarded = [0u8; 8];
    stream.read_exact(&mut discarded[..padding])?;
    let mut body = vec![0u8; body_len];
    stream.read_exact(&mut body)?;

    let mut reader = Reader::new(&fields, little_endian);
    let mut reply_serial = None;
    let mut error_name = None;
    while reader.pos() < fields.len() {
        reader.align(8)?;
        let code = reader.u8()?;
        let signature = reader.signature()?;
        match (code, signature.as_str()) {
            (5, "u") => reply_serial = Some(reader.u32()?),
            (4, "s") => error_name = Some(reader.string()?),
            _ => reader.skip_one(&signature)?,
        }
    }
    Ok(Message {
        kind,
        reply_serial,
        error_name,
        body,
        little_endian,
    })
}

/// Bounds-checked reader over one marshalled region. All padding
/// decisions are absolute relative to the region start, which callers
/// keep 8-byte aligned exactly like a message body/header array.
struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
    little_endian: bool,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8], little_endian: bool) -> Self {
        Self {
            data,
            pos: 0,
            little_endian,
        }
    }
    fn pos(&self) -> usize {
        self.pos
    }
    fn align(&mut self, boundary: usize) -> io::Result<()> {
        let remainder = self.pos % boundary;
        if remainder != 0 {
            self.pos += boundary - remainder;
        }
        if self.pos > self.data.len() {
            return Err(truncated());
        }
        Ok(())
    }
    fn take(&mut self, count: usize) -> io::Result<&'a [u8]> {
        let end = self.pos.checked_add(count).ok_or_else(truncated)?;
        let slice = self.data.get(self.pos..end).ok_or_else(truncated)?;
        self.pos = end;
        Ok(slice)
    }
    fn u8(&mut self) -> io::Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> io::Result<u16> {
        self.align(2)?;
        let bytes: [u8; 2] = self.take(2)?.try_into().expect("two bytes");
        Ok(if self.little_endian {
            u16::from_le_bytes(bytes)
        } else {
            u16::from_be_bytes(bytes)
        })
    }
    fn u32(&mut self) -> io::Result<u32> {
        self.align(4)?;
        let bytes: [u8; 4] = self.take(4)?.try_into().expect("four bytes");
        Ok(if self.little_endian {
            u32::from_le_bytes(bytes)
        } else {
            u32::from_be_bytes(bytes)
        })
    }
    fn u64(&mut self) -> io::Result<u64> {
        self.align(8)?;
        let bytes: [u8; 8] = self.take(8)?.try_into().expect("eight bytes");
        Ok(if self.little_endian {
            u64::from_le_bytes(bytes)
        } else {
            u64::from_be_bytes(bytes)
        })
    }
    /// `s` / `o`: 4-aligned length, bytes, NUL.
    fn string(&mut self) -> io::Result<String> {
        let length = self.u32()? as usize;
        let bytes = self.take(length)?;
        let text = std::str::from_utf8(bytes)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "D-Bus string is not UTF-8"))?
            .to_owned();
        if self.u8()? != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "D-Bus string lacks its terminator",
            ));
        }
        Ok(text)
    }
    /// `g`: 1-byte length, bytes, NUL.
    fn signature(&mut self) -> io::Result<String> {
        let length = self.u8()? as usize;
        let bytes = self.take(length)?;
        let text = std::str::from_utf8(bytes)
            .map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "D-Bus signature is not ASCII")
            })?
            .to_owned();
        if self.u8()? != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "D-Bus signature lacks its terminator",
            ));
        }
        Ok(text)
    }
    fn boolean(&mut self) -> io::Result<bool> {
        Ok(self.u32()? != 0)
    }

    /// Skip exactly one value of the single complete type in `signature`.
    fn skip_one(&mut self, signature: &str) -> io::Result<()> {
        let (kind, end) = next_type(signature, 0).ok_or_else(truncated)?;
        if end != signature.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "D-Bus variant signature holds more than one type",
            ));
        }
        self.skip_kind(kind)
    }

    fn skip_kind(&mut self, kind: &str) -> io::Result<()> {
        let bytes = kind.as_bytes();
        match *bytes.first().ok_or_else(truncated)? {
            b'y' => {
                self.take(1)?;
            }
            b'b' | b'i' | b'u' | b'h' => {
                self.u32()?;
            }
            b'n' | b'q' => {
                self.u16()?;
            }
            b'x' | b't' | b'd' => {
                self.u64()?;
            }
            b's' | b'o' => {
                self.string()?;
            }
            b'g' => {
                self.signature()?;
            }
            b'v' => {
                let inner = self.signature()?;
                self.skip_kind(&inner)?;
            }
            b'a' => {
                let (element, _) = next_type(kind, 1).ok_or_else(truncated)?;
                self.align(4)?;
                let length = self.u32()? as usize;
                self.align(content_alignment(element))?;
                self.take(length)?;
            }
            b'(' | b'{' => {
                self.align(8)?;
                let inner = &kind[1..kind.len() - 1];
                let mut cursor = 0;
                while cursor < inner.len() {
                    let (member, end) = next_type(inner, cursor).ok_or_else(truncated)?;
                    self.skip_kind(member)?;
                    cursor = end;
                }
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("unsupported D-Bus type in {kind}"),
                ))
            }
        }
        Ok(())
    }
}

fn truncated() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "truncated D-Bus message")
}

/// Content alignment of a complete type, used for array bodies.
fn content_alignment(kind: &str) -> usize {
    match kind.as_bytes().first().copied() {
        Some(b'y' | b'g' | b'v') => 1,
        Some(b'n' | b'q') => 2,
        Some(b'b' | b'i' | b'u' | b'h' | b's' | b'o' | b'a') => 4,
        _ => 8,
    }
}

/// Split one complete type out of a signature starting at `start`;
/// returns (type text, index after it).
fn next_type(signature: &str, start: usize) -> Option<(&str, usize)> {
    let bytes = signature.as_bytes();
    match *bytes.get(start)? {
        b'a' => next_type(signature, start + 1).map(|(_, end)| (&signature[start..end], end)),
        b'(' | b'{' => {
            let close = if bytes[start] == b'(' { b')' } else { b'}' };
            let mut depth = 0usize;
            let mut index = start;
            while index < bytes.len() {
                match bytes[index] {
                    b'(' | b'{' => depth += 1,
                    b')' | b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            if bytes[index] != close {
                                return None;
                            }
                            return Some((&signature[start..=index], index + 1));
                        }
                    }
                    _ => {}
                }
                index += 1;
            }
            None
        }
        _ => Some((&signature[start..start + 1], start + 1)),
    }
}

// ---------------------------------------------------------------------------
// ObjectManager results
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ManagedObject {
    path: String,
    address: Option<String>,
    name: Option<String>,
    alias: Option<String>,
    paired: Option<bool>,
    connected: Option<bool>,
}

impl ManagedObject {
    /// Convert only BlueZ device objects that are paired or connected.
    /// Names never become identity; an unusable address drops the entry.
    fn into_headset(self) -> Option<HostHeadset> {
        let paired = self.paired.unwrap_or(false);
        let connected = self.connected;
        if !paired && connected != Some(true) {
            return None;
        }
        let address = BluetoothAddress::parse(self.address.as_deref()?).ok()?;
        let name = self
            .alias
            .or(self.name)
            .unwrap_or_else(|| address.to_string());
        let state = match connected {
            Some(true) => HostConnectionState::Connected,
            Some(false) => HostConnectionState::Disconnected,
            None => HostConnectionState::Unknown,
        };
        Some(HostHeadset {
            address,
            name,
            paired,
            connected: state,
            control_available: None,
            model: None,
        })
    }

    fn consume_prop(
        &mut self,
        key: &str,
        signature: &str,
        reader: &mut Reader<'_>,
    ) -> io::Result<()> {
        match (key, signature) {
            ("Address", "s") | ("Name", "s") | ("Alias", "s") => {
                let value = reader.string()?;
                match key {
                    "Address" => self.address = Some(value),
                    "Name" => self.name = Some(value),
                    _ => self.alias = Some(value),
                }
            }
            ("Paired", "b") | ("Connected", "b") => {
                let value = reader.boolean()?;
                if key == "Paired" {
                    self.paired = Some(value);
                } else {
                    self.connected = Some(value);
                }
            }
            _ => reader.skip_one(signature)?,
        }
        Ok(())
    }
}

fn parse_managed_objects(body: &[u8], little_endian: bool) -> io::Result<Vec<ManagedObject>> {
    let mut reader = Reader::new(body, little_endian);
    reader.align(4)?;
    let total = reader.u32()? as usize;
    reader.align(8)?;
    let end = reader
        .pos()
        .checked_add(total)
        .filter(|end| *end <= body.len())
        .ok_or_else(truncated)?;
    let mut objects = Vec::new();
    while reader.pos() < end {
        reader.align(8)?;
        if reader.pos() >= end {
            break;
        }
        let path = reader.string()?;
        let mut object = ManagedObject {
            path,
            ..ManagedObject::default()
        };
        reader.align(4)?;
        let interfaces_len = reader.u32()? as usize;
        reader.align(8)?;
        let interfaces_end = reader
            .pos()
            .checked_add(interfaces_len)
            .filter(|limit| *limit <= body.len())
            .ok_or_else(truncated)?;
        while reader.pos() < interfaces_end {
            reader.align(8)?;
            let interface = reader.string()?;
            reader.align(4)?;
            let properties_len = reader.u32()? as usize;
            reader.align(8)?;
            let properties_end = reader
                .pos()
                .checked_add(properties_len)
                .filter(|limit| *limit <= body.len())
                .ok_or_else(truncated)?;
            while reader.pos() < properties_end {
                reader.align(8)?;
                let key = reader.string()?;
                let signature = reader.signature()?;
                if interface == DEVICE_INTERFACE {
                    object.consume_prop(&key, &signature, &mut reader)?;
                } else {
                    reader.skip_one(&signature)?;
                }
            }
        }
        objects.push(object);
    }
    Ok(objects)
}

fn read_line(stream: &mut UnixStream, max: usize) -> io::Result<String> {
    let mut buffer = Vec::new();
    let mut byte = [0u8; 1];
    while buffer.len() < max {
        match stream.read(&mut byte) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "D-Bus socket closed during authentication",
                ))
            }
            Ok(_) => {
                buffer.push(byte[0]);
                if buffer.ends_with(b"\r\n") {
                    buffer.truncate(buffer.len() - 2);
                    return Ok(String::from_utf8_lossy(&buffer).into_owned());
                }
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "D-Bus authentication line exceeded its bound",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Independent, byte-by-byte writer for test fixtures. It mirrors the
    // published D-Bus layout rather than reusing the reader above, so a
    // shared mistake cannot hide behind a round trip.
    struct Writer {
        bytes: Vec<u8>,
        le: bool,
    }
    impl Writer {
        fn new(le: bool) -> Self {
            Self {
                bytes: Vec::new(),
                le,
            }
        }
        fn align(&mut self, boundary: usize) {
            align(&mut self.bytes, boundary);
        }
        fn u32(&mut self, value: u32) {
            self.align(4);
            if self.le {
                self.bytes.extend_from_slice(&value.to_le_bytes());
            } else {
                self.bytes.extend_from_slice(&value.to_be_bytes());
            }
        }
        fn string(&mut self, value: &str) {
            self.u32(value.len() as u32);
            self.bytes.extend_from_slice(value.as_bytes());
            self.bytes.push(0);
        }
        fn signature(&mut self, value: &str) {
            self.bytes.push(value.len() as u8);
            self.bytes.extend_from_slice(value.as_bytes());
            self.bytes.push(0);
        }
        fn variant_string(&mut self, value: &str) {
            self.signature("s");
            self.string(value);
        }
        fn variant_bool(&mut self, value: bool) {
            self.signature("b");
            self.u32(u32::from(value));
        }
        fn dict_entry<F: FnOnce(&mut Self)>(&mut self, build: F) {
            self.align(8);
            build(self);
        }
        /// Array with explicit element alignment: 8 for structs/dict
        /// entries, 4 for strings/arrays, per the D-Bus layout rules.
        fn array<F: FnOnce(&mut Self)>(&mut self, element_alignment: usize, build: F) {
            self.align(4);
            let length_at = self.bytes.len();
            self.bytes.extend_from_slice(&[0, 0, 0, 0]);
            self.align(element_alignment);
            let content_at = self.bytes.len();
            build(self);
            let length = (self.bytes.len() - content_at) as u32;
            if self.le {
                self.bytes[length_at..length_at + 4].copy_from_slice(&length.to_le_bytes());
            } else {
                self.bytes[length_at..length_at + 4].copy_from_slice(&length.to_be_bytes());
            }
        }
        fn finish(self) -> Vec<u8> {
            self.bytes
        }
    }

    /// One device with a mixed property set, including values that must
    /// be skipped (a string array inside a variant).
    fn managed_objects_fixture(le: bool) -> Vec<u8> {
        let mut w = Writer::new(le);
        w.array(8, |w| {
            w.dict_entry(|w| {
                w.string("/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF");
                // a{sa{sv}}
                w.array(8, |w| {
                    w.dict_entry(|w| {
                        w.string(DEVICE_INTERFACE);
                        // a{sv}
                        w.array(8, |w| {
                            w.dict_entry(|w| {
                                w.string("Address");
                                w.variant_string("AA:BB:CC:DD:EE:FF");
                            });
                            w.dict_entry(|w| {
                                w.string("Alias");
                                w.variant_string("UGREEN Studio Pro");
                            });
                            w.dict_entry(|w| {
                                w.string("Paired");
                                w.variant_bool(true);
                            });
                            w.dict_entry(|w| {
                                w.string("Connected");
                                w.variant_bool(true);
                            });
                            w.dict_entry(|w| {
                                w.string("UUIDs");
                                w.signature("as");
                                w.array(4, |w| {
                                    w.string("0000110b-0000-1000-8000-00805f9b34fb");
                                    w.string("0000110e-0000-1000-8000-00805f9b34fb");
                                });
                            });
                        });
                    });
                    w.dict_entry(|w| {
                        w.string("org.freedesktop.DBus.Properties");
                        w.array(8, |_| {});
                    });
                });
            });
            w.dict_entry(|w| {
                w.string("/org/bluez/hci0");
                w.array(8, |w| {
                    w.dict_entry(|w| {
                        w.string("org.bluez.Adapter1");
                        w.array(8, |w| {
                            w.dict_entry(|w| {
                                w.string("Address");
                                w.variant_string("90:DE:80:66:D2:28");
                            });
                        });
                    });
                });
            });
        });
        w.finish()
    }

    #[test]
    fn parses_object_manager_body_with_skipped_properties() {
        let body = managed_objects_fixture(true);
        let objects = parse_managed_objects(&body, true).unwrap();
        assert_eq!(objects.len(), 2);

        let device = &objects[0];
        assert_eq!(device.path, "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF");
        assert_eq!(device.address.as_deref(), Some("AA:BB:CC:DD:EE:FF"));
        assert_eq!(device.alias.as_deref(), Some("UGREEN Studio Pro"));
        assert_eq!(device.paired, Some(true));
        assert_eq!(device.connected, Some(true));

        let headset = device.clone().into_headset().unwrap();
        assert_eq!(headset.address.as_str(), "AA:BB:CC:DD:EE:FF");
        assert_eq!(headset.name, "UGREEN Studio Pro");
        assert!(headset.paired);
        assert_eq!(headset.connected, HostConnectionState::Connected);

        // Adapters are not headsets: their properties are skipped and the
        // entry never reaches the listing.
        assert_eq!(objects[1].path, "/org/bluez/hci0");
        assert!(objects[1].address.is_none());
        assert!(objects[1].clone().into_headset().is_none());
    }

    #[test]
    fn unpaired_and_disconnected_objects_are_dropped() {
        let mut object = ManagedObject {
            path: "/org/bluez/hci0/dev_11_22_33_44_55_66".into(),
            address: Some("11:22:33:44:55:66".into()),
            paired: Some(false),
            connected: Some(false),
            ..ManagedObject::default()
        };
        assert!(object.clone().into_headset().is_none());
        // Connected without pairing stays listed and honest about pairing.
        object.connected = Some(true);
        let headset = object.into_headset().unwrap();
        assert!(!headset.paired);
        assert_eq!(headset.connected, HostConnectionState::Connected);
    }

    #[test]
    fn invalid_addresses_never_reach_the_listing() {
        let object = ManagedObject {
            path: "/org/bluez/hci0/dev_bad".into(),
            address: Some("not-an-address".into()),
            paired: Some(true),
            ..ManagedObject::default()
        };
        assert!(object.into_headset().is_none());
    }

    #[test]
    fn truncated_and_oversized_bodies_are_rejected() {
        let body = managed_objects_fixture(true);
        for cut in 0..body.len() {
            if cut % 17 == 0 {
                assert!(
                    parse_managed_objects(&body[..cut], true).is_err(),
                    "prefix of {cut} bytes must not parse"
                );
            }
        }
        let mut reader = Reader::new(&[0, 1, 2], true);
        assert!(reader.u32().is_err());
        assert!(reader.string().is_err());
    }

    #[test]
    fn byte_order_flag_really_flips_decoding() {
        // A fixture written big endian parses only with the flag clear,
        // and the little-endian fixture parses only with the flag set:
        // the u32 lengths differ enough to blow past the buffer.
        let le_body = managed_objects_fixture(true);
        let be_body = managed_objects_fixture(false);
        let le_objects = parse_managed_objects(&le_body, true)
            .expect("little-endian fixture must parse little-endian");
        assert_eq!(le_objects.len(), 2);
        let be_objects = parse_managed_objects(&be_body, false)
            .expect("big-endian fixture must parse big-endian");
        assert_eq!(be_objects.len(), 2);
        assert_eq!(be_objects[0].alias.as_deref(), Some("UGREEN Studio Pro"));
        if let Ok(objects) = parse_managed_objects(&le_body, false) {
            assert!(objects.is_empty(), "cross-endian parse produced garbage");
        }
        // The reader honours the flag on raw scalars, too.
        let mut reader = Reader::new(&[0, 0, 0, 7], false);
        assert_eq!(reader.u32().unwrap(), 7);
        let mut reader = Reader::new(&[0, 0, 0, 7], true);
        assert_eq!(reader.u32().unwrap(), 0x07000000);
    }

    #[test]
    fn method_call_layout_matches_the_spec() {
        let message = build_method_call(
            7,
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus",
            "Hello",
            "org.freedesktop.DBus",
            None,
        )
        .unwrap();
        assert!(message[0] == b'l' || message[0] == b'B');
        assert_eq!(message[1], 1);
        assert_eq!(&message[4..8], &0u32.to_ne_bytes());
        assert_eq!(&message[8..12], &7u32.to_ne_bytes());
        // Header fields array ends, then padding to an 8-byte body.
        let fields_len = u32_at(&message[12..16], message[0] == b'l') as usize;
        assert_eq!(message.len(), (16 + fields_len).div_ceil(8) * 8);
        assert_eq!(message.len() % 8, 0);
        let fields = &message[16..16 + fields_len];
        assert!(fields.windows(5).any(|w| w.eq(&b"Hello"[..])));
        assert!(fields
            .windows(20)
            .any(|w| w.eq(&b"org.freedesktop.DBus"[..])));
    }

    #[test]
    fn next_type_handles_nested_signatures() {
        assert_eq!(next_type("s", 0), Some(("s", 1)));
        assert_eq!(next_type("as", 0), Some(("as", 2)));
        assert_eq!(next_type("a{sv}", 0), Some(("a{sv}", 5)));
        assert_eq!(next_type("(ssa{sv})", 0), Some(("(ssa{sv})", 9)));
        assert_eq!(next_type("a(ss)", 0), Some(("a(ss)", 5)));
        assert_eq!(next_type("", 0), None);
        assert_eq!(next_type("(sa", 0), None);
        // Textually balanced, even where the wire grammar would object.
        assert_eq!(next_type("(s{sv})", 0), Some(("(s{sv})", 7)));
    }

    /// Live system bus read; ignored by default like all hardware tests.
    #[test]
    #[ignore = "requires a running system bus with bluetoothd"]
    fn live_system_bus_device_listing() {
        let devices = list_devices().expect("system bus should answer");
        for device in &devices {
            assert!(!device.name.is_empty() || device.paired);
        }
        println!("{} device(s) via D-Bus", devices.len());
    }
}
