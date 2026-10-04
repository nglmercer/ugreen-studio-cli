//! Unsolicited notification frames observed from retail firmware 0.2.5.
//!
//! Exactly two shapes have been seen on the wire, both six bytes:
//!
//! ```text
//! 85 86 87 02 0A <echo>     (spatial-audio write echo)
//! 85 86 87 02 05 00
//! ```
//!
//! Layout: three-byte magic, one `kind` byte (observed `0x02` in every
//! capture), two payload bytes. There is no length field and no
//! checksum, so [`NOTIFICATION_FRAME_LEN`] is fixed at six. That is an
//! observation, not a theory: no capture from the official UGREEN app
//! has yet shown a longer frame, and inventing a length or a meaning
//! would risk mis-splitting the stream. Bytes arriving after the sixth
//! are re-examined as new input by the decoder.

/// Three-byte notification magic. Shares no prefix with response
/// (`DD EE FF`) or request (`AA BB CC`) framing.
pub const NOTIFICATION_MAGIC: [u8; 3] = [0x85, 0x86, 0x87];
/// Six bytes, observed on every capture; see the module docs before
/// ever changing this.
pub const NOTIFICATION_FRAME_LEN: usize = 6;

/// One verified (checksum-less) notification frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationFrame {
    /// Byte 4 of the frame. `0x02` in every capture so far; any other
    /// value is kept verbatim rather than mapped to anything.
    pub kind: u8,
    /// Bytes 5–6: `0A <echo>` for spatial writes, `05 00` otherwise.
    pub payload: [u8; 2],
}

/// What a notification means to this application.
///
/// The mapping table is deliberately empty of real meanings: it holds
/// only unknowns until a capture from the official app is attributed
/// frame by frame. Semantics are never guessed from payload bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeadsetEvent {
    /// Recognised framing, unrecognised meaning.
    UnknownNotification(NotificationFrame),
}

impl NotificationFrame {
    /// The complete event mapping. Exhaustive on purpose: adding a
    /// meaning requires a capture, not a hunch.
    pub fn event(&self) -> HeadsetEvent {
        HeadsetEvent::UnknownNotification(self.clone())
    }
}
