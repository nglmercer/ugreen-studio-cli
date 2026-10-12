//! Model-specific settings. These instruction IDs must NOT be used for HiTune Max5c.
//!
//! `SPECS` is the single authority for setting metadata: CLI
//! parsing, `ugreen commands`, readback verification and the
//! TUI all derive from these records. Keys, value names and
//! wire bytes are protocol identifiers and are never
//! translated; only error prose is localized.
use crate::{
    i18n::{Lang, L},
    protocol,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub instruction: u8,
    pub payload: Vec<u8>,
}

/// One verified setting: its instruction, value encoding and
/// readback semantics. Adding a setting means adding one
/// record here; every interface derives from it.
pub struct SettingSpec {
    /// Canonical profile and CLI key.
    pub key: &'static str,
    /// Wire instruction ID.
    pub instruction: u8,
    /// Accepted (value, wire byte) pairs, in display order.
    pub values: &'static [(&'static str, u8)],
    /// Device-info byte index used for readback matching.
    pub readback_index: u8,
    /// ANC off/ambient match the low nibble only: the device
    /// preserves its previous ANC depth on some firmware.
    pub nibble_masked: bool,
    /// spatial audio writes are not acknowledged by the device.
    pub expects_ack: bool,
    /// Consecutive settings sharing a group render on one
    /// `ugreen commands` line.
    pub group: Option<&'static str>,
    /// Localized invalid-value error for this setting.
    pub invalid_value: fn(&'static L) -> &'static str,
}

const ON_OFF: &[(&str, u8)] = &[("on", 1), ("off", 0)];
const ACTIONS: &[(&str, u8)] = &[("none", 0), ("next", 4), ("previous", 5)];

pub const SPECS: &[SettingSpec] = &[
    SettingSpec {
        key: "anc",
        instruction: 9,
        values: &[
            ("off", 0xa0),
            ("ultra", 0xa1),
            ("general", 0xb1),
            ("gentle", 0xc1),
            ("adaptive", 0xd1),
            ("ambient", 0xa2),
        ],
        readback_index: 3,
        nibble_masked: true,
        expects_ack: true,
        group: None,
        invalid_value: |t| t.set_bad_anc,
    },
    SettingSpec {
        key: "eq",
        instruction: 5,
        values: &[
            ("classic", 0),
            ("jazz", 1),
            ("electronic", 2),
            ("pop", 3),
            ("classical", 4),
            ("rock", 5),
            ("bass", 6),
            ("treble", 7),
        ],
        readback_index: 4,
        nibble_masked: false,
        expects_ack: true,
        group: None,
        invalid_value: |t| t.set_bad_eq,
    },
    SettingSpec {
        key: "game",
        instruction: 8,
        values: ON_OFF,
        readback_index: 6,
        nibble_masked: false,
        expects_ack: true,
        group: Some("on-off"),
        invalid_value: |t| t.set_bad_onoff,
    },
    SettingSpec {
        key: "spatial",
        instruction: 18,
        values: ON_OFF,
        readback_index: 20,
        nibble_masked: false,
        expects_ack: false,
        group: Some("on-off"),
        invalid_value: |t| t.set_bad_onoff,
    },
    SettingSpec {
        key: "dual",
        instruction: 6,
        values: ON_OFF,
        readback_index: 5,
        nibble_masked: false,
        expects_ack: true,
        group: Some("on-off"),
        invalid_value: |t| t.set_bad_onoff,
    },
    SettingSpec {
        key: "wind",
        instruction: 23,
        values: ON_OFF,
        readback_index: 25,
        nibble_masked: false,
        expects_ack: true,
        group: Some("on-off"),
        invalid_value: |t| t.set_bad_onoff,
    },
    SettingSpec {
        key: "prompts",
        instruction: 12,
        values: &[("voice", 0), ("beeps", 2)],
        readback_index: 16,
        nibble_masked: false,
        expects_ack: true,
        group: None,
        invalid_value: |t| t.set_bad_prompts,
    },
    SettingSpec {
        key: "volume-up-action",
        instruction: 20,
        values: ACTIONS,
        readback_index: 22,
        nibble_masked: false,
        expects_ack: true,
        group: Some("actions"),
        invalid_value: |t| t.set_bad_button,
    },
    SettingSpec {
        key: "volume-down-action",
        instruction: 21,
        values: ACTIONS,
        readback_index: 23,
        nibble_masked: false,
        expects_ack: true,
        group: Some("actions"),
        invalid_value: |t| t.set_bad_button,
    },
];

fn spec_of(key: &str) -> Option<&'static SettingSpec> {
    SPECS.iter().find(|spec| spec.key == key)
}

/// The `ugreen commands` listing, generated from `SPECS`.
/// Keys and value names are protocol identifiers.
pub fn commands_help() -> String {
    let mut lines = Vec::new();
    let mut index = 0;
    while index < SPECS.len() {
        let spec = &SPECS[index];
        match spec.group {
            Some(group) => {
                let mut keys = vec![spec.key];
                let mut next = index + 1;
                while next < SPECS.len() && SPECS[next].group == Some(group) {
                    keys.push(SPECS[next].key);
                    next += 1;
                }
                lines.push(format!("{}: {}", keys.join(", "), value_list(spec.values)));
                index = next;
            }
            None => {
                lines.push(format!("{}: {}", spec.key, value_list(spec.values)));
                index += 1;
            }
        }
    }
    lines.join("\n")
}

fn value_list(values: &[(&'static str, u8)]) -> String {
    values
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>()
        .join("|")
}

impl Setting {
    pub fn parse(key: &str, value: &str) -> Result<Self, String> {
        Self::parse_in(Lang::En, key, value)
    }
    pub fn parse_in(lang: Lang, key: &str, value: &str) -> Result<Self, String> {
        let t = crate::i18n::txt(lang);
        let spec = spec_of(key).ok_or_else(|| {
            t.set_unknown
                .replacen("{}", key, 1)
                .replacen("{}", &commands_help(), 1)
        })?;
        let byte = spec
            .values
            .iter()
            .find(|(name, _)| *name == value)
            .map(|(_, byte)| *byte)
            .ok_or_else(|| (spec.invalid_value)(t).to_string())?;
        Ok(Self {
            key: key.into(),
            value: value.into(),
            instruction: spec.instruction,
            payload: vec![byte],
        })
    }
    pub fn packet(&self) -> Vec<u8> {
        protocol::request(self.instruction, &self.payload).expect("known setting payload fits")
    }
    pub fn expects_ack(&self) -> bool {
        spec_of(&self.key)
            .map(|spec| spec.expects_ack)
            .unwrap_or(true)
    }
    pub fn matches(&self, info: &StudioProState) -> bool {
        let Some(spec) = spec_of(&self.key) else {
            return false;
        };
        let Some(actual) = info.raw.get(spec.readback_index as usize).copied() else {
            return false;
        };
        match (spec.nibble_masked, self.payload[0] & 15) {
            // Off/ambient preserve the device's previous ANC
            // depth on some firmware, so the high nibble may
            // differ; ultra shares its nibble and matches exactly.
            (true, nibble) if nibble != 1 => actual & 15 == nibble,
            _ => actual == self.payload[0],
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioProState {
    pub raw: Vec<u8>,
}
impl StudioProState {
    pub fn new(raw: Vec<u8>) -> Result<Self, String> {
        if raw.len() < 8 {
            return Err(format!(
                "device-info payload too short: {} bytes (need at least 8)",
                raw.len()
            ));
        }
        Ok(Self { raw })
    }
    pub fn battery(&self) -> Option<u8> {
        self.raw.first().copied().filter(|v| (1..=100).contains(v))
    }
    pub fn value(&self, key: &str) -> Option<String> {
        let spec = spec_of(key)?;
        let raw = *self.raw.get(spec.readback_index as usize)?;
        let byte = if spec.nibble_masked {
            match raw & 15 {
                0 => return Some("off".into()),
                2 => return Some("ambient".into()),
                _ => raw,
            }
        } else {
            raw
        };
        spec.values
            .iter()
            .find(|(_, wire)| *wire == byte)
            .map(|(name, _)| (*name).into())
    }
    pub fn profile(&self) -> String {
        let mut out = "# UGREEN Studio Pro profile v1\nmodel=studio-pro\n".to_string();
        for key in KEYS {
            if let Some(value) = self.value(key) {
                out.push_str(&format!("{key}={value}\n"));
            }
        }
        out
    }
}
pub const KEYS: [&str; 9] = [
    "anc",
    "eq",
    "game",
    "spatial",
    "dual",
    "wind",
    "prompts",
    "volume-up-action",
    "volume-down-action",
];

/// Read a bounded, simple profile; reject duplicates, wrong models and unknown commands before any write.
pub fn parse_profile(text: &str) -> Result<Vec<Setting>, String> {
    parse_profile_in(Lang::En, text)
}
pub fn parse_profile_in(lang: Lang, text: &str) -> Result<Vec<Setting>, String> {
    let t = crate::i18n::txt(lang);
    if text.len() > 16_384 {
        return Err(t.profile_big.into());
    }
    let mut settings = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut model = false;
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| t.profile_line.replacen("{}", &format!("{}", i + 1), 1))?;
        let (key, value) = (key.trim(), value.trim());
        if !seen.insert(key) {
            return Err(t.profile_dup.replacen("{}", key, 1));
        }
        if key == "model" {
            if value != "studio-pro" {
                return Err(t.profile_model.into());
            }
            model = true;
        } else {
            settings.push(Setting::parse_in(lang, key, value)?);
        }
    }
    if !model {
        return Err(t.profile_need_model.into());
    }
    if settings.is_empty() {
        return Err(t.profile_empty.into());
    }
    Ok(settings)
}
pub fn firmware(payload: &[u8]) -> Option<String> {
    let first = payload.get(..3)?;
    let version = if first.iter().any(|v| *v != 0) {
        first
    } else {
        payload.get(3..6)?
    };
    Some(
        version
            .iter()
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join("."),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn model_specific_commands() {
        assert_eq!(Setting::parse("game", "on").unwrap().instruction, 8);
        assert_eq!(Setting::parse("dual", "on").unwrap().instruction, 6);
        assert_eq!(Setting::parse("wind", "on").unwrap().instruction, 23);
        assert!(Setting::parse("find", "left").is_err());
        assert!(Setting::parse("eq", "loud").is_err());
    }
    #[test]
    fn profile_validation() {
        assert_eq!(
            parse_profile("model=studio-pro\nanc=ultra\neq=bass")
                .unwrap()
                .len(),
            2
        );
        for bad in [
            "anc=ultra",
            "model=max5c\nanc=ultra",
            "model=studio-pro\nanc=off\nanc=ultra",
            "model=studio-pro\nraw=0100",
            "model=studio-pro",
        ] {
            assert!(parse_profile(bad).is_err(), "{bad}");
        }
    }
    #[test]
    fn info_unknowns_are_not_defaults() {
        assert!(StudioProState::new(vec![0; 7]).is_err());
        let info = StudioProState::new(vec![255; 8]).unwrap();
        assert_eq!(info.battery(), None);
        assert_eq!(info.value("eq"), None);
        assert_eq!(info.value("spatial"), None);
        assert!(!info.profile().contains("spatial="));
    }
    #[test]
    fn firmware_fallback() {
        assert_eq!(firmware(&[0, 0, 0, 1, 2, 3]).as_deref(), Some("1.2.3"));
        assert_eq!(firmware(&[1, 2]), None);
    }
    #[test]
    fn profile_roundtrip() {
        let mut raw = vec![0; 30];
        raw[0] = 20;
        raw[3] = 0xa1;
        raw[16] = 2;
        let info = StudioProState::new(raw).unwrap();
        for setting in parse_profile(&info.profile()).unwrap() {
            assert!(setting.matches(&info));
        }
    }
}
