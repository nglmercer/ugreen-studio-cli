//! Model-specific settings. These instruction IDs must NOT be used for HiTune Max5c.
use crate::{i18n::Lang, protocol};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub instruction: u8,
    pub payload: Vec<u8>,
}
const EQ: [&str; 8] = [
    "classic",
    "jazz",
    "electronic",
    "pop",
    "classical",
    "rock",
    "bass",
    "treble",
];
const ANC: [(&str, u8); 6] = [
    ("off", 0xa0),
    ("ultra", 0xa1),
    ("general", 0xb1),
    ("gentle", 0xc1),
    ("adaptive", 0xd1),
    ("ambient", 0xa2),
];
pub const HELP: &str = "anc: off|ultra|general|gentle|adaptive|ambient\neq: classic|jazz|electronic|pop|classical|rock|bass|treble\ngame, spatial, dual, wind: on|off\nprompts: voice|beeps\nvolume-up-action, volume-down-action: none|next|previous";
impl Setting {
    pub fn parse(key: &str, value: &str) -> Result<Self, String> {
        Self::parse_in(Lang::En, key, value)
    }
    pub fn parse_in(lang: Lang, key: &str, value: &str) -> Result<Self, String> {
        let t = crate::i18n::txt(lang);
        let (instruction, byte) = match key {
            "anc" => (
                9,
                ANC.iter()
                    .find(|(name, _)| *name == value)
                    .map(|(_, b)| *b)
                    .ok_or(t.set_bad_anc)?,
            ),
            "eq" => (
                5,
                EQ.iter()
                    .position(|name| *name == value)
                    .ok_or(t.set_bad_eq)? as u8,
            ),
            "game" | "spatial" | "dual" | "wind" => {
                let byte = match value {
                    "on" => 1,
                    "off" => 0,
                    _ => return Err(t.set_bad_onoff.into()),
                };
                (
                    match key {
                        "game" => 8,
                        "spatial" => 18,
                        "dual" => 6,
                        _ => 23,
                    },
                    byte,
                )
            }
            "prompts" => (
                12,
                match value {
                    "voice" => 0,
                    "beeps" => 2,
                    _ => return Err(t.set_bad_prompts.into()),
                },
            ),
            "volume-up-action" | "volume-down-action" => (
                if key == "volume-up-action" { 20 } else { 21 },
                match value {
                    "none" => 0,
                    "next" => 4,
                    "previous" => 5,
                    _ => return Err(t.set_bad_button.into()),
                },
            ),
            _ => return Err(t.set_unknown.replacen("{}", key, 1).replacen("{}", HELP, 1)),
        };
        Ok(Self {
            key: key.into(),
            value: value.into(),
            instruction,
            payload: vec![byte],
        })
    }
    pub fn packet(&self) -> Vec<u8> {
        protocol::request(self.instruction, &self.payload).expect("known setting payload fits")
    }
    pub fn expects_ack(&self) -> bool {
        self.instruction != 18
    }
    pub fn matches(&self, info: &DeviceInfo) -> bool {
        let idx = match self.key.as_str() {
            "anc" => 3,
            "eq" => 4,
            "dual" => 5,
            "game" => 6,
            "prompts" => 16,
            "spatial" => 20,
            "volume-up-action" => 22,
            "volume-down-action" => 23,
            "wind" => 25,
            _ => return false,
        };
        match info.raw.get(idx) {
            // Off/ambient preserve the device's previous ANC depth on some firmware.
            Some(actual) if self.key == "anc" && self.payload[0] & 15 != 1 => {
                actual & 15 == self.payload[0] & 15
            }
            Some(actual) => *actual == self.payload[0],
            None => false,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    pub raw: Vec<u8>,
}
impl DeviceInfo {
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
        let value = match key {
            "anc" => match *self.raw.get(3)? {
                b if b & 15 == 0 => "off",
                b if b & 15 == 2 => "ambient",
                b => ANC.iter().find(|(_, v)| *v == b)?.0,
            },
            "eq" => *EQ.get(usize::from(*self.raw.get(4)?))?,
            "dual" | "game" | "spatial" | "wind" => match *self.raw.get(match key {
                "dual" => 5,
                "game" => 6,
                "spatial" => 20,
                _ => 25,
            })? {
                0 => "off",
                1 => "on",
                _ => return None,
            },
            "prompts" => match *self.raw.get(16)? {
                0 => "voice",
                2 => "beeps",
                _ => return None,
            },
            "volume-up-action" | "volume-down-action" => match *self
                .raw
                .get(if key == "volume-up-action" { 22 } else { 23 })?
            {
                0 => "none",
                4 => "next",
                5 => "previous",
                _ => return None,
            },
            _ => return None,
        };
        Some(value.into())
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
        assert!(DeviceInfo::new(vec![0; 7]).is_err());
        let info = DeviceInfo::new(vec![255; 8]).unwrap();
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
        let info = DeviceInfo::new(raw).unwrap();
        for setting in parse_profile(&info.profile()).unwrap() {
            assert!(setting.matches(&info));
        }
    }
}
