use crate::{
    i18n::Lang,
    protocol::{self, Decoder, IncomingFrame, ResponseFrame},
    settings::{Setting, StudioProState},
};
use std::io::{self, Read, Write};
use std::time::{Duration, Instant};

pub struct Client<T> {
    io: T,
    decoder: Decoder,
    timeout: Duration,
}
impl<T: Read + Write> Client<T> {
    pub fn new(io: T, timeout: Duration) -> Self {
        Self {
            io,
            decoder: Decoder::default(),
            timeout,
        }
    }
    pub fn request(&mut self, instruction: u8, payload: &[u8]) -> io::Result<ResponseFrame> {
        self.request_in(Lang::En, instruction, payload)
    }
    pub fn request_in(
        &mut self,
        lang: Lang,
        instruction: u8,
        payload: &[u8],
    ) -> io::Result<ResponseFrame> {
        let t = crate::i18n::txt(lang);
        self.io
            .write_all(&protocol::request(instruction, payload)?)?;
        self.io.flush()?;
        let deadline = Instant::now() + self.timeout;
        let mut bytes = [0; 512];
        loop {
            if Instant::now() >= deadline {
                return Err(io::Error::new(io::ErrorKind::TimedOut, t.req_timeout));
            }
            match self.io.read(&mut bytes) {
                Ok(0) => return Err(io::Error::new(io::ErrorKind::UnexpectedEof, t.req_closed)),
                Ok(n) => {
                    // Notifications (spatial echoes) and unknown frames
                    // are observed by the decoder but never satisfy a
                    // request; only a verified response to this exact
                    // instruction counts.
                    for frame in self.decoder.feed(&bytes[..n]) {
                        match frame {
                            IncomingFrame::Response(frame) if frame.instruction == instruction => {
                                if !frame.succeeded {
                                    return Err(io::Error::other(t.req_rejected.replacen(
                                        "{:02X}",
                                        &format!("{instruction:02X}"),
                                        1,
                                    )));
                                }
                                return Ok(frame);
                            }
                            IncomingFrame::Response(_)
                            | IncomingFrame::Notification(_)
                            | IncomingFrame::Unknown(_) => {}
                        }
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            }
        }
    }
    pub fn info(&mut self) -> io::Result<StudioProState> {
        StudioProState::new(self.request(protocol::INFO, &[0])?.payload)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
    pub fn firmware(&mut self) -> io::Result<String> {
        let response = self.request(protocol::FIRMWARE, &[0])?;
        crate::settings::firmware(&response.payload).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "firmware response lacks version bytes",
            )
        })
    }
    /// Only report success after a CRC-valid device-info readback matches.
    /// A failed acknowledgement is never silently retried, because writes may have taken effect.
    /// Spatial audio is the exception: retail firmware applies the write but
    /// answers with an `85 86 87` notification instead of a `DD EE FF`
    /// acknowledgement, so verification there relies on the readback alone.
    pub fn set(&mut self, setting: &Setting) -> io::Result<StudioProState> {
        if setting.expects_ack() {
            self.request(setting.instruction,&setting.payload).map_err(|e| io::Error::new(e.kind(),format!("setting may have been sent but acknowledgement failed: {e}; query status before retrying")))?;
        } else {
            self.io
                .write_all(&protocol::request(setting.instruction, &setting.payload)?)?;
            self.io.flush()?;
        }
        let info = self.info().map_err(|e| {
            io::Error::new(
                e.kind(),
                format!("write acknowledged but readback failed: {e}"),
            )
        })?;
        if !setting.matches(&info) {
            return Err(io::Error::other(format!(
                "write acknowledged but {} readback does not match {}; query status",
                setting.key, setting.value
            )));
        }
        Ok(info)
    }
    pub fn send(&mut self, setting: &Setting) -> io::Result<()> {
        self.io
            .write_all(&protocol::request(setting.instruction, &setting.payload)?)?;
        self.io.flush()
    }
    pub fn into_inner(self) -> T {
        self.io
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    #[derive(Default)]
    struct Mock {
        read: VecDeque<Vec<u8>>,
        written: Vec<u8>,
        short_write: bool,
    }
    impl Read for Mock {
        fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
            if let Some(data) = self.read.pop_front() {
                let n = data.len().min(b.len());
                b[..n].copy_from_slice(&data[..n]);
                if n < data.len() {
                    self.read.push_front(data[n..].to_vec());
                }
                Ok(n)
            } else {
                Err(io::Error::new(io::ErrorKind::TimedOut, "mock timeout"))
            }
        }
    }
    impl Write for Mock {
        fn write(&mut self, b: &[u8]) -> io::Result<usize> {
            let n = if self.short_write {
                b.len().min(2)
            } else {
                b.len()
            };
            self.written.extend_from_slice(&b[..n]);
            Ok(n)
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    fn response(instruction: u8, success: u8, payload: &[u8]) -> Vec<u8> {
        let mut b = vec![0xdd, 0xee, 0xff, instruction, success, payload.len() as u8];
        b.extend_from_slice(payload);
        b.extend_from_slice(&protocol::crc_ccitt(&b[3..]).to_le_bytes());
        b
    }
    #[test]
    fn partial_write_and_fragmented_response() {
        let raw = response(4, 1, &[20, 0, 0, 0xa0, 0, 1, 0, 0]);
        let mut c = Client::new(
            Mock {
                read: raw.chunks(2).map(|b| b.to_vec()).collect(),
                short_write: true,
                ..Default::default()
            },
            Duration::from_secs(1),
        );
        assert_eq!(c.info().unwrap().battery(), Some(20));
        assert_eq!(c.into_inner().written, protocol::request(4, &[0]).unwrap());
    }
    #[test]
    fn no_response_is_not_success() {
        let mut c = Client::new(Mock::default(), Duration::from_secs(1));
        assert!(c.info().is_err());
    }
    #[test]
    fn negative_ack_is_error() {
        let mut c = Client::new(
            Mock {
                read: vec![response(4, 0, &[])].into(),
                ..Default::default()
            },
            Duration::from_secs(1),
        );
        assert!(c.info().unwrap_err().to_string().contains("rejected"));
    }
    #[test]
    fn ignores_unrelated_and_bad_crc() {
        let mut bad = response(4, 1, &[20; 8]);
        bad[8] ^= 1;
        let good = response(4, 1, &[42; 8]);
        let mut c = Client::new(
            Mock {
                read: vec![[bad, response(5, 1, &[0]), good].concat()].into(),
                ..Default::default()
            },
            Duration::from_secs(1),
        );
        assert_eq!(c.info().unwrap().battery(), Some(42));
    }
    #[test]
    fn set_checks_actual_readback() {
        let mut payload = vec![0; 26];
        payload[6] = 1;
        let mut c = Client::new(
            Mock {
                read: vec![response(8, 1, &[1]), response(4, 1, &payload)].into(),
                ..Default::default()
            },
            Duration::from_secs(1),
        );
        assert!(c.set(&Setting::parse("game", "on").unwrap()).is_ok());
    }
    #[test]
    fn wrong_readback_fails() {
        let mut c = Client::new(
            Mock {
                read: vec![response(8, 1, &[1]), response(4, 1, &[0; 26])].into(),
                ..Default::default()
            },
            Duration::from_secs(1),
        );
        assert!(c
            .set(&Setting::parse("game", "on").unwrap())
            .unwrap_err()
            .to_string()
            .contains("does not match"));
    }
    #[test]
    fn spatial_write_verifies_via_readback_without_ack() {
        let mut payload = vec![0; 26];
        payload[20] = 1;
        let mut c = Client::new(
            Mock {
                read: vec![response(4, 1, &payload)].into(),
                ..Default::default()
            },
            Duration::from_secs(1),
        );
        let setting = Setting::parse("spatial", "on").unwrap();
        assert!(!setting.expects_ack());
        assert!(c.set(&setting).is_ok());
        let mut expected = protocol::request(18, &[1]).unwrap();
        expected.extend_from_slice(&protocol::request(protocol::INFO, &[0]).unwrap());
        assert_eq!(c.into_inner().written, expected);
    }

    #[test]
    fn notification_before_the_answer_never_blocks_the_request() {
        // Retail firmware answers spatial writes with `85 86 87 02 0A
        // <echo>`; that frame may reach the client while it waits for
        // the device-info response and must simply be observed.
        let mut payload = vec![0; 26];
        payload[20] = 1;
        let mut c = Client::new(
            Mock {
                read: vec![[
                    vec![0x85, 0x86, 0x87, 0x02, 0x0A, 0x01],
                    response(4, 1, &payload),
                ]
                .concat()]
                .into(),
                ..Default::default()
            },
            Duration::from_secs(1),
        );
        assert!(c.set(&Setting::parse("spatial", "on").unwrap()).is_ok());
    }
}
