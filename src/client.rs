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
    pub fn operation_capabilities(&self) -> crate::models::Capabilities {
        crate::models::supported_capabilities()
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
        self.info_in(Lang::En)
    }
    pub fn info_in(&mut self, lang: Lang) -> io::Result<StudioProState> {
        let payload = self.request_in(lang, protocol::INFO, &[0])?.payload;
        let length = payload.len();
        StudioProState::new(payload).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                crate::i18n::txt(lang)
                    .info_short
                    .replacen("{}", &length.to_string(), 1),
            )
        })
    }
    pub fn firmware(&mut self) -> io::Result<String> {
        self.firmware_in(Lang::En)
    }
    pub fn firmware_in(&mut self, lang: Lang) -> io::Result<String> {
        let response = self.request_in(lang, protocol::FIRMWARE, &[0])?;
        crate::settings::firmware(&response.payload).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, crate::i18n::txt(lang).fw_short)
        })
    }
    /// Evidence-aware setting API. Unknown model is rejected before I/O;
    /// unsupported firmware permits the read-only firmware query only.
    /// Each setting uses preflight and the existing ack/readback path.
    pub fn set_for_model(
        &mut self,
        model: Option<crate::models::Model>,
        setting: &Setting,
    ) -> io::Result<StudioProState> {
        self.set_for_model_in(Lang::En, model, setting)
    }
    pub fn set_for_model_in(
        &mut self,
        lang: Lang,
        model: Option<crate::models::Model>,
        setting: &Setting,
    ) -> io::Result<StudioProState> {
        crate::application::validate_setting(setting)?;
        let model = model.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Unsupported,
                "setting requires an explicitly confirmed model",
            )
        })?;
        let firmware = self.firmware_in(lang)?;
        crate::application::authorize_setting(Some(model), Some(&firmware), setting)?;
        let info = self.info_in(lang)?;
        crate::application::validate_preflight(&info, setting)?;
        self.set_in(lang, setting)
    }
    /// Low-level compatibility API: caller must establish model and firmware evidence.
    /// Prefer [`Self::set_for_model`] for automatic evidence preflight.
    /// Only report success after a CRC-valid device-info readback matches.
    /// A failed acknowledgement is never silently retried, because writes may have taken effect.
    /// Spatial audio is the exception: retail firmware applies the write but
    /// answers with an `85 86 87` notification instead of a `DD EE FF`
    /// acknowledgement, so verification there relies on the readback alone.
    pub fn set(&mut self, setting: &Setting) -> io::Result<StudioProState> {
        self.set_in(Lang::En, setting)
    }
    pub fn set_in(&mut self, lang: Lang, setting: &Setting) -> io::Result<StudioProState> {
        crate::application::validate_setting(setting)?;
        let t = crate::i18n::txt(lang);
        let acknowledged = setting.expects_ack();
        if acknowledged {
            self.request_in(lang, setting.instruction, &setting.payload)
                .map_err(|error| {
                    io::Error::new(
                        error.kind(),
                        t.set_ackfail.replacen("{}", &error.to_string(), 1),
                    )
                })?;
        } else {
            self.send(setting)?;
        }
        let info = self.info_in(lang).map_err(|error| {
            io::Error::new(
                error.kind(),
                if acknowledged {
                    t.set_rbfail
                } else {
                    t.set_rbfail_noack
                }
                .replacen("{}", &error.to_string(), 1),
            )
        })?;
        if !setting.matches(&info) {
            return Err(io::Error::other(
                if acknowledged {
                    t.set_mismatch
                } else {
                    t.set_mismatch_noack
                }
                .replacen("{}", &setting.key, 1)
                .replacen("{}", &setting.value, 1),
            ));
        }
        Ok(info)
    }
    pub fn send(&mut self, setting: &Setting) -> io::Result<()> {
        crate::application::validate_setting(setting)?;
        self.io
            .write_all(&protocol::request(setting.instruction, &setting.payload)?)?;
        self.io.flush()
    }
    /// Execute an implemented typed control after firmware preflight.
    /// Writes are never retried implicitly, including LDAC and resets.
    pub fn control(
        &mut self,
        command: &protocol::static_analysis::Command,
    ) -> io::Result<ControlOutcome> {
        self.control_cancellable(command, &|| false)
    }
    pub fn control_cancellable(
        &mut self,
        command: &protocol::static_analysis::Command,
        cancelled: &dyn Fn() -> bool,
    ) -> io::Result<ControlOutcome> {
        if cancelled() {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "control cancelled",
            ));
        }
        command.packet()?;
        self.require_capture(
            command.control_capability().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "query is not a control")
            })?,
            "control",
        )?;
        self.control_verified_cancellable(command, cancelled)
    }
    #[cfg(test)]
    fn control_verified(
        &mut self,
        command: &protocol::static_analysis::Command,
    ) -> io::Result<ControlOutcome> {
        self.control_verified_cancellable(command, &|| false)
    }
    fn control_verified_cancellable(
        &mut self,
        command: &protocol::static_analysis::Command,
        cancelled: &dyn Fn() -> bool,
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
        if let C::ClosePeer { index, address } = command {
            let peers = self.collect_peers()?;
            if peers.slots[usize::from(*index - 1)]
                .as_ref()
                .map(|p| &p.address)
                != Some(address)
            {
                return Err(invalid_data(
                    "selected peer slot/MAC changed; refresh before closing",
                ));
            }
        }
        // Cancellation during firmware/peer preflight must stop the write.
        if cancelled() {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "control cancelled before write",
            ));
        }
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
        if cancelled() {
            return Err(io::Error::new(io::ErrorKind::Interrupted,
                "control may have been sent; cancelled before readback; inspect state before retrying"));
        }
        if matches!(command, C::Find { .. }) {
            // The verified six-byte decoder lacks the finder-right byte.
            // A sent control cannot claim a confirmed finder state or audio.
            return Ok(ControlOutcome::SentUnconfirmed);
        }
        let readback = match command {
            C::ClosePeer { .. } => Some((5, &payload[..1], false)),
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
            if let C::ClosePeer { address, .. } = command {
                // Match the app's settling interval before querying changed links.
                std::thread::sleep(Duration::from_millis(200));
                let peers = self.collect_peers().map_err(|error| io::Error::new(error.kind(),
                    format!("dual-close sent; peer readback failed: {error}; inspect before retrying")))?;
                if peers.slots.iter().flatten().any(|p| &p.address == address) {
                    return Err(invalid_data(
                        "dual-close acknowledged but selected peer is still connected",
                    ));
                }
            }
            return Ok(ControlOutcome::Readback(info));
        }
        Ok(ControlOutcome::Acknowledged(ack.expect(
            "non-finder controls without readback require ack",
        )))
    }
    /// Read all headset peer slots, including UTF-8 names and slot indices.
    /// Available by default, with exact firmware preflight before the query.
    pub fn peers(&mut self) -> io::Result<protocol::static_analysis::PeerList> {
        self.require_capture(
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
    fn require_capture(&mut self, bit: u32, feature: &str) -> io::Result<()> {
        // Unsupported operations refuse before I/O. Implemented operations still
        // require exact firmware preflight before their first packet.
        if !self.operation_capabilities().contains(bit) {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("{feature} is not verified for Studio Pro; official-app capture required"),
            ));
        }
        let firmware = self.firmware()?;
        crate::models::require_verified_firmware(
            Some(crate::models::Model::StudioPro),
            Some(&firmware),
        )?;
        Ok(())
    }
    pub fn codecs(&mut self) -> io::Result<[(u8, protocol::static_analysis::Codec); 2]> {
        self.require_capture(crate::models::Capabilities::CODEC_STATUS, "codec status")?;
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
        self.require_capture(bit, feature)?;
        match protocol::static_analysis::interpret(&self.request(instruction, &[0])?)
            .map_err(invalid_data)?
        {
            protocol::static_analysis::Response::Byte { value, .. } => Ok(value),
            _ => Err(invalid_data("unexpected byte response")),
        }
    }
    pub fn box_mac(&mut self) -> io::Result<crate::bluetooth::BluetoothAddress> {
        self.require_capture(crate::models::Capabilities::BOX_MAC, "box MAC")?;
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
    /// Independent peer management is unavailable; targeted dual-close is a distinct control.
    pub fn peer_disconnect(&mut self, _peer: &MultipointPeer) -> io::Result<()> {
        Err(UnsupportedFeature::PeerDisconnect.into())
    }
    /// Independent peer management is unavailable; targeted dual-close is a distinct control.
    pub fn peer_reconnect(&mut self, _peer: &MultipointPeer) -> io::Result<()> {
        Err(UnsupportedFeature::PeerReconnect.into())
    }
    /// Independent peer management is unavailable; targeted dual-close is a distinct control.
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
    fn evidence_aware_setting_queries_unknown_firmware_but_never_sends_setting() {
        let mut client = Client::new(
            Mock {
                read: vec![response(protocol::FIRMWARE, 1, &[0, 2, 6])].into(),
                ..Default::default()
            },
            Duration::from_secs(1),
        );
        let setting = Setting::parse("game", "on").unwrap();
        assert_eq!(
            client
                .set_for_model(Some(crate::models::Model::StudioPro), &setting)
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
        assert_eq!(
            client.into_inner().written,
            protocol::request(protocol::FIRMWARE, &[0]).unwrap()
        );
    }
    #[test]
    fn evidence_aware_setting_rejects_unknown_model_before_io() {
        let mut client = Client::new(Mock::default(), Duration::from_secs(1));
        let setting = Setting::parse("game", "on").unwrap();
        assert_eq!(
            client.set_for_model(None, &setting).unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
        assert!(client.into_inner().written.is_empty());
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
    fn queries_require_successful_firmware_preflight() {
        let mut c = Client::new(Mock::default(), Duration::from_secs(1));
        for result in [
            c.peers().map(|_| ()),
            c.codecs().map(|_| ()),
            c.headphone_id().map(|_| ()),
            c.headphone_color().map(|_| ()),
            c.box_mac().map(|_| ()),
        ] {
            assert_eq!(result.unwrap_err().kind(), io::ErrorKind::TimedOut);
        }
        assert_eq!(
            c.into_inner().written,
            protocol::request(1, &[0]).unwrap().repeat(5)
        );
    }
    #[test]
    fn peers_are_available_by_default_after_firmware_preflight() {
        let mut c = Client::new(
            Mock {
                read: vec![
                    response(1, 1, &[0, 2, 5, 0, 0, 0]),
                    response(13, 1, &[1, 1, b'P', b'C', 1, 2, 3, 4, 5, 6]),
                ]
                .into(),
                ..Default::default()
            },
            Duration::from_secs(1),
        );
        let peers = c.peers().unwrap();
        assert!(peers.complete);
        assert_eq!(peers.slots[0].as_ref().unwrap().name, "PC");
        assert_eq!(
            c.into_inner().written,
            [
                protocol::request(1, &[0]).unwrap(),
                protocol::request(13, &[0]).unwrap()
            ]
            .concat()
        );
        let mut c = Client::new(
            Mock {
                read: vec![response(1, 1, &[0, 2, 6, 0, 0, 0])].into(),
                ..Default::default()
            },
            Duration::from_secs(1),
        );
        assert!(c.peers().is_err());
        assert_eq!(c.into_inner().written, protocol::request(1, &[0]).unwrap());
    }
    #[test]
    fn unestablished_controls_still_refuse_before_io() {
        use protocol::static_analysis::Command as C;
        for command in [
            C::FactoryReset,
            C::HighQualityDecoding(true),
            C::Find {
                enabled: true,
                side: protocol::static_analysis::Side::Left,
            },
        ] {
            let mut c = Client::new(Mock::default(), Duration::from_secs(1));
            assert_eq!(
                c.control(&command).unwrap_err().kind(),
                io::ErrorKind::Unsupported
            );
            assert!(c.into_inner().written.is_empty());
        }
        let mut c = Client::new(Mock::default(), Duration::from_secs(1));
        assert_eq!(
            c.control(&C::Peers).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
        assert!(c.into_inner().written.is_empty());
    }
    #[test]
    fn targeted_dual_close_checks_slot_before_write_and_both_readbacks() {
        use protocol::static_analysis::Command as C;
        let command = C::ClosePeer {
            index: 1,
            address: crate::bluetooth::BluetoothAddress::from_octets([1, 2, 3, 4, 5, 6]),
        };
        let before = response(13, 1, &[1, 1, b'P', 1, 2, 3, 4, 5, 6]);
        let mut raw = vec![0; 31];
        raw[5] = 0;
        for after in [response(13, 1, &[0, 0]), before.clone()] {
            let mut c = Client::new(
                Mock {
                    read: vec![
                        response(1, 1, &[0, 2, 5, 0, 0, 0]),
                        before.clone(),
                        response(6, 1, &[0]),
                        response(4, 1, &raw),
                        after.clone(),
                    ]
                    .into(),
                    ..Default::default()
                },
                Duration::from_secs(1),
            );
            let result = c.control(&command);
            assert_eq!(result.is_ok(), after == response(13, 1, &[0, 0]));
            assert_eq!(
                c.into_inner().written,
                [
                    protocol::request(1, &[0]).unwrap(),
                    protocol::request(13, &[0]).unwrap(),
                    command.packet().unwrap(),
                    protocol::request(4, &[0]).unwrap(),
                    protocol::request(13, &[0]).unwrap()
                ]
                .concat()
            );
        }
        let mut c = Client::new(
            Mock {
                read: vec![
                    response(1, 1, &[0, 2, 5, 0, 0, 0]),
                    response(13, 1, &[0, 0]),
                ]
                .into(),
                ..Default::default()
            },
            Duration::from_secs(1),
        );
        assert!(c
            .control(&command)
            .unwrap_err()
            .to_string()
            .contains("slot/MAC changed"));
        assert_eq!(
            c.into_inner().written,
            [
                protocol::request(1, &[0]).unwrap(),
                protocol::request(13, &[0]).unwrap()
            ]
            .concat()
        );
    }
    #[test]
    fn cancellation_during_peer_preflight_prevents_targeted_close() {
        use std::sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        };
        struct CancelOnPeer {
            inner: Mock,
            cancelled: Arc<AtomicBool>,
        }
        impl Read for CancelOnPeer {
            fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
                let n = self.inner.read(output)?;
                if n >= 4 && output[..4] == [0xdd, 0xee, 0xff, 13] {
                    self.cancelled.store(true, Ordering::SeqCst);
                }
                Ok(n)
            }
        }
        impl Write for CancelOnPeer {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                self.inner.write(bytes)
            }
            fn flush(&mut self) -> io::Result<()> {
                self.inner.flush()
            }
        }
        let cancelled = Arc::new(AtomicBool::new(false));
        let io = CancelOnPeer {
            inner: Mock {
                read: vec![
                    response(1, 1, &[0, 2, 5, 0, 0, 0]),
                    response(13, 1, &[1, 1, b'P', 1, 2, 3, 4, 5, 6]),
                ]
                .into(),
                ..Default::default()
            },
            cancelled: cancelled.clone(),
        };
        let mut c = Client::new(io, Duration::from_secs(1));
        let command = protocol::static_analysis::Command::ClosePeer {
            index: 1,
            address: crate::bluetooth::BluetoothAddress::from_octets([1, 2, 3, 4, 5, 6]),
        };
        let error = c
            .control_cancellable(&command, &|| cancelled.load(Ordering::SeqCst))
            .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::Interrupted);
        assert_eq!(
            c.into_inner().inner.written,
            [
                protocol::request(1, &[0]).unwrap(),
                protocol::request(13, &[0]).unwrap()
            ]
            .concat()
        );
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
    fn controls_require_firmware_before_any_setting_write() {
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
        for command in commands {
            let supported = crate::models::supported_capabilities()
                .contains(command.control_capability().unwrap());
            let mut c = Client::new(Mock::default(), Duration::from_secs(1));
            let error = c.control(&command).unwrap_err();
            assert_eq!(
                error.kind(),
                if supported {
                    io::ErrorKind::TimedOut
                } else {
                    io::ErrorKind::Unsupported
                }
            );
            assert_eq!(
                c.into_inner().written,
                if supported {
                    protocol::request(1, &[0]).unwrap()
                } else {
                    vec![]
                }
            );
        }
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
    #[test]
    fn forged_setting_packets_fail_before_any_io_in_all_setting_apis() {
        for mutation in 0..3 {
            let mut setting = Setting::parse("game", "on").unwrap();
            match mutation {
                0 => setting.instruction = 16,
                1 => setting.payload.clear(),
                _ => setting.payload.push(0),
            }
            let mut client = Client::new(Mock::default(), Duration::from_secs(1));
            assert!(client
                .set_for_model(Some(crate::models::Model::StudioPro), &setting)
                .is_err());
            assert!(client.set(&setting).is_err());
            assert!(client.send(&setting).is_err());
            assert!(client.into_inner().written.is_empty());
        }
    }
    #[test]
    fn capture_gate_still_checks_exact_firmware_after_historical_bit_is_available() {
        let mut client = Client::new(
            Mock {
                read: vec![response(protocol::FIRMWARE, 1, &[0, 2, 6])].into(),
                ..Default::default()
            },
            Duration::from_secs(1),
        );
        assert_eq!(
            client
                .require_capture(crate::models::Capabilities::ANC, "test operation")
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
        assert_eq!(
            client.into_inner().written,
            protocol::request(protocol::FIRMWARE, &[0]).unwrap()
        );
    }

    #[test]
    fn localized_spatial_readback_failure_never_claims_an_acknowledgement() {
        let mut client = Client::new(Mock::default(), Duration::from_secs(1));
        let error = client
            .set_in(Lang::Es, &Setting::parse("spatial", "on").unwrap())
            .unwrap_err()
            .to_string();
        assert!(error.starts_with(
            crate::i18n::txt(Lang::Es)
                .set_rbfail_noack
                .split("{}")
                .next()
                .unwrap()
        ));
        assert!(!error.contains("acknowledged"));
    }
    #[test]
    fn evidence_aware_setting_refuses_missing_field_before_write() {
        let mut client = Client::new(
            Mock {
                read: vec![
                    response(protocol::FIRMWARE, 1, &[0, 2, 5]),
                    response(protocol::INFO, 1, &[0; 8]),
                ]
                .into(),
                ..Default::default()
            },
            Duration::from_secs(1),
        );
        let error = client
            .set_for_model(
                Some(crate::models::Model::StudioPro),
                &Setting::parse("wind", "on").unwrap(),
            )
            .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(
            client.into_inner().written,
            [
                protocol::request(protocol::FIRMWARE, &[0]).unwrap(),
                protocol::request(protocol::INFO, &[0]).unwrap()
            ]
            .concat()
        );
    }
}
