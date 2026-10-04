//! The RFCOMM stream parser: fragments, concatenation and noise in,
//! verified frames out.
//!
//! Three frame shapes are recognised on the RX stream, in this order:
//!
//! 1. `DD EE FF ...` responses (CCITT-FALSE checksum verified).
//! 2. `85 86 87 ...` notifications (fixed six bytes, no checksum —
//!    see [`crate::protocol::notification`]).
//! 3. `AA BB CC ...` request layouts arriving backwards: parsed with
//!    the TX framing rules (MODBUS checksum verified) but reported as
//!    [`IncomingFrame::Unknown`], because a headset should never send
//!    requests and no capture has verified what these would mean.
//!
//! Anything else is discarded byte by byte and counted. Storage never
//! exceeds 263 bytes: the largest candidate (a 255-byte response) plus
//! resynchronisation slack.

use super::{
    crc::{crc_ccitt, crc_modbus},
    notification::{NotificationFrame, NOTIFICATION_FRAME_LEN, NOTIFICATION_MAGIC},
    response::{ResponseFrame, RESPONSE_HEADER},
};

/// Bytes that start a host-to-headset frame; never legitimate on RX.
const TX_HEADER: [u8; 3] = [0xAA, 0xBB, 0xCC];

/// Counters describing everything the decoder saw but could not turn
/// into a verified response. The offline `decode` command surfaces all
/// of them; a capture that trips any counter is not treated as clean.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DecoderStats {
    /// `DD EE FF` candidates whose CCITT checksum failed.
    pub response_crc_failures: usize,
    /// Bytes skipped while hunting for a frame start, plus failed
    /// request-layout candidates on RX.
    pub discarded_bytes: usize,
    /// Well-framed RX bytes with no verified meaning (`Unknown`).
    pub unknown_frames: usize,
}

/// One frame decoded from the RX stream.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IncomingFrame {
    Response(ResponseFrame),
    Notification(NotificationFrame),
    /// A well-framed byte sequence with no verified RX meaning.
    Unknown(UnknownFrame),
}

/// A checksum-valid request-layout frame (`AA BB CC ...`) read from
/// the RX stream. Kept verbatim; nothing interprets it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnknownFrame {
    /// Instruction byte — untrusted, since headsets should not send these.
    pub instruction: u8,
    pub payload: Vec<u8>,
}

/// Stream parser: consumes concatenated or fragmented frames, rejects
/// bad CRCs, resynchronises on the next frame header, and bounds
/// retained data to 263 bytes.
#[derive(Debug, Default)]
pub struct Decoder {
    buffer: Vec<u8>,
    pub stats: DecoderStats,
}

impl Decoder {
    pub fn feed(&mut self, input: &[u8]) -> Vec<IncomingFrame> {
        let mut frames = Vec::new();
        // Processing byte-by-byte bounds storage even for arbitrarily large input.
        for byte in input {
            self.buffer.push(*byte);
            loop {
                if self.buffer.len() < 3 {
                    break;
                }
                if self.buffer[..3] == RESPONSE_HEADER {
                    if self.buffer.len() < 6 {
                        break;
                    }
                    let length = usize::from(self.buffer[5]) + 8;
                    if self.buffer.len() < length {
                        break;
                    }
                    let received =
                        u16::from_le_bytes([self.buffer[length - 2], self.buffer[length - 1]]);
                    if crc_ccitt(&self.buffer[3..length - 2]) != received {
                        self.stats.response_crc_failures += 1;
                        self.buffer.remove(0);
                        continue;
                    }
                    frames.push(IncomingFrame::Response(ResponseFrame {
                        instruction: self.buffer[3],
                        succeeded: self.buffer[4] != 0,
                        payload: self.buffer[6..length - 2].to_vec(),
                    }));
                    self.buffer.drain(..length);
                    continue;
                }
                if self.buffer[..3] == NOTIFICATION_MAGIC {
                    if self.buffer.len() < NOTIFICATION_FRAME_LEN {
                        break;
                    }
                    frames.push(IncomingFrame::Notification(NotificationFrame {
                        kind: self.buffer[3],
                        payload: [self.buffer[4], self.buffer[5]],
                    }));
                    self.buffer.drain(..NOTIFICATION_FRAME_LEN);
                    continue;
                }
                if self.buffer[..3] == TX_HEADER {
                    if self.buffer.len() < 5 {
                        break;
                    }
                    let length = 7 + usize::from(self.buffer[4]);
                    if self.buffer.len() < length {
                        break;
                    }
                    let received =
                        u16::from_le_bytes([self.buffer[length - 2], self.buffer[length - 1]]);
                    if crc_modbus(&self.buffer[3..length - 2]) != received {
                        // Not a frame anyone should have sent: drop a
                        // byte and resynchronise, never fabricate one.
                        self.stats.discarded_bytes += 1;
                        self.buffer.remove(0);
                        continue;
                    }
                    frames.push(IncomingFrame::Unknown(UnknownFrame {
                        instruction: self.buffer[3],
                        payload: self.buffer[5..length - 2].to_vec(),
                    }));
                    self.stats.unknown_frames += 1;
                    self.buffer.drain(..length);
                    continue;
                }
                self.buffer.remove(0);
                self.stats.discarded_bytes += 1;
            }
        }
        frames
    }
    pub fn pending_bytes(&self) -> usize {
        self.buffer.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{hex, parse_hex, HeadsetEvent};

    const CAPTURE: &str = "DD EE FF 04 01 1E 14 FF FF A0 00 01 00 00 08 0B 00 07 00 00 00 00 02 00 00 09 00 00 04 05 00 00 00 0C 0D 0E D0 E3";
    /// The two notification shapes recorded in `docs/protocol.md`.
    const NOTIFICATION_ECHO: [u8; 6] = [0x85, 0x86, 0x87, 0x02, 0x0A, 0x01];
    const NOTIFICATION_DONE: [u8; 6] = [0x85, 0x86, 0x87, 0x02, 0x05, 0x00];

    #[test]
    fn captured_response_every_split() {
        let bytes = parse_hex(CAPTURE).unwrap();
        for cut in 0..=bytes.len() {
            let mut d = Decoder::default();
            let mut frames = d.feed(&bytes[..cut]);
            frames.extend(d.feed(&bytes[cut..]));
            assert_eq!(frames.len(), 1);
            let IncomingFrame::Response(frame) = &frames[0] else {
                panic!("expected a response, got {frames:?}");
            };
            assert_eq!(frame.instruction, 4);
            assert_eq!(frame.payload.len(), 30);
            assert_eq!(frame.payload[0], 20);
            assert!(frame.succeeded);
            assert_eq!(d.pending_bytes(), 0);
            assert_eq!(d.stats, DecoderStats::default());
        }
    }

    #[test]
    fn noise_corruption_and_concatenation() {
        let bytes = parse_hex(CAPTURE).unwrap();
        let mut corrupt = bytes.clone();
        corrupt[7] ^= 1;
        let input = [vec![0, 0xdd, 1, 0xdd, 0xee], corrupt, bytes.clone(), bytes].concat();
        let mut d = Decoder::default();
        let frames = d.feed(&input);
        assert_eq!(
            frames
                .iter()
                .filter(|f| matches!(f, IncomingFrame::Response(_)))
                .count(),
            2
        );
        assert_eq!(d.stats.response_crc_failures, 1);
        assert!(d.stats.discarded_bytes > 0);
        assert_eq!(d.stats.unknown_frames, 0);
    }

    #[test]
    fn retains_split_header() {
        let mut d = Decoder::default();
        assert!(d.feed(&[1, 2, 3, 0xdd, 0xee]).is_empty());
        let bytes = parse_hex(CAPTURE).unwrap();
        assert_eq!(d.feed(&bytes[2..]).len(), 1);
    }

    #[test]
    fn bounded_noise() {
        let mut d = Decoder::default();
        d.feed(&vec![0; 100_000]);
        assert!(d.pending_bytes() < 3);
    }

    #[test]
    fn maximum_frame_and_arbitrary_bytes_are_bounded() {
        let mut frame = vec![0xdd, 0xee, 0xff, 4, 1, 255];
        frame.extend_from_slice(&[42; 255]);
        frame.extend_from_slice(&crc_ccitt(&frame[3..]).to_le_bytes());
        let mut d = Decoder::default();
        assert_eq!(d.feed(&frame).len(), 1);
        let mut x = 0x12345678u32;
        for _ in 0..10_000 {
            x = x.wrapping_mul(1664525).wrapping_add(1013904223);
            d.feed(&x.to_le_bytes());
            assert!(d.pending_bytes() <= 263);
        }
    }

    #[test]
    fn notifications_decode_without_semantics() {
        let mut d = Decoder::default();
        let mut input = NOTIFICATION_ECHO.to_vec();
        input.extend_from_slice(&NOTIFICATION_DONE);
        let frames = d.feed(&input);
        assert_eq!(frames.len(), 2);
        for frame in &frames {
            let IncomingFrame::Notification(frame) = frame else {
                panic!("expected notifications, got {frame:?}");
            };
            assert_eq!(frame.kind, 0x02);
            assert_eq!(
                frame.event(),
                HeadsetEvent::UnknownNotification(frame.clone())
            );
        }
        let IncomingFrame::Notification(first) = &frames[0] else {
            unreachable!()
        };
        assert_eq!(first.payload, [0x0A, 0x01]);
        let IncomingFrame::Notification(second) = &frames[1] else {
            unreachable!()
        };
        assert_eq!(second.payload, [0x05, 0x00]);
        // Recognised framing is not discarded, rejected or unknown.
        assert_eq!(d.stats, DecoderStats::default());
        assert_eq!(d.pending_bytes(), 0);
    }

    #[test]
    fn notification_splits_are_reassembled_byte_by_byte() {
        for cut in 0..=NOTIFICATION_ECHO.len() {
            let mut d = Decoder::default();
            let mut frames = d.feed(&NOTIFICATION_ECHO[..cut]);
            frames.extend(d.feed(&NOTIFICATION_ECHO[cut..]));
            assert_eq!(frames.len(), 1, "cut at {cut}");
            assert_eq!(d.pending_bytes(), 0);
        }
    }

    #[test]
    fn notifications_between_responses_do_not_disturb_requests() {
        // The live client sees this mix after a spatial write: the
        // answer must still be found while the echo is observed.
        let answer = parse_hex(CAPTURE).unwrap();
        let mut input = answer.clone();
        input.extend_from_slice(&NOTIFICATION_ECHO);
        input.extend_from_slice(&answer);
        let mut d = Decoder::default();
        let frames = d.feed(&input);
        assert_eq!(frames.len(), 3);
        assert!(matches!(frames[0], IncomingFrame::Response(_)));
        assert!(matches!(frames[1], IncomingFrame::Notification(_)));
        assert!(matches!(frames[2], IncomingFrame::Response(_)));
        assert_eq!(d.stats, DecoderStats::default());
    }

    #[test]
    fn request_layouts_on_rx_become_unknown_not_responses() {
        // AA BB CC frame built exactly like a host request.
        let mut bytes = vec![0xaa, 0xbb, 0xcc, 0x11, 0x02, 0xAB, 0xCD];
        bytes.extend_from_slice(&crc_modbus(&bytes[3..]).to_le_bytes());
        let mut d = Decoder::default();
        let frames = d.feed(&bytes);
        assert_eq!(frames.len(), 1);
        let IncomingFrame::Unknown(frame) = &frames[0] else {
            panic!("expected unknown, got {frames:?}");
        };
        assert_eq!(frame.instruction, 0x11);
        assert_eq!(frame.payload, [0xAB, 0xCD]);
        assert_eq!(d.stats.unknown_frames, 1);
        assert_eq!(d.stats.response_crc_failures, 0);
        assert_eq!(d.stats.discarded_bytes, 0);
        // It never satisfies response-shaped expectations.
        assert!(!matches!(frames[0], IncomingFrame::Response(_)));
    }

    #[test]
    fn request_layouts_with_bad_checksum_are_discarded_not_framed() {
        let mut bytes = vec![0xaa, 0xbb, 0xcc, 0x11, 0x02, 0xAB, 0xCD, 0x00, 0x00];
        bytes[8] ^= 0xFF;
        let mut d = Decoder::default();
        let frames = d.feed(&bytes);
        assert!(frames.is_empty());
        assert_eq!(d.stats.unknown_frames, 0);
        assert!(d.stats.discarded_bytes > 0);
    }

    #[test]
    fn trailing_partial_notification_stays_pending_not_guessed() {
        let mut d = Decoder::default();
        assert!(d.feed(&NOTIFICATION_ECHO[..5]).is_empty());
        assert_eq!(d.pending_bytes(), 5);
        assert_eq!(
            d.feed(&NOTIFICATION_ECHO[5..]).len(),
            1,
            "the sixth byte completes the frame"
        );
        assert_eq!(d.pending_bytes(), 0);
    }

    #[test]
    fn hex_roundtrip_matches_the_capture() {
        let bytes = parse_hex(CAPTURE).unwrap();
        assert_eq!(hex(&bytes), CAPTURE);
    }
}
