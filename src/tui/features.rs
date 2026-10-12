//! Typed operations for the feature command panel. Rendering never does I/O.
use crate::{
    client::{Client, ControlOutcome},
    models::{Capabilities, Model},
    protocol::static_analysis::{Command, Side},
    settings::StudioProState,
};
use std::io::{self, Read, Write};
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Operation {
    Peers,
    Codec,
    Id,
    Color,
    BoxMac,
    Control(Command),
    Find {
        side: Side,
        seconds: u8,
        target: String,
    },
}
#[derive(Debug)]
pub(super) struct ResultData {
    pub text: String,
    pub info: Option<StudioProState>,
    pub codecs: Option<[(u8, crate::protocol::static_analysis::Codec); 2]>,
    pub reset: bool,
    pub certain: bool,
}
impl Operation {
    pub fn parse(input: &str) -> Result<Self, String> {
        if input.len() > 256 || !input.is_ascii() || input.bytes().any(|b| b.is_ascii_control()) {
            return Err("feature command must be at most 256 printable ASCII bytes".into());
        }
        let args: Vec<String> = input.split_whitespace().map(str::to_owned).collect();
        let words: Vec<&str> = args.iter().map(String::as_str).collect();
        Ok(match words.as_slice() {
            ["peers"] => Self::Peers,
            ["codec"] => Self::Codec,
            ["id"] => Self::Id,
            ["color"] => Self::Color,
            ["box-mac"] => Self::BoxMac,
            ["find", side, seconds, target] => {
                let side = match *side {
                    "left" => Side::Left,
                    "right" => Side::Right,
                    _ => return Err("finder side must be left or right".into()),
                };
                let seconds = seconds
                    .parse()
                    .map_err(|_| "finder duration must be 1..30 seconds")?;
                crate::finder::validate(seconds, target).map_err(|e| e.to_string())?;
                Self::Find {
                    side,
                    seconds,
                    target: (*target).into(),
                }
            }
            ["control", ..] => {
                let command = Command::parse(&args[1..])?;
                if command.control_capability().is_none() {
                    return Err("query is not a control".into());
                }
                Self::Control(command)
            }
            _ => return Err(
                "use peers|codec|id|color|box-mac, control OP ARGS, or find SIDE SECONDS TARGET"
                    .into(),
            ),
        })
    }
    pub fn is_write(&self) -> bool {
        matches!(self, Self::Control(_) | Self::Find { .. })
    }
    pub fn is_reset(&self) -> bool {
        matches!(self, Self::Control(Command::FactoryReset))
    }
    pub fn check(&self, caps: Capabilities) -> io::Result<()> {
        let bit = match self {
            Self::Peers => Capabilities::MULTIPOINT_PEERS,
            Self::Codec => Capabilities::CODEC_STATUS,
            Self::Id => Capabilities::HEADPHONE_ID,
            Self::Color => Capabilities::HEADPHONE_COLOR,
            Self::BoxMac => Capabilities::BOX_MAC,
            Self::Control(c) => c.control_capability().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "query is not a control")
            })?,
            Self::Find {
                side: Side::Left, ..
            } => Capabilities::FIND_LEFT,
            Self::Find { .. } => Capabilities::FIND_RIGHT,
        };
        if caps.contains(bit) {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "feature is not verified for Studio Pro; official-app capture required",
            ))
        }
    }
    pub fn execute<T: Read + Write>(
        &self,
        client: &mut Client<T>,
        cancelled: &dyn Fn() -> bool,
    ) -> io::Result<ResultData> {
        self.check(Model::StudioPro.capabilities())?;
        let mut result = ResultData {
            text: String::new(),
            info: None,
            codecs: None,
            reset: false,
            certain: true,
        };
        result.text = match self {
            Self::Peers => format!("Peers: {:?}", client.peers()?),
            Self::Codec => {
                let codecs = client.codecs()?;
                result.codecs = Some(codecs);
                format!("Codecs: {codecs:?}")
            }
            Self::Id => format!("ID: {}", client.headphone_id()?),
            Self::Color => format!("Color: {}", client.headphone_color()?),
            Self::BoxMac => format!("Box MAC: {}", client.box_mac()?),
            Self::Find {
                side,
                seconds,
                target,
            } => {
                crate::finder::ring_cancellable(client, *side, *seconds, target, cancelled)?;
                result.certain = false;
                "Host playback completed; stop sent; device state unconfirmed. Refresh before further writes.".into()
            }
            Self::Control(command) => match client.control(command)? {
                ControlOutcome::Readback(info) => {
                    result.info = Some(info);
                    "Control verified by device-info readback".into()
                }
                ControlOutcome::Acknowledged(_) => {
                    result.reset = self.is_reset();
                    result.certain = result.reset;
                    "Control acknowledged; resulting device state not independently confirmed. Refresh before further writes.".into()
                }
                ControlOutcome::SentUnconfirmed => {
                    result.certain = false;
                    "Control sent; finder state and audio unconfirmed. Refresh before further writes.".into()
                }
            },
        };
        Ok(result)
    }
}
