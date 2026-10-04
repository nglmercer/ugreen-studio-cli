//! Raw session monitor: records every byte that crosses the vendor
//! control link, in order and with direction, for offline analysis.
//!
//! The monitor is a debug tap, not a protocol participant: it never
//! interprets or alters bytes. RX bytes can be replayed through the
//! regular [`Decoder`] to reconstruct the frame view, and [`Monitor::export`]
//! prints the session in the same whitespace-hex style as the files
//! under `tests/fixtures/protocol/`: RX bytes get `# rx` provenance
//! comments and their own hex lines, TX bytes are folded into `# tx`
//! comments, so stripping the comments leaves exactly the RX stream —
//! ready for `ugreen decode` or for review as a new fixture.

use super::{Decoder, DecoderStats, IncomingFrame};
use std::{
    io::{self, Read, Write},
    sync::{Arc, Mutex},
};

/// Which side of the link produced the bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Tx,
    Rx,
}

impl Direction {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Tx => "tx",
            Self::Rx => "rx",
        }
    }
}

/// One recorded chunk exactly as it appeared on the wire.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MonitorEvent {
    pub direction: Direction,
    pub bytes: Vec<u8>,
}

/// Ordered record of one control session.
#[derive(Clone, Debug, Default)]
pub struct Monitor {
    events: Vec<MonitorEvent>,
}

impl Monitor {
    /// Append a chunk. Empty chunks are dropped so an idle session
    /// exports as no events rather than a run of blank lines.
    pub fn record(&mut self, direction: Direction, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        self.events.push(MonitorEvent {
            direction,
            bytes: bytes.to_vec(),
        });
    }
    pub fn events(&self) -> &[MonitorEvent] {
        &self.events
    }
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
    pub fn total_bytes(&self) -> usize {
        self.events.iter().map(|event| event.bytes.len()).sum()
    }
    /// Replay every recorded RX chunk, in order, through a fresh
    /// [`Decoder`]. This mirrors what the live decoder saw and answers
    /// with the same frames plus the final statistics.
    pub fn decode_rx(&self) -> (Vec<IncomingFrame>, DecoderStats) {
        let mut decoder = Decoder::default();
        let mut frames = Vec::new();
        for event in &self.events {
            if event.direction == Direction::Rx {
                frames.extend(decoder.feed(&event.bytes));
            }
        }
        (frames, decoder.stats)
    }
    /// Fixture-style export of the whole session. TX bytes are folded
    /// into `# tx ...` provenance comments and RX bytes get a `# rx`
    /// comment plus their own hex line, so stripping the comment lines
    /// yields the RX stream alone — directly usable as the argument of
    /// `ugreen decode` and directly usable as a protocol fixture.
    pub fn export(&self) -> String {
        let mut out = String::new();
        for event in &self.events {
            match event.direction {
                Direction::Tx => {
                    out.push_str("# tx ");
                    out.push_str(&event.bytes.len().to_string());
                    out.push_str(" bytes: ");
                    out.push_str(&super::hex(&event.bytes));
                    out.push('\n');
                }
                Direction::Rx => {
                    out.push_str("# rx ");
                    out.push_str(&event.bytes.len().to_string());
                    out.push_str(" bytes\n");
                    out.push_str(&super::hex(&event.bytes));
                    out.push('\n');
                }
            }
        }
        out
    }
    /// RX bytes only, as raw hex with no comments: directly usable as
    /// the argument of `ugreen decode`.
    pub fn rx_hex(&self) -> String {
        let mut out = String::new();
        for event in &self.events {
            if event.direction == Direction::Rx {
                if !out.is_empty() {
                    out.push(' ');
                }
                out.push_str(&super::hex(&event.bytes));
            }
        }
        out
    }
}

/// Shared handle used by [`Monitored`] so the log can be read after
/// the connection is dropped.
pub type SharedMonitor = Arc<Mutex<Monitor>>;

pub fn shared_monitor() -> SharedMonitor {
    Arc::new(Mutex::new(Monitor::default()))
}

/// An IO wrapper that tees every byte through a [`SharedMonitor`].
/// Reads record RX, writes record TX, and the inner transport always
/// sees exactly what the caller sent.
#[derive(Debug)]
pub struct Monitored<I> {
    inner: I,
    log: SharedMonitor,
}

impl<I> Monitored<I> {
    pub fn new(inner: I, log: SharedMonitor) -> Self {
        Self { inner, log }
    }
    pub fn log(&self) -> &SharedMonitor {
        &self.log
    }
    pub fn into_inner_and_log(self) -> (I, SharedMonitor) {
        (self.inner, self.log)
    }
}

fn lock(log: &SharedMonitor) -> std::sync::MutexGuard<'_, Monitor> {
    log.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl<I: Read> Read for Monitored<I> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        lock(&self.log).record(Direction::Rx, &buf[..n]);
        Ok(n)
    }
}

impl<I: Write> Write for Monitored<I> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.inner.write(buf)?;
        lock(&self.log).record(Direction::Tx, &buf[..n]);
        Ok(n)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol;

    fn response(instruction: u8, payload: &[u8]) -> Vec<u8> {
        let mut b = vec![0xdd, 0xee, 0xff, instruction, 1, payload.len() as u8];
        b.extend_from_slice(payload);
        b.extend_from_slice(&protocol::crc_ccitt(&b[3..]).to_le_bytes());
        b
    }

    #[derive(Default)]
    struct Mock {
        read: VecDeque<Vec<u8>>,
        written: Vec<u8>,
    }
    use std::collections::VecDeque;
    impl Read for Mock {
        fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
            let Some(data) = self.read.pop_front() else {
                return Err(io::Error::new(io::ErrorKind::TimedOut, "mock timeout"));
            };
            let n = data.len().min(b.len());
            b[..n].copy_from_slice(&data[..n]);
            if n < data.len() {
                self.read.push_front(data[n..].to_vec());
            }
            Ok(n)
        }
    }
    impl Write for Mock {
        fn write(&mut self, b: &[u8]) -> io::Result<usize> {
            self.written.extend_from_slice(b);
            Ok(b.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn records_chunks_in_order_and_skips_empty_ones() {
        let mut monitor = Monitor::default();
        monitor.record(Direction::Tx, b"");
        monitor.record(Direction::Rx, &[1, 2]);
        monitor.record(Direction::Tx, &[3]);
        monitor.record(Direction::Rx, &[]);
        assert_eq!(monitor.events().len(), 2);
        assert_eq!(monitor.events()[0].direction, Direction::Rx);
        assert_eq!(monitor.events()[0].bytes, [1, 2]);
        assert_eq!(monitor.events()[1].direction, Direction::Tx);
        assert_eq!(monitor.total_bytes(), 3);
        assert!(!monitor.is_empty());
    }

    #[test]
    fn monitored_io_tees_every_byte_in_both_directions() {
        let log = shared_monitor();
        let mut io = Monitored::new(
            Mock {
                read: vec![response(4, &[20; 8])].into(),
                ..Default::default()
            },
            log.clone(),
        );
        let request = protocol::request(4, &[0]).unwrap();
        io.write_all(&request).unwrap();
        io.flush().unwrap();
        let mut buffer = [0; 64];
        let n = io.read(&mut buffer).unwrap();
        let (inner, log) = io.into_inner_and_log();
        assert_eq!(&buffer[..n], response(4, &[20; 8]).as_slice());
        assert_eq!(inner.written, request);
        let monitor = lock(&log);
        assert_eq!(monitor.events().len(), 2);
        assert_eq!(monitor.events()[0].direction, Direction::Tx);
        assert_eq!(monitor.events()[0].bytes, request);
        assert_eq!(monitor.events()[1].direction, Direction::Rx);
        assert_eq!(monitor.events()[1].bytes, &buffer[..n]);
    }

    #[test]
    fn decode_rx_replays_the_live_frame_view() {
        let mut monitor = Monitor::default();
        monitor.record(Direction::Tx, &protocol::request(4, &[0]).unwrap());
        monitor.record(Direction::Rx, &response(4, &[20; 8]));
        monitor.record(Direction::Rx, &[0x85, 0x86, 0x87, 0x02, 0x0A, 0x01]);
        monitor.record(Direction::Rx, &response(4, &[42; 8]));
        let (frames, stats) = monitor.decode_rx();
        assert_eq!(frames.len(), 3);
        assert!(matches!(frames[0], IncomingFrame::Response(_)));
        assert!(matches!(frames[1], IncomingFrame::Notification(_)));
        assert!(matches!(frames[2], IncomingFrame::Response(_)));
        assert_eq!(stats, DecoderStats::default());
    }

    #[test]
    fn export_is_fixture_style_and_the_rx_side_feeds_decode() {
        let mut monitor = Monitor::default();
        let request = protocol::request(4, &[0]).unwrap();
        let reply = response(4, &[20; 8]);
        monitor.record(Direction::Tx, &request);
        monitor.record(Direction::Rx, &reply);
        let export = monitor.export();
        assert!(
            export.contains("# tx 8 bytes: AA BB CC 04 01 00 31 91"),
            "{export}"
        );
        assert!(export.contains("# rx 16 bytes"), "{export}");
        // Stripping comments (as parse_hex does) yields the RX stream
        // alone: decode-ready and fixture-ready.
        assert_eq!(
            protocol::parse_hex(&export).unwrap(),
            reply,
            "export must decode as the recorded RX stream"
        );
        let mut decoder = Decoder::default();
        let frames = decoder.feed(&protocol::parse_hex(&export).unwrap());
        assert_eq!(frames.len(), 1);
        assert_eq!(decoder.stats, DecoderStats::default());
        // Programmatic raw view stays available without comments.
        assert_eq!(monitor.rx_hex(), protocol::hex(&reply));
    }
}
