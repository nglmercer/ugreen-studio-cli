//! Offline interpretations from `docs/reverse-engineering.md`.
//!
//! These types are static APK evidence, not hardware verification. They
//! deliberately have no transport API and do not enable live capabilities.
use super::{request, ResponseFrame};
use crate::bluetooth::BluetoothAddress;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Action {
    None = 0,
    PlayPause = 1,
    VolumeUp = 2,
    VolumeDown = 3,
    Next = 4,
    Previous = 5,
    VoiceAssistant = 6,
    GameMusic = 7,
    NoiseControl = 8,
    AnswerHangUp = 9,
    LongPressEq = 11,
}
impl TryFrom<u8> for Action {
    type Error = String;
    fn try_from(v: u8) -> Result<Self, String> {
        Ok(match v {
            0 => Self::None,
            1 => Self::PlayPause,
            2 => Self::VolumeUp,
            3 => Self::VolumeDown,
            4 => Self::Next,
            5 => Self::Previous,
            6 => Self::VoiceAssistant,
            7 => Self::GameMusic,
            8 => Self::NoiseControl,
            9 => Self::AnswerHangUp,
            11 => Self::LongPressEq,
            _ => return Err(format!("unknown button action {v}")),
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Firmware,
    HeadphoneId,
    Color,
    Peers,
    Codec,
    BoxMac,
    FactoryReset,
    Find {
        enabled: bool,
        side: Side,
    },
    Buttons([Action; 8]),
    AiActions([Action; 4]),
    Ldac {
        enabled: bool,
        peer: Option<BluetoothAddress>,
    },
    HighQualityDecoding(bool),
    Volume(u8),
    Stereo(bool),
    AiControl(bool),
    SharingOff,
    SharingSinglePeer,
    SharingPeer {
        index: u8,
        address: BluetoothAddress,
    },
}
impl Command {
    /// Required capture-backed capability for a control operation.
    /// Query variants return None and must use the query APIs.
    pub fn control_capability(&self) -> Option<u32> {
        use crate::models::Capabilities as C;
        Some(match self {
            Self::Buttons(_) => C::BUTTONS,
            Self::AiActions(_) => C::AI_ACTIONS,
            Self::Ldac { peer: None, .. } => C::LDAC,
            Self::Ldac { peer: Some(_), .. } => C::LDAC_PEER,
            Self::HighQualityDecoding(_) => C::HIGH_QUALITY_DECODING,
            Self::Volume(_) => C::VOLUME,
            Self::Stereo(_) => C::CHANNEL_MODE,
            Self::AiControl(_) => C::AI_CONTROL,
            Self::SharingOff => C::AUDIO_SHARING,
            Self::SharingSinglePeer => C::AUDIO_SHARING_SINGLE,
            Self::SharingPeer { .. } => C::AUDIO_SHARING_PEER,
            Self::FactoryReset => C::FACTORY_RESET,
            Self::Find {
                side: Side::Left, ..
            } => C::FIND_LEFT,
            Self::Find {
                side: Side::Right, ..
            } => C::FIND_RIGHT,
            _ => return None,
        })
    }
    /// Refuse live controls before opening a connection or writing bytes.
    pub fn check_control(&self) -> io::Result<()> {
        let bit = self.control_capability().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "query is not a control; use its query command",
            )
        })?;
        if crate::models::Model::StudioPro.capabilities().contains(bit) {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "control is not verified for Studio Pro; official-app capture required",
            ))
        }
    }
    /// Parse the offline `candidate` command. No arbitrary instructions are accepted.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let words: Vec<&str> = args.iter().map(String::as_str).collect();
        let toggle = |v| match v {
            "on" => Ok(true),
            "off" => Ok(false),
            _ => Err("expected on or off".to_string()),
        };
        let address = |v| BluetoothAddress::parse(v).map_err(|e| e.to_string());
        let actions = |v: &str| -> Result<Vec<Action>, String> {
            v.split(',')
                .map(|s| {
                    s.parse::<u8>()
                        .map_err(|_| "actions must be comma-separated byte values".to_string())
                        .and_then(Action::try_from)
                })
                .collect()
        };
        Ok(match words.as_slice() {
            ["firmware"] => Self::Firmware, ["id"] => Self::HeadphoneId,
            ["color"] => Self::Color, ["peers"] => Self::Peers,
            ["codec"] => Self::Codec, ["box-mac"] => Self::BoxMac,
            ["factory-reset"] => Self::FactoryReset,
            ["find", side, enabled] => Self::Find {
                side: match *side { "left" => Side::Left, "right" => Side::Right, _ => return Err("finder side must be left or right".into()) },
                enabled: toggle(*enabled)?,
            },
            ["ldac", enabled] => Self::Ldac { enabled: toggle(*enabled)?, peer: None },
            ["ldac", enabled, mac] => Self::Ldac { enabled: toggle(*enabled)?, peer: Some(address(mac)?) },
            ["high-quality", enabled] => Self::HighQualityDecoding(toggle(*enabled)?),
            ["volume", value] => Self::Volume(value.parse().map_err(|_| "volume must be a byte (0..255); hardware range unverified")?),
            ["channel", "mono"] => Self::Stereo(false), ["channel", "stereo"] => Self::Stereo(true),
            ["ai-control", enabled] => Self::AiControl(toggle(*enabled)?),
            ["buttons", values] => Self::Buttons(actions(values)?.try_into().map_err(|_| "buttons needs eight action values")?),
            ["ai-actions", values] => Self::AiActions(actions(values)?.try_into().map_err(|_| "ai-actions needs four action values")?),
            ["sharing", "off"] => Self::SharingOff,
            ["sharing", "single"] => Self::SharingSinglePeer,
            ["sharing", index, mac] => {
                let index = index.parse().map_err(|_| "sharing needs peer index 1 or 2")?;
                if !matches!(index, 1 | 2) { return Err("sharing needs peer index 1 or 2".into()); }
                Self::SharingPeer { index, address: address(mac)? }
            }
            _ => return Err("usage: ugreen candidate firmware|id|color|peers|codec|box-mac|factory-reset; find left|right on|off; ldac on|off [MAC]; high-quality on|off; volume BYTE; channel mono|stereo; ai-control on|off; buttons A,A,A,A,A,A,A,A; ai-actions A,A,A,A; sharing off|single|INDEX MAC".into()),
        })
    }
    /// Build an offline candidate. Never sends it or claims verification.
    pub fn packet(&self) -> io::Result<Vec<u8>> {
        let (instruction, payload) = match self {
            Self::Firmware => (1, vec![0]),
            Self::HeadphoneId => (2, vec![0]),
            Self::Color => (3, vec![0]),
            Self::Peers => (13, vec![0]),
            Self::FactoryReset => (14, vec![0]),
            Self::Codec => (15, vec![0]),
            Self::BoxMac => (31, vec![0]),
            Self::Find { enabled, side } => (
                7,
                vec![
                    u8::from(*enabled),
                    match side {
                        Side::Left => 1,
                        Side::Right => 2,
                    },
                ],
            ),
            Self::Buttons(actions) => (10, actions.iter().map(|a| *a as u8).collect()),
            Self::AiActions(actions) => (24, actions.iter().map(|a| *a as u8).collect()),
            Self::Ldac { enabled, peer } => {
                let mut p = vec![u8::from(*enabled)];
                if let Some(address) = peer {
                    p.extend_from_slice(&if *enabled { address.octets() } else { [0; 6] });
                }
                (11, p)
            }
            Self::HighQualityDecoding(v) => (30, vec![u8::from(*v)]),
            Self::Volume(v) => (16, vec![*v]),
            Self::Stereo(v) => (25, vec![u8::from(*v)]),
            Self::AiControl(v) => (27, vec![u8::from(*v)]),
            Self::SharingOff => (22, vec![0]),
            Self::SharingSinglePeer => (22, vec![1, 255, 255, 255, 255, 255, 255, 255]),
            Self::SharingPeer { index, address } => {
                if !matches!(index, 1 | 2) {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "peer index must be 1 or 2",
                    ));
                }
                let mut p = vec![1, *index];
                p.extend_from_slice(&address.octets());
                (22, p)
            }
        };
        request(instruction, &payload)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Codec {
    Sbc,
    Aac,
    Ldac,
    Unknown(u8),
}
impl From<u8> for Codec {
    fn from(v: u8) -> Self {
        match v {
            0 => Self::Sbc,
            2 => Self::Aac,
            255 => Self::Ldac,
            v => Self::Unknown(v),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Peer {
    pub index: u8,
    pub name: String,
    pub address: BluetoothAddress,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeerUpdate {
    Clear,
    Connected { peer: Peer, second_absent: bool },
}
impl PeerUpdate {
    pub fn parse(p: &[u8]) -> Result<Self, String> {
        if p.len() < 2 {
            return Err("peer response needs flag and index".into());
        }
        match p[1] {
            0 => Ok(Self::Clear),
            1 | 2 => {
                if p.len() < 8 {
                    return Err("peer response lacks MAC".into());
                }
                let split = p.len() - 6;
                let name = std::str::from_utf8(&p[2..split])
                    .map_err(|_| "peer name is not UTF-8")?
                    .to_owned();
                Ok(Self::Connected {
                    peer: Peer {
                        index: p[1],
                        name,
                        address: BluetoothAddress::from_octets(p[split..].try_into().unwrap()),
                    },
                    second_absent: p[1] == 1 && p[0] == 1,
                })
            }
            _ => Err("unknown peer index".into()),
        }
    }
}
/// Accumulates all peer frames, including two frames in one socket read.
/// `None` denotes missing evidence; an explicit clear denotes an empty list.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PeerList {
    pub slots: [Option<Peer>; 2],
    pub observed: bool,
    /// Explicit clear/single-peer absence or two populated slots received.
    /// A lone peer response without the absence flag is incomplete.
    pub complete: bool,
}
impl PeerList {
    pub fn apply(&mut self, update: PeerUpdate) -> Result<(), String> {
        if let PeerUpdate::Connected { peer, .. } = &update {
            if !matches!(peer.index, 1 | 2) {
                return Err("unknown peer index".into());
            }
        }
        self.observed = true;
        match update {
            PeerUpdate::Clear => {
                self.slots = [None, None];
                self.complete = true;
            }
            PeerUpdate::Connected {
                peer,
                second_absent,
            } => {
                if second_absent {
                    self.slots[1] = None;
                }
                let i = usize::from(peer.index - 1);
                self.slots[i] = Some(peer);
                self.complete = second_absent || self.slots.iter().all(Option::is_some);
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    Firmware([[u8; 3]; 2]),
    Byte { instruction: u8, value: u8 },
    Peers(PeerUpdate),
    Codecs([(u8, Codec); 2]),
    BoxMac(BluetoothAddress),
    Buttons([u8; 4]),
    Ack { instruction: u8, payload: Vec<u8> },
}
/// Interpret a CRC-valid response without promoting its meaning to verified.
pub fn interpret(frame: &ResponseFrame) -> Result<Response, String> {
    if !frame.succeeded {
        return Err("device rejected request".into());
    }
    let p = &frame.payload;
    let exact = |n| {
        if p.len() == n {
            Ok(())
        } else {
            Err(format!(
                "instruction {:02X} needs {n} bytes, got {}",
                frame.instruction,
                p.len()
            ))
        }
    };
    Ok(match frame.instruction {
        1 => {
            exact(6)?;
            Response::Firmware([p[..3].try_into().unwrap(), p[3..].try_into().unwrap()])
        }
        13 => Response::Peers(PeerUpdate::parse(p)?),
        15 => {
            exact(4)?;
            Response::Codecs([(p[0], p[1].into()), (p[2], p[3].into())])
        }
        31 => {
            exact(6)?;
            let mut mac: [u8; 6] = p[..].try_into().unwrap();
            mac.reverse();
            Response::BoxMac(BluetoothAddress::from_octets(mac))
        }
        24 => {
            exact(4)?;
            Response::Buttons(p[..].try_into().unwrap())
        }
        2 | 3 | 5 | 6 | 9 | 11 | 12 | 16 | 17 | 18 | 20 | 21 | 22 | 23 | 25 | 27 | 30 => {
            exact(1)?;
            Response::Byte {
                instruction: frame.instruction,
                value: p[0],
            }
        }
        7 | 10 | 14 => Response::Ack {
            instruction: frame.instruction,
            payload: p.clone(),
        },
        _ => return Err("no static response interpretation for this instruction".into()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn payload(command: Command) -> Vec<u8> {
        let bytes = command.packet().unwrap();
        assert_eq!(
            super::super::crc_modbus(&bytes[3..bytes.len() - 2]).to_le_bytes(),
            bytes[bytes.len() - 2..]
        );
        bytes[5..bytes.len() - 2].to_vec()
    }
    #[test]
    fn candidate_layouts() {
        let mac = BluetoothAddress::from_octets([1, 2, 3, 4, 5, 6]);
        assert_eq!(
            payload(Command::Ldac {
                enabled: true,
                peer: Some(mac.clone())
            }),
            [1, 1, 2, 3, 4, 5, 6]
        );
        assert_eq!(
            payload(Command::Ldac {
                enabled: false,
                peer: Some(mac.clone())
            }),
            [0; 7]
        );
        assert_eq!(
            payload(Command::Ldac {
                enabled: false,
                peer: None
            }),
            [0]
        );
        assert_eq!(
            payload(Command::SharingPeer {
                index: 2,
                address: mac.clone()
            }),
            [1, 2, 1, 2, 3, 4, 5, 6]
        );
        assert!(Command::SharingPeer {
            index: 0,
            address: mac
        }
        .packet()
        .is_err());
        assert_eq!(
            payload(Command::SharingSinglePeer),
            [1, 255, 255, 255, 255, 255, 255, 255]
        );
        for (side, index) in [(Side::Left, 1), (Side::Right, 2)] {
            for enabled in [false, true] {
                assert_eq!(
                    payload(Command::Find { enabled, side }),
                    [u8::from(enabled), index]
                );
            }
        }
        assert_eq!(payload(Command::Buttons([Action::LongPressEq; 8])), [11; 8]);
        assert!(Action::try_from(10).is_err());
    }
    #[test]
    fn peer_updates_preserve_utf8_and_mac_order() {
        let mut list = PeerList::default();
        for index in [1, 2] {
            let mut p = vec![0, index];
            p.extend_from_slice("電話".as_bytes());
            p.extend_from_slice(&[1, 2, 3, 4, 5, index]);
            list.apply(PeerUpdate::parse(&p).unwrap()).unwrap();
        }
        assert_eq!(list.slots[1].as_ref().unwrap().name, "電話");
        assert_eq!(
            list.slots[0].as_ref().unwrap().address.octets(),
            [1, 2, 3, 4, 5, 1]
        );
        list.apply(PeerUpdate::parse(&[1, 1, 1, 2, 3, 4, 5, 6]).unwrap())
            .unwrap();
        assert!(list.slots[1].is_none());
        list.apply(PeerUpdate::Clear).unwrap();
        assert!(list.observed && list.slots == [None, None]);
        for p in [
            vec![],
            vec![0],
            vec![0, 3],
            vec![0, 1],
            vec![0, 1, 255, 1, 2, 3, 4, 5, 6],
        ] {
            assert!(PeerUpdate::parse(&p).is_err());
        }
    }
    #[test]
    fn response_lengths_rejections_and_unknown_codecs() {
        for (instruction, n) in [(1, 6), (15, 4), (31, 6), (24, 4), (11, 1)] {
            for size in [n - 1, n + 1] {
                assert!(interpret(&ResponseFrame {
                    instruction,
                    succeeded: true,
                    payload: vec![0; size]
                })
                .is_err());
            }
        }
        assert_eq!(
            interpret(&ResponseFrame {
                instruction: 31,
                succeeded: true,
                payload: vec![1, 2, 3, 4, 5, 6]
            })
            .unwrap(),
            Response::BoxMac(BluetoothAddress::from_octets([6, 5, 4, 3, 2, 1]))
        );
        assert_eq!(
            interpret(&ResponseFrame {
                instruction: 15,
                succeeded: true,
                payload: vec![1, 255, 2, 7]
            })
            .unwrap(),
            Response::Codecs([(1, Codec::Ldac), (2, Codec::Unknown(7))])
        );
        assert!(interpret(&ResponseFrame {
            instruction: 15,
            succeeded: false,
            payload: vec![0; 4]
        })
        .is_err());
    }
}

/// Additional device-info fields, kept apart from verified live status.
/// Offset 19 is intentionally raw: the APK uses it as both box status
/// and volume. Absent tail bytes stay unknown on 30-byte firmware replies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    pub right_battery: Option<u8>,
    pub box_battery: Option<u8>,
    pub codec: Option<Codec>,
    pub buttons: Option<[u8; 8]>,
    pub box_status_or_volume: Option<u8>,
    pub wear_detection: Option<u8>,
    pub audio_sharing: Option<bool>,
    pub ai_actions: Option<[u8; 4]>,
    pub ai_control: Option<bool>,
}
impl DeviceInfo {
    pub fn parse(p: &[u8]) -> Result<Self, String> {
        if p.len() < 8 {
            return Err("device-info needs at least 8 bytes".into());
        }
        let battery = |i| p.get(i).copied().filter(|v| *v <= 100);
        Ok(Self {
            right_battery: battery(1),
            box_battery: battery(2),
            codec: p.get(7).copied().map(Codec::from),
            buttons: p.get(8..16).map(|s| s.try_into().unwrap()),
            box_status_or_volume: p.get(19).copied(),
            wear_detection: p.get(21).copied(),
            audio_sharing: p.get(24).copied().and_then(boolean),
            ai_actions: p.get(26..30).map(|s| s.try_into().unwrap()),
            ai_control: p.get(30).copied().and_then(boolean),
        })
    }
}
fn boolean(v: u8) -> Option<bool> {
    match v {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    }
}

/// APK notification meanings use byte 5 as the event code. Byte 4 is
/// an envelope field. Six-byte captures expose only one event data byte;
/// the APK expects longer battery/finder packets, not yet verified here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Notification {
    Battery {
        left: u8,
        right: Option<u8>,
        case: Option<u8>,
    },
    Anc(u8),
    Game(u8),
    Eq(u8),
    MasterSlaveSwitch,
    Finder {
        left: u8,
        right: Option<u8>,
    },
    Spatial(u8),
    Sharing(u8),
    Dual(u8),
    Unknown {
        envelope: u8,
        code: u8,
        value: u8,
    },
}
pub fn interpret_notification(frame: &super::NotificationFrame) -> Notification {
    let [code, value] = frame.payload;
    match code {
        1 => Notification::Battery {
            left: value,
            right: None,
            case: None,
        },
        2 => Notification::Anc(value),
        4 => Notification::Game(value),
        5 => Notification::Eq(value),
        8 => Notification::MasterSlaveSwitch,
        9 => Notification::Finder {
            left: value,
            right: None,
        },
        10 => Notification::Spatial(value),
        11 => Notification::Sharing(value),
        12 => Notification::Dual(value),
        code => Notification::Unknown {
            envelope: frame.kind,
            code,
            value,
        },
    }
}
