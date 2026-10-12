use crate::{
    i18n::Lang,
    multipoint::{MultipointPeer, UnsupportedFeature},
    protocol::{self, Decoder, IncomingFrame, ResponseFrame},
    settings::{Setting, StudioProState},
};
use std::io::{self, Read, Write};
use std::time::{Duration, Instant};

/// Evidence returned after a control command. An acknowledgement alone
/// does not prove the resulting device state; finder sends have neither.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlOutcome {
    Readback(StudioProState),
    Acknowledged(ResponseFrame),
    SentUnconfirmed,
}

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
    /// Execute a typed control only after the model capability is promoted.
    /// Writes are never retried implicitly, including LDAC and resets.
    pub fn control(
        &mut self,
        command: &protocol::static_analysis::Command,
    ) -> io::Result<ControlOutcome> {
        command.check_control()?;
        self.control_verified(command)
    }
    fn control_verified(
        &mut self,
        command: &protocol::static_analysis::Command,
    ) -> io::Result<ControlOutcome> {
        use protocol::static_analysis::Command as C;
        let packet = command.packet()?;
        // No packet is sent for a query passed to the control API.
        if command.control_capability().is_none() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "query is not a control",
            ));
        }
        let instruction = packet[3];
        let payload = &packet[5..packet.len() - 2];
        let send_only = matches!(command, C::Buttons(_) | C::Find { .. });
        let ack = if send_only {
            self.io.write_all(&packet).and_then(|_| self.io.flush()).map_err(|e|
                io::Error::new(e.kind(), format!("control may have been partially sent: {e}; inspect state before retrying")))?;
            None
        } else {
            let ack = self.request(instruction, payload).map_err(|e| io::Error::new(e.kind(), format!("control may have been sent but acknowledgement failed: {e}; inspect state before retrying")))?;
            if !matches!(command, C::FactoryReset) {
                let expected = match command {
                    C::AiActions(_) => payload,
                    _ => &payload[..1],
                };
                if ack.payload != expected {
                    return Err(invalid_data(
                        "control acknowledged with mismatched value; inspect state before retrying",
                    ));
                }
            }
            Some(ack)
        };
        if matches!(command, C::Find { .. }) {
            // The verified six-byte decoder lacks the finder-right byte.
            // A sent control cannot claim a confirmed finder state or audio.
            return Ok(ControlOutcome::SentUnconfirmed);
        }
        let readback = match command {
            C::Buttons(_) => Some((8, payload, false)),
            C::AiActions(_) => Some((26, payload, false)),
            C::Volume(_) => Some((19, &payload[..1], false)),
            C::AiControl(_) => Some((30, &payload[..1], false)),
            C::SharingOff | C::SharingSinglePeer | C::SharingPeer { .. } => {
                Some((24, &payload[..1], payload[0] == 1))
            }
            _ => None,
        };
        if let Some((offset, expected, wind_off)) = readback {
            let info = self.info().map_err(|e| {
                io::Error::new(
                    e.kind(),
                    format!("control sent but readback failed: {e}; inspect state before retrying"),
                )
            })?;
            if info.raw.get(offset..offset + expected.len()) != Some(expected)
                || (wind_off && info.raw.get(25) != Some(&0))
            {
                return Err(invalid_data(
                    "control readback does not match; inspect state before retrying",
                ));
            }
            return Ok(ControlOutcome::Readback(info));
        }
        Ok(ControlOutcome::Acknowledged(ack.expect(
            "non-finder controls without readback require ack",
        )))
    }
    /// Read all headset peer slots, including UTF-8 names and slot indices.
    /// Static evidence is insufficient: the model gate is checked before I/O.
    pub fn peers(&mut self) -> io::Result<protocol::static_analysis::PeerList> {
        Self::require_capture(
            crate::models::Capabilities::MULTIPOINT_PEERS,
            "multipoint peer list",
        )?;
        self.collect_peers()
    }
    pub fn multipoint_peers(&mut self) -> io::Result<Vec<MultipointPeer>> {
        Ok(self
            .peers()?
            .slots
            .into_iter()
            .flatten()
            .map(|peer| MultipointPeer {
                address: peer.address,
                connected: true,
            })
            .collect())
    }
    fn require_capture(bit: u32, feature: &str) -> io::Result<()> {
        if crate::models::Model::StudioPro.capabilities().contains(bit) {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("{feature} is not verified for Studio Pro; official-app capture required"),
            ))
        }
    }
    pub fn codecs(&mut self) -> io::Result<[(u8, protocol::static_analysis::Codec); 2]> {
        Self::require_capture(crate::models::Capabilities::CODEC_STATUS, "codec status")?;
        match protocol::static_analysis::interpret(&self.request(15, &[0])?)
            .map_err(invalid_data)?
        {
            protocol::static_analysis::Response::Codecs(codecs) => Ok(codecs),
            _ => Err(invalid_data("unexpected codec response")),
        }
    }
    pub fn headphone_id(&mut self) -> io::Result<u8> {
        self.query_byte(crate::models::Capabilities::HEADPHONE_ID, "headphone ID", 2)
    }
    pub fn headphone_color(&mut self) -> io::Result<u8> {
        self.query_byte(
            crate::models::Capabilities::HEADPHONE_COLOR,
            "headphone color",
            3,
        )
    }
    fn query_byte(&mut self, bit: u32, feature: &str, instruction: u8) -> io::Result<u8> {
        Self::require_capture(bit, feature)?;
        match protocol::static_analysis::interpret(&self.request(instruction, &[0])?)
            .map_err(invalid_data)?
        {
            protocol::static_analysis::Response::Byte { value, .. } => Ok(value),
            _ => Err(invalid_data("unexpected byte response")),
        }
    }
    pub fn box_mac(&mut self) -> io::Result<crate::bluetooth::BluetoothAddress> {
        Self::require_capture(crate::models::Capabilities::BOX_MAC, "box MAC")?;
        match protocol::static_analysis::interpret(&self.request(31, &[0])?)
            .map_err(invalid_data)?
        {
            protocol::static_analysis::Response::BoxMac(address) => Ok(address),
            _ => Err(invalid_data("unexpected box MAC response")),
        }
    }
    fn collect_peers(&mut self) -> io::Result<protocol::static_analysis::PeerList> {
        use protocol::static_analysis::{PeerList, PeerUpdate};
        self.io.write_all(&protocol::request(13, &[0])?)?;
        self.io.flush()?;
        let deadline = Instant::now() + self.timeout;
        let mut peers = PeerList::default();
        let mut bytes = [0; 512];
        loop {
            if Instant::now() >= deadline {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "peer list incomplete; no empty list inferred",
                ));
            }
            match self.io.read(&mut bytes) {
                Ok(0) => {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "connection closed before complete peer list",
                    ))
                }
                Ok(n) => {
                    // Process the whole batch, never drop a coalesced second peer.
                    for frame in self.decoder.feed(&bytes[..n]) {
                        if let IncomingFrame::Response(frame) = frame {
                            if frame.instruction != 13 {
                                continue;
                            }
                            if !frame.succeeded {
                                return Err(io::Error::other("peer query rejected"));
                            }
                            let update = PeerUpdate::parse(&frame.payload).map_err(invalid_data)?;
                            peers.apply(update).map_err(invalid_data)?;
                        }
                    }
                    if peers.complete {
                        return Ok(peers);
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            }
        }
    }
    /// Refused before any write, like [`Self::multipoint_peers`].
    pub fn peer_disconnect(&mut self, _peer: &MultipointPeer) -> io::Result<()> {
        Err(UnsupportedFeature::PeerDisconnect.into())
    }
    /// Refused before any write, like [`Self::multipoint_peers`].
    pub fn peer_reconnect(&mut self, _peer: &MultipointPeer) -> io::Result<()> {
        Err(UnsupportedFeature::PeerReconnect.into())
    }
    /// Refused before any write, like [`Self::multipoint_peers`].
    pub fn peer_switch(&mut self, _peer: &MultipointPeer) -> io::Result<()> {
        Err(UnsupportedFeature::PeerSwitch.into())
    }
    pub fn into_inner(self) -> T {
        self.io
    }
}
fn invalid_data(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
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

    #[test]
    fn peer_operations_are_refused_before_any_byte_leaves() {
        let mut c = Client::new(Mock::default(), Duration::from_secs(1));
        let peer = MultipointPeer {
            address: crate::bluetooth::BluetoothAddress::parse("11:22:33:44:55:66").unwrap(),
            connected: true,
        };
        for result in [
            c.multipoint_peers().map(|_| ()),
            c.peer_disconnect(&peer),
            c.peer_reconnect(&peer),
            c.peer_switch(&peer),
        ] {
            let error = result.expect_err("unverified peer operations must refuse");
            assert_eq!(error.kind(), io::ErrorKind::Unsupported, "{error}");
            assert!(error.to_string().contains("not verified"), "{error}");
        }
        // Nothing was written, so nothing needs a reply either.
        assert!(c.into_inner().written.is_empty());
    }
    #[test]
    fn static_queries_refuse_before_io() {
        let mut c = Client::new(Mock::default(), Duration::from_secs(1));
        for result in [
            c.peers().map(|_| ()),
            c.codecs().map(|_| ()),
            c.headphone_id().map(|_| ()),
            c.headphone_color().map(|_| ()),
            c.box_mac().map(|_| ()),
        ] {
            assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Unsupported);
        }
        assert!(c.into_inner().written.is_empty());
    }
    #[test]
    fn peer_collection_handles_fragmented_and_coalesced_responses() {
        let a = response(13, 1, &[0, 1, b'A', 1, 2, 3, 4, 5, 6]);
        let b = response(13, 1, &[0, 2, b'B', 6, 5, 4, 3, 2, 1]);
        let stream = [vec![0x85, 0x86, 0x87, 2, 5, 0], a, b].concat();
        for chunks in [
            stream.chunks(2).map(|b| b.to_vec()).collect(),
            vec![stream.clone()].into(),
        ] {
            let mut c = Client::new(
                Mock {
                    read: chunks,
                    short_write: true,
                    ..Default::default()
                },
                Duration::from_secs(1),
            );
            let peers = c.collect_peers().unwrap();
            assert_eq!(peers.slots[0].as_ref().unwrap().name, "A");
            assert_eq!(
                peers.slots[1].as_ref().unwrap().address.octets(),
                [6, 5, 4, 3, 2, 1]
            );
            assert_eq!(c.into_inner().written, protocol::request(13, &[0]).unwrap());
        }
    }
    #[test]
    fn peer_collection_requires_explicit_completion() {
        for data in [
            response(13, 1, &[0, 1, 1, 2, 3, 4, 5, 6]),
            response(13, 0, &[]),
            response(13, 1, &[0, 3]),
        ] {
            let mut c = Client::new(
                Mock {
                    read: vec![data].into(),
                    ..Default::default()
                },
                Duration::from_secs(1),
            );
            assert!(c.collect_peers().is_err());
        }
        for data in [
            response(13, 1, &[0, 0]),
            response(13, 1, &[1, 1, 1, 2, 3, 4, 5, 6]),
        ] {
            let mut c = Client::new(
                Mock {
                    read: vec![data].into(),
                    ..Default::default()
                },
                Duration::from_secs(1),
            );
            assert!(c.collect_peers().unwrap().observed);
        }
    }
    #[test]
    fn all_static_controls_are_gated_before_io() {
        use protocol::static_analysis::{Action, Command as C, Side};
        let mac = crate::bluetooth::BluetoothAddress::from_octets([1; 6]);
        let commands = [
            C::Buttons([Action::None; 8]),
            C::AiActions([Action::None; 4]),
            C::Ldac {
                enabled: true,
                peer: None,
            },
            C::Ldac {
                enabled: true,
                peer: Some(mac.clone()),
            },
            C::HighQualityDecoding(true),
            C::Volume(8),
            C::Stereo(true),
            C::AiControl(true),
            C::SharingOff,
            C::SharingSinglePeer,
            C::SharingPeer {
                index: 2,
                address: mac,
            },
            C::FactoryReset,
            C::Find {
                enabled: true,
                side: Side::Left,
            },
            C::Find {
                enabled: false,
                side: Side::Right,
            },
        ];
        let mut c = Client::new(Mock::default(), Duration::from_secs(1));
        for command in commands {
            assert_eq!(
                c.control(&command).unwrap_err().kind(),
                io::ErrorKind::Unsupported
            );
        }
        assert!(c.into_inner().written.is_empty());
    }
    #[test]
    fn controls_validate_echo_and_never_retry() {
        use protocol::static_analysis::Command as C;
        for command in [
            C::Ldac {
                enabled: true,
                peer: None,
            },
            C::HighQualityDecoding(true),
            C::Stereo(true),
        ] {
            let instruction = command.packet().unwrap()[3];
            let mut c = Client::new(
                Mock {
                    read: vec![response(instruction, 1, &[1])].into(),
                    ..Default::default()
                },
                Duration::from_secs(1),
            );
            assert!(matches!(
                c.control_verified(&command).unwrap(),
                ControlOutcome::Acknowledged(_)
            ));
            assert_eq!(c.into_inner().written, command.packet().unwrap());
            for read in [
                vec![response(instruction, 1, &[0])].into(),
                vec![response(instruction, 0, &[1])].into(),
                std::collections::VecDeque::new(),
            ] {
                let mut c = Client::new(
                    Mock {
                        read,
                        ..Default::default()
                    },
                    Duration::from_secs(1),
                );
                assert!(c.control_verified(&command).is_err());
                assert_eq!(c.into_inner().written, command.packet().unwrap());
            }
        }
    }
    #[test]
    fn controls_require_readback_and_sharing_wind_off() {
        use protocol::static_analysis::{Action, Command as C};
        for (command, offset, bytes, ack) in [
            (C::Buttons([Action::Next; 8]), 8, vec![4; 8], false),
            (
                C::AiActions([Action::VoiceAssistant; 4]),
                26,
                vec![6; 4],
                true,
            ),
            (C::Volume(8), 19, vec![8], true),
            (C::AiControl(true), 30, vec![1], true),
            (C::SharingSinglePeer, 24, vec![1], true),
        ] {
            for correct in [true, false] {
                let mut state = vec![0; 31];
                if correct {
                    state[offset..offset + bytes.len()].copy_from_slice(&bytes);
                }
                let instruction = command.packet().unwrap()[3];
                let mut read = VecDeque::new();
                if ack {
                    read.push_back(response(instruction, 1, &bytes));
                }
                read.push_back(response(4, 1, &state));
                let mut c = Client::new(
                    Mock {
                        read,
                        ..Default::default()
                    },
                    Duration::from_secs(1),
                );
                assert_eq!(c.control_verified(&command).is_ok(), correct, "{command:?}");
            }
        }
        let mut state = vec![0; 31];
        state[24] = 1;
        state[25] = 1;
        let mut c = Client::new(
            Mock {
                read: vec![response(22, 1, &[1]), response(4, 1, &state)].into(),
                ..Default::default()
            },
            Duration::from_secs(1),
        );
        assert!(c.control_verified(&C::SharingSinglePeer).is_err());
    }
    #[test]
    fn finder_send_does_not_claim_audio_or_state_and_queries_never_write() {
        use protocol::static_analysis::{Command as C, Side};
        let mut c = Client::new(Mock::default(), Duration::from_secs(1));
        assert!(c.control_verified(&C::Codec).is_err());
        assert!(c.io.written.is_empty());
        let command = C::Find {
            enabled: true,
            side: Side::Left,
        };
        assert_eq!(
            c.control_verified(&command).unwrap(),
            ControlOutcome::SentUnconfirmed
        );
        assert_eq!(c.into_inner().written, command.packet().unwrap());
    }
    #[test]
    fn reset_requires_ack_and_does_not_claim_host_cleanup() {
        let command = protocol::static_analysis::Command::FactoryReset;
        let mut c = Client::new(
            Mock {
                read: vec![response(14, 1, &[])].into(),
                short_write: true,
                ..Default::default()
            },
            Duration::from_secs(1),
        );
        assert!(matches!(
            c.control_verified(&command).unwrap(),
            ControlOutcome::Acknowledged(_)
        ));
        assert_eq!(c.into_inner().written, command.packet().unwrap());
        let mut c = Client::new(Mock::default(), Duration::from_secs(1));
        assert!(c
            .control_verified(&command)
            .unwrap_err()
            .to_string()
            .contains("may have been sent"));
        assert_eq!(c.into_inner().written, command.packet().unwrap());
    }
}
