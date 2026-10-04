//! Studio Pro HP206 framing, derived from the attributed upstream sources.
//! TX uses MODBUS; RX uses CCITT-FALSE, proven by the captured upstream fixture.
//!
//! The RX stream carries three frame shapes, decoded into
//! [`IncomingFrame`]: responses, fixed six-byte notifications, and
//! well-framed bytes with no verified meaning. Everything else is
//! counted and skipped, never reinterpreted as status.

pub mod crc;
pub mod decoder;
pub mod notification;
pub mod response;

pub use crc::{crc_ccitt, crc_modbus};
pub use decoder::{Decoder, DecoderStats, IncomingFrame, UnknownFrame};
pub use notification::{
    HeadsetEvent, NotificationFrame, NOTIFICATION_FRAME_LEN, NOTIFICATION_MAGIC,
};
pub use response::ResponseFrame;

use std::io;

pub const INFO: u8 = 4;
pub const FIRMWARE: u8 = 1;

pub fn request(instruction: u8, payload: &[u8]) -> io::Result<Vec<u8>> {
    let length = u8::try_from(payload.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "payload exceeds 255 bytes"))?;
    let mut frame = vec![0xaa, 0xbb, 0xcc, instruction, length];
    frame.extend_from_slice(payload);
    frame.extend_from_slice(&crc_modbus(&frame[3..]).to_le_bytes());
    Ok(frame)
}

pub fn parse_hex(text: &str) -> Result<Vec<u8>, String> {
    let compact: String = text
        .chars()
        .filter(|c| !c.is_ascii_whitespace() && *c != ':')
        .collect();
    if !compact.len().is_multiple_of(2) || !compact.is_ascii() {
        return Err("hex must contain an even number of ASCII hexadecimal digits".into());
    }
    compact
        .as_bytes()
        .chunks(2)
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16)
                .map_err(|_| "invalid hexadecimal digit".into())
        })
        .collect()
}

pub fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upstream_device_query() {
        assert_eq!(
            hex(&request(INFO, &[0]).unwrap()),
            "AA BB CC 04 01 00 31 91"
        );
    }

    #[test]
    fn oversized_payload_rejected() {
        assert!(request(1, &[0; 256]).is_err());
    }

    #[test]
    fn hex_validation() {
        assert_eq!(parse_hex("aa:BB 00").unwrap(), [170, 187, 0]);
        for bad in ["abc", "GG", "é", "0x01"] {
            assert!(parse_hex(bad).is_err());
        }
    }
}
