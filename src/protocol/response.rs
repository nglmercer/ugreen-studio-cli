//! Headset-to-host response frames: `DD EE FF instruction success length
//! payload crc_lo crc_hi`, checksummed with CRC-16/CCITT-FALSE and
//! verified by the decoder before a frame is ever constructed.

/// The only bytes that may start a response candidate. Header and
/// length establish the candidate; only a matching checksum accepts it.
pub const RESPONSE_HEADER: [u8; 3] = [0xDD, 0xEE, 0xFF];

/// One verified response. The CRC has already been checked; `payload`
/// is exactly the bytes between `length` and the checksum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseFrame {
    pub instruction: u8,
    /// Nonzero counts as success; zero is rejection.
    pub succeeded: bool,
    pub payload: Vec<u8>,
}
