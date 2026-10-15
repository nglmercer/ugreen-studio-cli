//! D-Bus AddMatch subscriptions invalidate snapshots; signal payloads never
//! authorize operations or replace exact-address host enumeration.
//! See https://dbus.freedesktop.org/doc/dbus-specification.html#message-bus-routing-match-rules
use super::*;
use std::{
    net::Shutdown,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Instant,
};

const RULES: [&str; 5] = [
    "type='signal',sender='org.bluez',interface='org.freedesktop.DBus.Properties',member='PropertiesChanged',path_namespace='/org/bluez',arg0='org.bluez.Device1'",
    "type='signal',sender='org.bluez',interface='org.freedesktop.DBus.Properties',member='PropertiesChanged',path_namespace='/org/bluez',arg0='org.bluez.Adapter1'",
    "type='signal',sender='org.bluez',interface='org.freedesktop.DBus.ObjectManager',member='InterfacesAdded'",
    "type='signal',sender='org.bluez',interface='org.freedesktop.DBus.ObjectManager',member='InterfacesRemoved'",
    "type='signal',sender='org.freedesktop.DBus',interface='org.freedesktop.DBus',member='NameOwnerChanged',arg0='org.bluez'",
];
const MAX_SIGNAL: usize = 256 * 1024;
const TICK: Duration = Duration::from_millis(50);

pub(crate) struct HostSignalMonitor {
    changed: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    socket: Arc<Mutex<Option<UnixStream>>>,
}
impl HostSignalMonitor {
    /// No bus work occurs on the caller's thread. A single background listener
    /// coalesces arbitrarily many notifications into one bounded flag.
    pub(crate) fn start() -> Self {
        let changed = Arc::new(AtomicBool::new(false));
        let stop = Arc::new(AtomicBool::new(false));
        let socket = Arc::new(Mutex::new(None));
        let monitor = Self {
            changed: changed.clone(),
            stop: stop.clone(),
            socket: socket.clone(),
        };
        let _ = thread::Builder::new()
            .name("ugreen-bluez-signals".into())
            .spawn(move || {
                let mut backoff = Duration::from_secs(1);
                while !stop.load(Ordering::Acquire) {
                    let result = (|| -> io::Result<()> {
                        let mut bus = Bus::open()?;
                        if stop.load(Ordering::Acquire) {
                            return Ok(());
                        }
                        *socket
                            .lock()
                            .map_err(|_| io::Error::other("signal socket lock poisoned"))? =
                            Some(bus.stream.try_clone()?);
                        for rule in RULES {
                            if stop.load(Ordering::Acquire) {
                                return Ok(());
                            }
                            subscribe(&mut bus, rule)?;
                        }
                        changed.store(true, Ordering::Release);
                        backoff = Duration::from_secs(1);
                        listen(&mut bus.stream, &changed, &stop)
                    })();
                    if let Ok(mut guard) = socket.lock() {
                        *guard = None;
                    }
                    if stop.load(Ordering::Acquire) {
                        break;
                    }
                    if result.is_err() {
                        changed.store(true, Ordering::Release);
                    }
                    let deadline = Instant::now() + backoff;
                    while Instant::now() < deadline && !stop.load(Ordering::Acquire) {
                        thread::sleep(TICK);
                    }
                    backoff = (backoff * 2).min(Duration::from_secs(30));
                }
            });
        monitor
    }
    pub(crate) fn take_changed(&self) -> bool {
        self.changed.swap(false, Ordering::AcqRel)
    }
}
impl Drop for HostSignalMonitor {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        // Interrupt reads immediately after authentication. If startup is still
        // authenticating, existing bus I/O timeouts bound the detached thread.
        if let Ok(guard) = self.socket.lock() {
            if let Some(socket) = guard.as_ref() {
                let _ = socket.shutdown(Shutdown::Both);
            }
        }
    }
}
fn subscribe(bus: &mut Bus, rule: &str) -> io::Result<()> {
    bus.serial = bus
        .serial
        .checked_add(1)
        .ok_or_else(|| io::Error::other("D-Bus serial exhausted"))?;
    let mut body = Vec::new();
    push_string(&mut body, rule);
    let mut call = build_method_call(
        bus.serial,
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus",
        "AddMatch",
        "org.freedesktop.DBus",
        Some("s"),
    )?;
    call[4..8].copy_from_slice(&(body.len() as u32).to_ne_bytes());
    call.extend_from_slice(&body);
    bus.stream.write_all(&call)?;
    bus.await_reply().map(|_| ())
}
fn relevant(message: &Message) -> bool {
    if message.kind != 4 {
        return false;
    }
    let interface = message.interface.as_deref();
    let member = message.member.as_deref();
    let path = message.path.as_deref().unwrap_or_default();
    match (interface, member) {
        (Some("org.freedesktop.DBus.Properties"), Some("PropertiesChanged")) => {
            path == "/org/bluez" || path.starts_with("/org/bluez/")
        }
        (
            Some("org.freedesktop.DBus.ObjectManager"),
            Some("InterfacesAdded" | "InterfacesRemoved"),
        ) => true,
        (Some("org.freedesktop.DBus"), Some("NameOwnerChanged")) => true,
        _ => false,
    }
}
/// Incremental decoding retains partial headers/bodies across WouldBlock. Bound
/// both each message and each iteration so a signal flood cannot starve stop.
fn consume(buffer: &mut Vec<u8>, changed: &AtomicBool) -> io::Result<()> {
    let mut offset = 0;
    while buffer.len() - offset >= 16 {
        let fixed = &buffer[offset..offset + 16];
        let little = match fixed[0] {
            b'l' => true,
            b'B' => false,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "bad signal byte order",
                ))
            }
        };
        let body = u32_at(&fixed[4..8], little) as usize;
        let fields = u32_at(&fixed[12..16], little) as usize;
        if fields > MAX_FIELDS || body > MAX_SIGNAL {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "signal size limit",
            ));
        }
        let total = (16 + fields).next_multiple_of(8) + body;
        if total > MAX_SIGNAL {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "signal size limit",
            ));
        }
        if buffer.len() - offset < total {
            break;
        }
        let message = read_message(&mut &buffer[offset..offset + total])?;
        if relevant(&message) {
            changed.store(true, Ordering::Release);
        }
        offset += total;
    }
    buffer.drain(..offset);
    Ok(())
}
fn listen(stream: &mut UnixStream, changed: &AtomicBool, stop: &AtomicBool) -> io::Result<()> {
    stream.set_nonblocking(true)?;
    let mut buffer = Vec::new();
    let mut bytes = [0; 4096];
    let mut partial_since = None;
    while !stop.load(Ordering::Acquire) {
        for _ in 0..16 {
            match stream.read(&mut bytes) {
                Ok(0) => {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "signal bus closed",
                    ))
                }
                Ok(count) => {
                    buffer.extend_from_slice(&bytes[..count]);
                    consume(&mut buffer, changed)?;
                    if buffer.is_empty() {
                        partial_since = None;
                    } else if partial_since.is_none() {
                        partial_since = Some(Instant::now());
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(error),
            }
        }
        if partial_since.is_some_and(|start| start.elapsed() > IO_TIMEOUT) {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "partial signal expired",
            ));
        }
        thread::sleep(TICK);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires an explicitly selected connected headset and live system bus"]
    fn live_system_bus_accepts_all_subscriptions_for_connected_target() {
        let address = BluetoothAddress::parse(
            &std::env::var("UGREEN_LIVE_ADDRESS").expect("set UGREEN_LIVE_ADDRESS explicitly"),
        )
        .unwrap();
        let snapshot = host_snapshot().unwrap();
        assert!(!snapshot.adapters.is_empty());
        assert!(snapshot
            .devices
            .iter()
            .any(|device| device.address == address && device.connected.is_connected()));
        let mut bus = Bus::open().unwrap();
        for rule in RULES {
            subscribe(&mut bus, rule).unwrap();
        }
        // This proves subscription acceptance only. It deliberately does not
        // change radio state to manufacture a PropertiesChanged signal.
    }
    fn signal(path: &str, interface: &str, member: &str) -> Vec<u8> {
        let mut bytes = build_method_call(1, path, interface, member, "org.bluez", None).unwrap();
        bytes[1] = 4;
        bytes
    }
    #[test]
    fn fragmented_and_coalesced_signals_only_invalidate_host_snapshots() {
        let relevant = signal(
            "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF",
            "org.freedesktop.DBus.Properties",
            "PropertiesChanged",
        );
        let unrelated = signal(
            "/org/bluez_fake/device",
            "org.freedesktop.DBus.Properties",
            "PropertiesChanged",
        );
        for split in 0..relevant.len() {
            let changed = AtomicBool::new(false);
            let mut buffer = relevant[..split].to_vec();
            consume(&mut buffer, &changed).unwrap();
            assert!(!changed.load(Ordering::Acquire));
            buffer.extend_from_slice(&relevant[split..]);
            buffer.extend_from_slice(&unrelated);
            consume(&mut buffer, &changed).unwrap();
            assert!(changed.swap(false, Ordering::AcqRel));
            assert!(buffer.is_empty());
            buffer.extend_from_slice(&unrelated);
            consume(&mut buffer, &changed).unwrap();
            assert!(!changed.load(Ordering::Acquire));
        }
    }
    #[test]
    fn subscription_marshals_match_rule_and_waits_for_its_reply() {
        let (client, mut server) = UnixStream::pair().unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(1)))
            .unwrap();
        let responder = thread::spawn(move || {
            let request = read_message(&mut server).unwrap();
            assert_eq!(request.interface.as_deref(), Some("org.freedesktop.DBus"));
            assert_eq!(request.member.as_deref(), Some("AddMatch"));
            assert_eq!(
                Reader::new(&request.body, request.little_endian)
                    .string()
                    .unwrap(),
                RULES[0]
            );
            let mut reply = vec![
                if cfg!(target_endian = "little") {
                    b'l'
                } else {
                    b'B'
                },
                2,
                0,
                1,
            ];
            push_u32_native(&mut reply, 0);
            push_u32_native(&mut reply, 1);
            push_u32_native(&mut reply, 8);
            reply.extend_from_slice(&[5, 1, b'u', 0]);
            push_u32_native(&mut reply, 41);
            server.write_all(&reply).unwrap();
        });
        let mut bus = Bus {
            stream: client,
            serial: 40,
        };
        subscribe(&mut bus, RULES[0]).unwrap();
        responder.join().unwrap();
    }
    #[test]
    fn dropping_monitor_interrupts_its_socket_and_requests_stop() {
        let (mut client, _server) = UnixStream::pair().unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(1)))
            .unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let monitor = HostSignalMonitor {
            changed: Arc::new(AtomicBool::new(false)),
            stop: stop.clone(),
            socket: Arc::new(Mutex::new(Some(client.try_clone().unwrap()))),
        };
        let reader = thread::spawn(move || client.read_exact(&mut [0; 1]));
        drop(monitor);
        assert!(stop.load(Ordering::Acquire));
        assert_eq!(
            reader.join().unwrap().unwrap_err().kind(),
            io::ErrorKind::UnexpectedEof
        );
    }
    #[test]
    fn oversized_signal_is_rejected_before_allocating_its_body() {
        let mut bytes = signal(
            "/org/bluez",
            "org.freedesktop.DBus.Properties",
            "PropertiesChanged",
        );
        bytes[4..8].copy_from_slice(&((MAX_SIGNAL + 1) as u32).to_ne_bytes());
        assert!(consume(&mut bytes, &AtomicBool::new(false)).is_err());
    }
    #[test]
    fn listener_stop_and_eof_are_bounded_without_a_system_bus() {
        let (mut reader, mut writer) = UnixStream::pair().unwrap();
        let changed = Arc::new(AtomicBool::new(false));
        let stop = Arc::new(AtomicBool::new(false));
        let observed = changed.clone();
        let stopping = stop.clone();
        let handle = thread::spawn(move || listen(&mut reader, &observed, &stopping));
        writer
            .write_all(&signal(
                "/org/bluez/hci0",
                "org.freedesktop.DBus.Properties",
                "PropertiesChanged",
            ))
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while !changed.load(Ordering::Acquire) && Instant::now() < deadline {
            thread::sleep(TICK);
        }
        assert!(changed.load(Ordering::Acquire));
        stop.store(true, Ordering::Release);
        assert!(handle.join().unwrap().is_ok());
        let (mut reader, writer) = UnixStream::pair().unwrap();
        drop(writer);
        assert_eq!(
            listen(
                &mut reader,
                &AtomicBool::new(false),
                &AtomicBool::new(false)
            )
            .unwrap_err()
            .kind(),
            io::ErrorKind::UnexpectedEof
        );
    }
}
