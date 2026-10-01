//! Studio Pro HP206 framing, derived from the attributed upstream sources.
//! TX uses MODBUS; RX uses CCITT-FALSE, proven by the captured upstream fixture.
use std::io;

pub const INFO: u8 = 4;
pub const FIRMWARE: u8 = 1;
const RESPONSE_HEADER: [u8; 3] = [0xDD, 0xEE, 0xFF];

pub fn crc_modbus(data: &[u8]) -> u16 {
    let mut crc = 0xffffu16;
    for byte in data {
        crc ^= u16::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xa001
            } else {
                crc >> 1
            };
        }
    }
    crc
}
pub fn crc_ccitt(data: &[u8]) -> u16 {
    let mut crc = 0xffffu16;
    for byte in data {
        crc ^= u16::from(*byte) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 {
                (crc << 1) ^ 0x1021
            } else {
                crc << 1
            };
        }
    }
    crc
}
pub fn request(instruction: u8, payload: &[u8]) -> io::Result<Vec<u8>> {
    let length = u8::try_from(payload.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "payload exceeds 255 bytes"))?;
    let mut frame = vec![0xaa, 0xbb, 0xcc, instruction, length];
    frame.extend_from_slice(payload);
    frame.extend_from_slice(&crc_modbus(&frame[3..]).to_le_bytes());
    Ok(frame)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub instruction: u8,
    pub succeeded: bool,
    pub payload: Vec<u8>,
}

/// Stream parser: consumes concatenated or fragmented frames, rejects bad CRCs,
/// resynchronizes on the next response header, and bounds retained data to 263 bytes.
#[derive(Debug, Default)]
pub struct Decoder {
    buffer: Vec<u8>,
    pub rejected_checksums: usize,
    pub discarded_bytes: usize,
}
impl Decoder {
    pub fn feed(&mut self, input: &[u8]) -> Vec<Response> {
        let mut frames = Vec::new();
        // Processing byte-by-byte bounds storage even for arbitrarily large input.
        for byte in input {
            self.buffer.push(*byte);
            loop {
                if self.buffer.len() < 3 {
                    break;
                }
                if self.buffer[..3] != RESPONSE_HEADER {
                    self.buffer.remove(0);
                    self.discarded_bytes += 1;
                    continue;
                }
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
                    self.rejected_checksums += 1;
                    self.buffer.remove(0);
                    continue;
                }
                frames.push(Response {
                    instruction: self.buffer[3],
                    succeeded: self.buffer[4] != 0,
                    payload: self.buffer[6..length - 2].to_vec(),
                });
                self.buffer.drain(..length);
            }
        }
        frames
    }
    pub fn pending_bytes(&self) -> usize {
        self.buffer.len()
    }
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
    pub const CAPTURE: &str = "DD EE FF 04 01 1E 14 FF FF A0 00 01 00 00 08 0B 00 07 00 00 00 00 02 00 00 09 00 00 04 05 00 00 00 0C 0D 0E D0 E3";
    #[test]
    fn standard_crc_vectors() {
        assert_eq!(crc_modbus(b"123456789"), 0x4b37);
        assert_eq!(crc_ccitt(b"123456789"), 0x29b1);
    }
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
    fn captured_response_every_split() {
        let bytes = parse_hex(CAPTURE).unwrap();
        for cut in 0..=bytes.len() {
            let mut d = Decoder::default();
            let mut frames = d.feed(&bytes[..cut]);
            frames.extend(d.feed(&bytes[cut..]));
            assert_eq!(frames.len(), 1);
            assert_eq!(frames[0].instruction, INFO);
            assert_eq!(frames[0].payload.len(), 30);
            assert_eq!(frames[0].payload[0], 20);
            assert!(frames[0].succeeded);
            assert_eq!(d.pending_bytes(), 0);
        }
    }
    #[test]
    fn noise_corruption_and_concatenation() {
        let bytes = parse_hex(CAPTURE).unwrap();
        let mut corrupt = bytes.clone();
        corrupt[7] ^= 1;
        let input = [vec![0, 0xdd, 1, 0xdd, 0xee], corrupt, bytes.clone(), bytes].concat();
        let mut d = Decoder::default();
        assert_eq!(d.feed(&input).len(), 2);
        assert_eq!(d.rejected_checksums, 1);
        assert!(d.discarded_bytes > 0);
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
    fn hex_validation() {
        assert_eq!(parse_hex("aa:BB 00").unwrap(), [170, 187, 0]);
        for bad in ["abc", "GG", "é", "0x01"] {
            assert!(parse_hex(bad).is_err());
        }
    }
}
