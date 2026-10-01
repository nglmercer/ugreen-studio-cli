use crate::{
    protocol::{self, Decoder, Response},
    settings::{DeviceInfo, Setting},
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
    pub fn request(&mut self, instruction: u8, payload: &[u8]) -> io::Result<Response> {
        self.io
            .write_all(&protocol::request(instruction, payload)?)?;
        self.io.flush()?;
        let deadline = Instant::now() + self.timeout;
        let mut bytes = [0; 512];
        loop {
            if Instant::now() >= deadline {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "no matching valid response before deadline",
                ));
            }
            match self.io.read(&mut bytes) {
                Ok(0) => {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "Bluetooth connection closed",
                    ))
                }
                Ok(n) => {
                    for frame in self.decoder.feed(&bytes[..n]) {
                        if frame.instruction == instruction {
                            if !frame.succeeded {
                                return Err(io::Error::other(format!(
                                    "headphones rejected instruction 0x{instruction:02X}"
                                )));
                            }
                            return Ok(frame);
                        }
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            }
        }
    }
    pub fn info(&mut self) -> io::Result<DeviceInfo> {
        DeviceInfo::new(self.request(protocol::INFO, &[0])?.payload)
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
    pub fn set(&mut self, setting: &Setting) -> io::Result<DeviceInfo> {
        self.request(setting.instruction,&setting.payload).map_err(|e| io::Error::new(e.kind(),format!("setting may have been sent but acknowledgement failed: {e}; query status before retrying")))?;
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
}
