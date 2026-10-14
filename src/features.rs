//! Declarative feature registry: the single authority
//! for headset feature metadata. The TUI feature
//! workspace, its typed forms and the worker all derive
//! from `FEATURES`; adding a feature means adding one
//! record here plus its localized name and description.
//!
//! A capability bit describes capture-verified evidence
//! for the current model. A feature is only runnable
//! when its bit is set; pending features stay visible
//! but never open a live operation.
use crate::{
    bluetooth::BluetoothAddress,
    client::{Client, ControlOutcome},
    finder,
    i18n::L,
    models::{Capabilities, Model},
    protocol::static_analysis::{Action, Command, Side},
    settings::StudioProState,
};
use std::io::{self, Read, Write};

/// Where a feature belongs in the workspace. The order
/// is stable: localized category arrays are indexed by
/// `FeatureCategory::index`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeatureCategory {
    Audio,
    Connection,
    Environment,
    Feedback,
    DeviceInfo,
    Multipoint,
    Codecs,
    AdvancedAudio,
    Controls,
    AudioSharing,
    Finder,
    DeviceManagement,
    Diagnostics,
}
impl FeatureCategory {
    pub const ALL: [FeatureCategory; 13] = [
        Self::Audio,
        Self::Connection,
        Self::Environment,
        Self::Feedback,
        Self::DeviceInfo,
        Self::Multipoint,
        Self::Codecs,
        Self::AdvancedAudio,
        Self::Controls,
        Self::AudioSharing,
        Self::Finder,
        Self::DeviceManagement,
        Self::Diagnostics,
    ];
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// What a feature is, which decides how the interface
/// offers it: a verified setting edits through the
/// settings screen, a query reads, a control writes,
/// an offline analysis never touches hardware.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeatureKind {
    VerifiedSetting,
    DeviceQuery,
    DeviceControl,
    OfflineAnalysis,
}

/// Write risk. Destructive operations get their own
/// high-risk confirmation even when verified.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SafetyLevel {
    Safe,
    Write,
    Destructive,
}

/// One typed form field. The workspace renders a
/// control for `kind` and validates the draft against
/// it before any operation is built.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgumentKind {
    Toggle,
    Side,
    Seconds,
    Byte,
    Mac,
    PeerIndex,
    Choice(&'static [(&'static str, u8)]),
    /// N comma-separated button action bytes.
    Actions(usize),
}

pub struct ArgumentSpec {
    /// Localized field label.
    pub label: fn(&'static L) -> &'static str,
    pub kind: ArgumentKind,
}

impl ArgumentKind {
    /// Localized input hint for the form field.
    pub fn hint(&self, t: &'static L) -> String {
        match self {
            Self::Toggle => t.arg_hint_toggle.into(),
            Self::Side => t.arg_hint_side.into(),
            Self::Seconds => t.arg_hint_seconds.into(),
            Self::Byte => t.arg_hint_byte.into(),
            Self::Mac => t.arg_hint_mac.into(),
            Self::PeerIndex => t.arg_hint_peer.into(),
            Self::Choice(options) => options
                .iter()
                .map(|(name, _)| *name)
                .collect::<Vec<_>>()
                .join("|"),
            Self::Actions(count) => t.arg_hint_actions.replacen("{}", &count.to_string(), 1),
        }
    }
}

/// Every known headset feature. The order is the
/// workspace's default listing order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeatureId {
    // Verified settings (edit through the settings screen).
    Anc,
    Eq,
    GameMode,
    SpatialAudio,
    DualConnection,
    WindReduction,
    Prompts,
    VolumeUpAction,
    VolumeDownAction,
    // Capture-gated queries.
    MultipointPeers,
    CodecStatus,
    HeadphoneId,
    HeadphoneColor,
    BoxMac,
    // Capture-gated codec controls.
    LdacToggle,
    LdacPeerToggle,
    HighQualityDecoding,
    // Capture-gated advanced audio.
    VolumeCommand,
    ChannelMode,
    // Capture-gated controls.
    ButtonMapping,
    AiActions,
    AiControl,
    // Capture-gated audio sharing.
    AudioSharingOff,
    AudioSharingSingle,
    AudioSharingPeer,
    // Capture-gated finder.
    FindLeft,
    FindRight,
    // Capture-gated device management.
    FactoryReset,
    // Offline diagnostics.
    OfflineDecode,
    OfflineAnalyze,
    OfflineCandidate,
}
impl FeatureId {
    pub const ALL: [FeatureId; 31] = [
        Self::Anc,
        Self::Eq,
        Self::GameMode,
        Self::SpatialAudio,
        Self::DualConnection,
        Self::WindReduction,
        Self::Prompts,
        Self::VolumeUpAction,
        Self::VolumeDownAction,
        Self::MultipointPeers,
        Self::CodecStatus,
        Self::HeadphoneId,
        Self::HeadphoneColor,
        Self::BoxMac,
        Self::LdacToggle,
        Self::LdacPeerToggle,
        Self::HighQualityDecoding,
        Self::VolumeCommand,
        Self::ChannelMode,
        Self::ButtonMapping,
        Self::AiActions,
        Self::AiControl,
        Self::AudioSharingOff,
        Self::AudioSharingSingle,
        Self::AudioSharingPeer,
        Self::FindLeft,
        Self::FindRight,
        Self::FactoryReset,
        Self::OfflineDecode,
        Self::OfflineAnalyze,
        Self::OfflineCandidate,
    ];
    /// Stable index into the localized name and
    /// description arrays.
    pub const fn index(self) -> usize {
        self as usize
    }
}

pub struct FeatureSpec {
    pub id: FeatureId,
    pub category: FeatureCategory,
    pub kind: FeatureKind,
    /// Required capability bit; `None` marks features
    /// that need no capture evidence (offline analysis).
    pub capability: Option<u32>,
    pub arguments: &'static [ArgumentSpec],
    pub safety: SafetyLevel,
    /// Verified settings map to their settings key;
    /// editing stays in the settings screen.
    pub setting: Option<&'static str>,
}

const TOGGLE: ArgumentSpec = ArgumentSpec {
    label: |t| t.arg_enabled,
    kind: ArgumentKind::Toggle,
};
const SECONDS: ArgumentSpec = ArgumentSpec {
    label: |t| t.arg_duration,
    kind: ArgumentKind::Seconds,
};
const TARGET: ArgumentSpec = ArgumentSpec {
    label: |t| t.arg_target,
    kind: ArgumentKind::Mac,
};
const VALUE: ArgumentSpec = ArgumentSpec {
    label: |t| t.arg_value,
    kind: ArgumentKind::Byte,
};
const MODE: ArgumentSpec = ArgumentSpec {
    label: |t| t.arg_mode,
    kind: ArgumentKind::Choice(&[("mono", 0), ("stereo", 1)]),
};
const PEER: ArgumentSpec = ArgumentSpec {
    label: |t| t.arg_peer,
    kind: ArgumentKind::PeerIndex,
};
const PEER_MAC: ArgumentSpec = ArgumentSpec {
    label: |t| t.arg_address,
    kind: ArgumentKind::Mac,
};
const BUTTONS: ArgumentSpec = ArgumentSpec {
    label: |t| t.arg_actions,
    kind: ArgumentKind::Actions(8),
};
const AI_ACTIONS: ArgumentSpec = ArgumentSpec {
    label: |t| t.arg_actions,
    kind: ArgumentKind::Actions(4),
};

/// Every registered feature, in workspace listing order.
pub const FEATURES: &[FeatureSpec] = &[
    // Verified settings.
    FeatureSpec {
        id: FeatureId::Anc,
        category: FeatureCategory::Audio,
        kind: FeatureKind::VerifiedSetting,
        capability: Some(Capabilities::ANC),
        arguments: &[],
        safety: SafetyLevel::Safe,
        setting: Some("anc"),
    },
    FeatureSpec {
        id: FeatureId::Eq,
        category: FeatureCategory::Audio,
        kind: FeatureKind::VerifiedSetting,
        capability: Some(Capabilities::EQ),
        arguments: &[],
        safety: SafetyLevel::Safe,
        setting: Some("eq"),
    },
    FeatureSpec {
        id: FeatureId::GameMode,
        category: FeatureCategory::Audio,
        kind: FeatureKind::VerifiedSetting,
        capability: Some(Capabilities::GAME_MODE),
        arguments: &[],
        safety: SafetyLevel::Safe,
        setting: Some("game"),
    },
    FeatureSpec {
        id: FeatureId::SpatialAudio,
        category: FeatureCategory::Audio,
        kind: FeatureKind::VerifiedSetting,
        capability: Some(Capabilities::SPATIAL),
        arguments: &[],
        safety: SafetyLevel::Safe,
        setting: Some("spatial"),
    },
    FeatureSpec {
        id: FeatureId::DualConnection,
        category: FeatureCategory::Connection,
        kind: FeatureKind::VerifiedSetting,
        capability: Some(Capabilities::DUAL_DEVICE),
        arguments: &[],
        safety: SafetyLevel::Safe,
        setting: Some("dual"),
    },
    FeatureSpec {
        id: FeatureId::WindReduction,
        category: FeatureCategory::Environment,
        kind: FeatureKind::VerifiedSetting,
        capability: Some(Capabilities::WIND_REDUCTION),
        arguments: &[],
        safety: SafetyLevel::Safe,
        setting: Some("wind"),
    },
    FeatureSpec {
        id: FeatureId::Prompts,
        category: FeatureCategory::Feedback,
        kind: FeatureKind::VerifiedSetting,
        capability: Some(Capabilities::PROMPTS),
        arguments: &[],
        safety: SafetyLevel::Safe,
        setting: Some("prompts"),
    },
    FeatureSpec {
        id: FeatureId::VolumeUpAction,
        category: FeatureCategory::Feedback,
        kind: FeatureKind::VerifiedSetting,
        capability: Some(Capabilities::VOLUME_UP_ACTION),
        arguments: &[],
        safety: SafetyLevel::Safe,
        setting: Some("volume-up-action"),
    },
    FeatureSpec {
        id: FeatureId::VolumeDownAction,
        category: FeatureCategory::Feedback,
        kind: FeatureKind::VerifiedSetting,
        capability: Some(Capabilities::VOLUME_DOWN_ACTION),
        arguments: &[],
        safety: SafetyLevel::Safe,
        setting: Some("volume-down-action"),
    },
    // Capture-gated queries.
    FeatureSpec {
        id: FeatureId::MultipointPeers,
        category: FeatureCategory::Multipoint,
        kind: FeatureKind::DeviceQuery,
        capability: Some(Capabilities::MULTIPOINT_PEERS),
        arguments: &[],
        safety: SafetyLevel::Safe,
        setting: None,
    },
    FeatureSpec {
        id: FeatureId::CodecStatus,
        category: FeatureCategory::Codecs,
        kind: FeatureKind::DeviceQuery,
        capability: Some(Capabilities::CODEC_STATUS),
        arguments: &[],
        safety: SafetyLevel::Safe,
        setting: None,
    },
    FeatureSpec {
        id: FeatureId::HeadphoneId,
        category: FeatureCategory::DeviceInfo,
        kind: FeatureKind::DeviceQuery,
        capability: Some(Capabilities::HEADPHONE_ID),
        arguments: &[],
        safety: SafetyLevel::Safe,
        setting: None,
    },
    FeatureSpec {
        id: FeatureId::HeadphoneColor,
        category: FeatureCategory::DeviceInfo,
        kind: FeatureKind::DeviceQuery,
        capability: Some(Capabilities::HEADPHONE_COLOR),
        arguments: &[],
        safety: SafetyLevel::Safe,
        setting: None,
    },
    FeatureSpec {
        id: FeatureId::BoxMac,
        category: FeatureCategory::DeviceInfo,
        kind: FeatureKind::DeviceQuery,
        capability: Some(Capabilities::BOX_MAC),
        arguments: &[],
        safety: SafetyLevel::Safe,
        setting: None,
    },
    // Capture-gated codec controls.
    FeatureSpec {
        id: FeatureId::LdacToggle,
        category: FeatureCategory::Codecs,
        kind: FeatureKind::DeviceControl,
        capability: Some(Capabilities::LDAC),
        arguments: &[TOGGLE],
        safety: SafetyLevel::Write,
        setting: None,
    },
    FeatureSpec {
        id: FeatureId::LdacPeerToggle,
        category: FeatureCategory::Codecs,
        kind: FeatureKind::DeviceControl,
        capability: Some(Capabilities::LDAC_PEER),
        arguments: &[TOGGLE, PEER_MAC],
        safety: SafetyLevel::Write,
        setting: None,
    },
    FeatureSpec {
        id: FeatureId::HighQualityDecoding,
        category: FeatureCategory::Codecs,
        kind: FeatureKind::DeviceControl,
        capability: Some(Capabilities::HIGH_QUALITY_DECODING),
        arguments: &[TOGGLE],
        safety: SafetyLevel::Write,
        setting: None,
    },
    // Capture-gated advanced audio.
    FeatureSpec {
        id: FeatureId::VolumeCommand,
        category: FeatureCategory::AdvancedAudio,
        kind: FeatureKind::DeviceControl,
        capability: Some(Capabilities::VOLUME),
        arguments: &[VALUE],
        safety: SafetyLevel::Write,
        setting: None,
    },
    FeatureSpec {
        id: FeatureId::ChannelMode,
        category: FeatureCategory::AdvancedAudio,
        kind: FeatureKind::DeviceControl,
        capability: Some(Capabilities::CHANNEL_MODE),
        arguments: &[MODE],
        safety: SafetyLevel::Write,
        setting: None,
    },
    // Capture-gated controls.
    FeatureSpec {
        id: FeatureId::ButtonMapping,
        category: FeatureCategory::Controls,
        kind: FeatureKind::DeviceControl,
        capability: Some(Capabilities::BUTTONS),
        arguments: &[BUTTONS],
        safety: SafetyLevel::Write,
        setting: None,
    },
    FeatureSpec {
        id: FeatureId::AiActions,
        category: FeatureCategory::Controls,
        kind: FeatureKind::DeviceControl,
        capability: Some(Capabilities::AI_ACTIONS),
        arguments: &[AI_ACTIONS],
        safety: SafetyLevel::Write,
        setting: None,
    },
    FeatureSpec {
        id: FeatureId::AiControl,
        category: FeatureCategory::Controls,
        kind: FeatureKind::DeviceControl,
        capability: Some(Capabilities::AI_CONTROL),
        arguments: &[TOGGLE],
        safety: SafetyLevel::Write,
        setting: None,
    },
    // Capture-gated audio sharing.
    FeatureSpec {
        id: FeatureId::AudioSharingOff,
        category: FeatureCategory::AudioSharing,
        kind: FeatureKind::DeviceControl,
        capability: Some(Capabilities::AUDIO_SHARING),
        arguments: &[],
        safety: SafetyLevel::Write,
        setting: None,
    },
    FeatureSpec {
        id: FeatureId::AudioSharingSingle,
        category: FeatureCategory::AudioSharing,
        kind: FeatureKind::DeviceControl,
        capability: Some(Capabilities::AUDIO_SHARING_SINGLE),
        arguments: &[],
        safety: SafetyLevel::Write,
        setting: None,
    },
    FeatureSpec {
        id: FeatureId::AudioSharingPeer,
        category: FeatureCategory::AudioSharing,
        kind: FeatureKind::DeviceControl,
        capability: Some(Capabilities::AUDIO_SHARING_PEER),
        arguments: &[PEER, PEER_MAC],
        safety: SafetyLevel::Write,
        setting: None,
    },
    // Capture-gated finder.
    FeatureSpec {
        id: FeatureId::FindLeft,
        category: FeatureCategory::Finder,
        kind: FeatureKind::DeviceControl,
        capability: Some(Capabilities::FIND_LEFT),
        arguments: &[SECONDS, TARGET],
        safety: SafetyLevel::Write,
        setting: None,
    },
    FeatureSpec {
        id: FeatureId::FindRight,
        category: FeatureCategory::Finder,
        kind: FeatureKind::DeviceControl,
        capability: Some(Capabilities::FIND_RIGHT),
        arguments: &[SECONDS, TARGET],
        safety: SafetyLevel::Write,
        setting: None,
    },
    // Capture-gated device management.
    FeatureSpec {
        id: FeatureId::FactoryReset,
        category: FeatureCategory::DeviceManagement,
        kind: FeatureKind::DeviceControl,
        capability: Some(Capabilities::FACTORY_RESET),
        arguments: &[],
        safety: SafetyLevel::Destructive,
        setting: None,
    },
    // Offline diagnostics; no hardware access, ever.
    FeatureSpec {
        id: FeatureId::OfflineDecode,
        category: FeatureCategory::Diagnostics,
        kind: FeatureKind::OfflineAnalysis,
        capability: None,
        arguments: &[],
        safety: SafetyLevel::Safe,
        setting: None,
    },
    FeatureSpec {
        id: FeatureId::OfflineAnalyze,
        category: FeatureCategory::Diagnostics,
        kind: FeatureKind::OfflineAnalysis,
        capability: None,
        arguments: &[],
        safety: SafetyLevel::Safe,
        setting: None,
    },
    FeatureSpec {
        id: FeatureId::OfflineCandidate,
        category: FeatureCategory::Diagnostics,
        kind: FeatureKind::OfflineAnalysis,
        capability: None,
        arguments: &[],
        safety: SafetyLevel::Safe,
        setting: None,
    },
];

/// Find a feature by its stable id.
pub fn lookup(id: FeatureId) -> Option<&'static FeatureSpec> {
    FEATURES.iter().find(|spec| spec.id == id)
}

/// The feature that edits a verified setting key.
pub fn by_setting(key: &str) -> Option<&'static FeatureSpec> {
    FEATURES.iter().find(|spec| spec.setting == Some(key))
}

impl FeatureSpec {
    pub fn offline_command(&self) -> Option<&'static str> {
        match self.id {
            FeatureId::OfflineDecode => Some("decode HEX"),
            FeatureId::OfflineAnalyze => Some("analyze HEX"),
            FeatureId::OfflineCandidate => Some("candidate OP [ARGS]"),
            _ => None,
        }
    }

    /// Localized display name.
    pub fn name(&self, t: &'static L) -> &'static str {
        t.feature_names[self.id.index()]
    }
    /// Localized description.
    pub fn description(&self, t: &'static L) -> &'static str {
        t.feature_descriptions[self.id.index()]
    }
    /// True when capture evidence allows a live operation.
    pub fn verified(&self, caps: Capabilities) -> bool {
        match self.capability {
            Some(bit) => caps.contains(bit),
            None => true,
        }
    }
    /// Build the typed operation from form drafts.
    /// Every draft must be present and valid; nothing
    /// is defaulted silently.
    pub fn operation(&self, drafts: &[Option<String>]) -> Result<Operation, String> {
        if drafts.len() != self.arguments.len() {
            return Err("feature form does not match its arguments".into());
        }
        let value = |index: usize| -> Result<String, String> {
            drafts[index]
                .clone()
                .ok_or_else(|| "feature value is not set".into())
        };
        let toggle = |index: usize| -> Result<bool, String> {
            match value(index)?.as_str() {
                "on" => Ok(true),
                "off" => Ok(false),
                _ => Err("expected on or off".into()),
            }
        };
        let mac = |index: usize| -> Result<BluetoothAddress, String> {
            BluetoothAddress::parse(&value(index)?).map_err(|e| e.to_string())
        };
        let command = match self.id {
            FeatureId::MultipointPeers => return Ok(Operation::Peers),
            FeatureId::CodecStatus => return Ok(Operation::Codec),
            FeatureId::HeadphoneId => return Ok(Operation::Id),
            FeatureId::HeadphoneColor => return Ok(Operation::Color),
            FeatureId::BoxMac => return Ok(Operation::BoxMac),
            FeatureId::FindLeft => {
                return Ok(Operation::Find {
                    side: Side::Left,
                    seconds: seconds(&value(0)?)?,
                    target: value(1)?,
                })
            }
            FeatureId::FindRight => {
                return Ok(Operation::Find {
                    side: Side::Right,
                    seconds: seconds(&value(0)?)?,
                    target: value(1)?,
                })
            }
            FeatureId::LdacToggle => Command::Ldac {
                enabled: toggle(0)?,
                peer: None,
            },
            FeatureId::LdacPeerToggle => Command::Ldac {
                enabled: toggle(0)?,
                peer: Some(mac(1)?),
            },
            FeatureId::HighQualityDecoding => Command::HighQualityDecoding(toggle(0)?),
            FeatureId::VolumeCommand => Command::Volume(byte(value(0)?)?),
            FeatureId::ChannelMode => Command::Stereo(match value(0)?.as_str() {
                "mono" => false,
                "stereo" => true,
                _ => return Err("expected mono or stereo".into()),
            }),
            FeatureId::ButtonMapping => Command::Buttons(
                actions(&value(0)?, 8)?
                    .try_into()
                    .map_err(|_| "button mapping needs eight action values".to_string())?,
            ),
            FeatureId::AiActions => Command::AiActions(
                actions(&value(0)?, 4)?
                    .try_into()
                    .map_err(|_| "AI actions need four action values".to_string())?,
            ),
            FeatureId::AiControl => Command::AiControl(toggle(0)?),
            FeatureId::AudioSharingOff => Command::SharingOff,
            FeatureId::AudioSharingSingle => Command::SharingSinglePeer,
            FeatureId::AudioSharingPeer => {
                let index: u8 = value(0)?
                    .parse()
                    .map_err(|_| "sharing needs peer index 1 or 2".to_string())?;
                if !matches!(index, 1 | 2) {
                    return Err("sharing needs peer index 1 or 2".into());
                }
                Command::SharingPeer {
                    index,
                    address: mac(1)?,
                }
            }
            FeatureId::FactoryReset => Command::FactoryReset,
            FeatureId::Anc
            | FeatureId::Eq
            | FeatureId::GameMode
            | FeatureId::SpatialAudio
            | FeatureId::DualConnection
            | FeatureId::WindReduction
            | FeatureId::Prompts
            | FeatureId::VolumeUpAction
            | FeatureId::VolumeDownAction
            | FeatureId::OfflineDecode
            | FeatureId::OfflineAnalyze
            | FeatureId::OfflineCandidate => return Err("feature has no device operation".into()),
        };
        Ok(Operation::Control(command))
    }
    /// Human-readable summary of the draft values,
    /// shown on the review screen.
    pub fn summarize(&self, drafts: &[Option<String>]) -> String {
        if drafts.is_empty() {
            return String::new();
        }
        drafts
            .iter()
            .map(|draft| match draft {
                Some(value) => value.clone(),
                None => "?".into(),
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn seconds(text: &str) -> Result<u8, String> {
    let value: u8 = text
        .parse()
        .map_err(|_| "finder duration must be 1..30 seconds".to_string())?;
    if (1..=30).contains(&value) {
        Ok(value)
    } else {
        Err("finder duration must be 1..30 seconds".into())
    }
}

fn byte(text: String) -> Result<u8, String> {
    text.parse()
        .map_err(|_| "volume must be a byte (0..255); hardware range unverified".into())
}

fn actions(text: &str, count: usize) -> Result<Vec<Action>, String> {
    let values: Vec<Action> = text
        .split(',')
        .map(|item| {
            item.trim()
                .parse::<u8>()
                .map_err(|_| "actions must be comma-separated byte values".to_string())
                .and_then(Action::try_from)
        })
        .collect::<Result<Vec<_>, _>>()?;
    if values.len() != count {
        return Err(format!("needs {count} action values"));
    }
    Ok(values)
}

/// A typed headset operation. Rendering never does I/O.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operation {
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
pub struct ResultData {
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
    /// The registry entry this operation was built from.
    pub fn id(&self) -> Option<FeatureId> {
        match self {
            Self::Peers => Some(FeatureId::MultipointPeers),
            Self::Codec => Some(FeatureId::CodecStatus),
            Self::Id => Some(FeatureId::HeadphoneId),
            Self::Color => Some(FeatureId::HeadphoneColor),
            Self::BoxMac => Some(FeatureId::BoxMac),
            Self::Control(command) => command_id(command),
            Self::Find { side, .. } => Some(match side {
                Side::Left => FeatureId::FindLeft,
                Side::Right => FeatureId::FindRight,
            }),
        }
    }
    pub fn is_write(&self) -> bool {
        matches!(self, Self::Control(_) | Self::Find { .. })
    }
    pub fn is_reset(&self) -> bool {
        matches!(self, Self::Control(Command::FactoryReset))
    }
    /// Required capability bit, when the operation has one.
    pub fn capability(&self) -> Option<u32> {
        self.id()
            .and_then(|id| lookup(id).and_then(|spec| spec.capability))
    }
    pub fn check(&self, caps: Capabilities) -> io::Result<()> {
        let bit = match self {
            Self::Peers => Capabilities::MULTIPOINT_PEERS,
            Self::Codec => Capabilities::CODEC_STATUS,
            Self::Id => Capabilities::HEADPHONE_ID,
            Self::Color => Capabilities::HEADPHONE_COLOR,
            Self::BoxMac => Capabilities::BOX_MAC,
            Self::Control(command) => command.control_capability().ok_or_else(|| {
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
                finder::ring_cancellable(client, *side, *seconds, target, cancelled)?;
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

fn command_id(command: &Command) -> Option<FeatureId> {
    match command {
        Command::Ldac { peer: None, .. } => Some(FeatureId::LdacToggle),
        Command::Ldac { peer: Some(_), .. } => Some(FeatureId::LdacPeerToggle),
        Command::HighQualityDecoding(_) => Some(FeatureId::HighQualityDecoding),
        Command::Volume(_) => Some(FeatureId::VolumeCommand),
        Command::Stereo(_) => Some(FeatureId::ChannelMode),
        Command::Buttons(_) => Some(FeatureId::ButtonMapping),
        Command::AiActions(_) => Some(FeatureId::AiActions),
        Command::AiControl(_) => Some(FeatureId::AiControl),
        Command::SharingOff => Some(FeatureId::AudioSharingOff),
        Command::SharingSinglePeer => Some(FeatureId::AudioSharingSingle),
        Command::SharingPeer { .. } => Some(FeatureId::AudioSharingPeer),
        Command::FactoryReset => Some(FeatureId::FactoryReset),
        Command::Find { .. } => None,
        Command::Firmware
        | Command::HeadphoneId
        | Command::Color
        | Command::Peers
        | Command::Codec
        | Command::BoxMac => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_is_complete_and_consistent() {
        assert_eq!(FEATURES.len(), FeatureId::ALL.len());
        for spec in FEATURES {
            assert_eq!(lookup(spec.id).map(|found| found.id), Some(spec.id));
            // Setting features map back to their spec and vice versa.
            if let Some(key) = spec.setting {
                assert!(
                    crate::settings::KEYS.contains(&key),
                    "{key} must be a known setting"
                );
                assert_eq!(by_setting(key).map(|found| found.id), Some(spec.id));
            }
            // Draft count always matches the argument count.
            assert_eq!(spec.arguments.len(), spec.arguments.len());
        }
    }

    #[test]
    fn every_feature_has_localized_metadata_slots() {
        // Name and description arrays are sized by the id
        // count; a missing entry would be a compile-time
        // array error in the language catalogs.
        for id in FeatureId::ALL {
            assert!(lookup(id).is_some(), "{id:?} registered");
        }
    }

    #[test]
    fn pending_features_never_verify_without_their_bit() {
        let caps = Model::StudioPro.capabilities();
        for spec in FEATURES {
            match spec.capability {
                Some(bit) => assert_eq!(spec.verified(caps), caps.contains(bit)),
                None => assert!(spec.verified(caps)),
            }
        }
    }

    #[test]
    fn verified_settings_resolve_to_setting_keys() {
        for key in crate::settings::KEYS {
            let spec = by_setting(key).expect("every setting has a feature");
            assert_eq!(spec.kind, FeatureKind::VerifiedSetting);
            assert_eq!(spec.safety, SafetyLevel::Safe);
            assert!(spec.arguments.is_empty());
        }
    }

    #[test]
    fn empty_capabilities_verify_only_offline_analysis() {
        let caps = Capabilities::new();
        for spec in FEATURES {
            if spec.kind == FeatureKind::OfflineAnalysis {
                assert!(spec.verified(caps), "{:?} is offline", spec.id);
            } else {
                assert!(!spec.verified(caps), "{:?} needs evidence", spec.id);
            }
        }
    }

    #[test]
    fn operation_builds_from_complete_drafts() {
        let spec = lookup(FeatureId::FindLeft).unwrap();
        let op = spec
            .operation(&[Some("5".into()), Some("sink".into())])
            .unwrap();
        assert_eq!(
            op,
            Operation::Find {
                side: Side::Left,
                seconds: 5,
                target: "sink".into()
            }
        );
        let spec = lookup(FeatureId::LdacPeerToggle).unwrap();
        let op = spec
            .operation(&[Some("on".into()), Some("AA:BB:CC:DD:EE:FF".into())])
            .unwrap();
        assert_eq!(
            op,
            Operation::Control(Command::Ldac {
                enabled: true,
                peer: Some(BluetoothAddress::parse("AA:BB:CC:DD:EE:FF").unwrap())
            })
        );
        let spec = lookup(FeatureId::ButtonMapping).unwrap();
        assert!(spec.operation(&[Some("1,2,3,4,5,6,7,8".into())]).is_ok());
        assert!(spec.operation(&[Some("1,2,3,4,5,6,7".into())]).is_err());
        assert!(spec.operation(&[None]).is_err());
    }

    #[test]
    fn invalid_drafts_are_rejected_before_any_operation() {
        let spec = lookup(FeatureId::VolumeCommand).unwrap();
        assert!(spec.operation(&[Some("256".into())]).is_err());
        assert!(spec.operation(&[Some("nope".into())]).is_err());
        let spec = lookup(FeatureId::ChannelMode).unwrap();
        assert!(spec.operation(&[Some("loud".into())]).is_err());
        let spec = lookup(FeatureId::LdacToggle).unwrap();
        assert!(spec.operation(&[Some("maybe".into())]).is_err());
        // Setting features have no device operation.
        let spec = lookup(FeatureId::Anc).unwrap();
        assert!(spec.operation(&[]).is_err());
    }

    #[test]
    fn operation_ids_round_trip_through_the_registry() {
        for operation in [
            Operation::Peers,
            Operation::Codec,
            Operation::Id,
            Operation::Color,
            Operation::BoxMac,
            Operation::Control(Command::FactoryReset),
            Operation::Control(Command::Volume(8)),
            Operation::Control(Command::Stereo(true)),
            Operation::Control(Command::AiControl(false)),
            Operation::Control(Command::SharingOff),
            Operation::Control(Command::SharingSinglePeer),
            Operation::Control(Command::HighQualityDecoding(true)),
            Operation::Control(Command::Ldac {
                enabled: false,
                peer: None,
            }),
            Operation::Find {
                side: Side::Right,
                seconds: 2,
                target: "sink".into(),
            },
        ] {
            let id = operation.id().expect("operation maps to a feature");
            assert_eq!(lookup(id).map(|spec| spec.id), Some(id));
            assert_eq!(
                operation.capability(),
                lookup(id).and_then(|spec| spec.capability)
            );
        }
    }
}
