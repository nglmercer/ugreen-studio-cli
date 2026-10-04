//! Persistent device registry: the last selected headset, its confirmed
//! model, and the connection preferences.
//!
//! The file is only a convenience: the model is still user-selected,
//! never detected, and every connection runs its own preflight. Both the
//! historical flat (v1) list and the sectioned v2 format parse through
//! this one code path, and unknown or malformed lines are skipped so a
//! single bad line never loses the rest of the file.

use crate::{bluetooth::BluetoothAddress, i18n::Lang, models::Model};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// Files larger than this are treated as corrupt. A convenience store
/// never grows past this bound, so something else is writing here.
const MAX_FILE_BYTES: usize = 64 * 1024;
/// Rendered files stay within the same bound; device entries are
/// dropped, in stable address order from the end, until they fit.
const MAX_WRITE_BYTES: usize = MAX_FILE_BYTES;
/// Names come from the OS and are display-only; bound them so a hostile
/// name cannot bloat the file or the parser.
const MAX_NAME_CHARS: usize = 96;

/// One remembered headset. Keyed by `address`; the name is cosmetic
/// and never creates identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviceEntry {
    pub address: BluetoothAddress,
    pub name: Option<String>,
    /// The model the user explicitly confirmed for this address.
    pub model: Option<Model>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DeviceRegistry {
    pub selected: Option<BluetoothAddress>,
    /// Sorted by address, at most one entry per address.
    pub devices: Vec<DeviceEntry>,
    pub channel: Option<u8>,
    pub timeout: Option<u64>,
    pub lang: Option<Lang>,
    pub autoconnect: bool,
}

impl DeviceRegistry {
    pub fn entry(&self, address: &BluetoothAddress) -> Option<&DeviceEntry> {
        self.devices
            .binary_search_by(|entry| entry.address.cmp(address))
            .ok()
            .map(|index| &self.devices[index])
    }
    fn entry_mut(&mut self, address: &BluetoothAddress) -> Option<&mut DeviceEntry> {
        self.devices
            .binary_search_by(|entry| entry.address.cmp(address))
            .ok()
            .map(|index| &mut self.devices[index])
    }
    /// Find or create the entry for `address`, keeping the list sorted.
    fn upsert(&mut self, address: BluetoothAddress) -> &mut DeviceEntry {
        match self
            .devices
            .binary_search_by(|entry| entry.address.cmp(&address))
        {
            Ok(index) => &mut self.devices[index],
            Err(index) => {
                self.devices.insert(
                    index,
                    DeviceEntry {
                        address,
                        name: None,
                        model: None,
                    },
                );
                &mut self.devices[index]
            }
        }
    }
    pub fn set_selected(&mut self, address: &BluetoothAddress) {
        self.upsert(address.clone());
        self.selected = Some(address.clone());
    }
    pub fn set_model(&mut self, address: &BluetoothAddress, model: Option<Model>) {
        if let Some(entry) = self.entry_mut(address) {
            entry.model = model;
        }
    }
    pub fn model_of(&self, address: &BluetoothAddress) -> Option<Model> {
        self.entry(address).and_then(|entry| entry.model)
    }
    pub fn selected_model(&self) -> Option<Model> {
        self.selected
            .as_ref()
            .and_then(|address| self.model_of(address))
    }

    pub fn save(&self) {
        if let Some(path) = state_path() {
            let _ = self.save_to(&path);
        }
    }
    /// Write v2 atomically: a temporary file next to the target, then
    /// rename over it, so readers never observe a half-written file.
    pub fn save_to(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, self.render())?;
        fs::rename(&tmp, path)
    }

    fn render(&self) -> String {
        let mut text = String::from("version=2\n");
        if let Some(selected) = self.selected.as_ref() {
            text.push_str("selected=");
            text.push_str(selected.as_str());
            text.push('\n');
        }
        if self.channel.is_some()
            || self.timeout.is_some()
            || self.lang.is_some()
            || self.autoconnect
        {
            text.push_str("[settings]\n");
            if let Some(channel) = self.channel {
                text.push_str(&format!("channel={channel}\n"));
            }
            if let Some(timeout) = self.timeout {
                text.push_str(&format!("timeout={timeout}\n"));
            }
            if let Some(lang) = self.lang {
                text.push_str("lang=");
                text.push_str(lang.code());
                text.push('\n');
            }
            if self.autoconnect {
                text.push_str("autoconnect=1\n");
            }
        }
        // The selected entry renders first, so size-cap trimming from
        // the end can never drop the target the user chose.
        let mut blocks: Vec<String> = Vec::with_capacity(self.devices.len());
        if let Some(selected) = self.selected.as_ref() {
            if let Some(entry) = self.entry(selected) {
                blocks.push(render_entry(entry));
            }
        }
        for entry in &self.devices {
            if Some(&entry.address) != self.selected.as_ref() {
                blocks.push(render_entry(entry));
            }
        }
        let mut remaining: usize = blocks.iter().map(String::len).sum();
        while !blocks.is_empty() && text.len() + remaining > MAX_WRITE_BYTES {
            if let Some(block) = blocks.pop() {
                remaining -= block.len();
            }
        }
        for block in blocks {
            text.push_str(&block);
        }
        text
    }

    pub fn load() -> Self {
        Self::load_from(state_path().as_deref())
    }
    /// Parse either on-disk format. Missing, oversized, oversized-version
    /// or unreadable files all yield the default registry: the file is
    /// never required for correct operation.
    pub fn load_from(path: Option<&Path>) -> Self {
        let text = match path.and_then(|path| fs::read_to_string(path).ok()) {
            Some(text) if text.len() <= MAX_FILE_BYTES => text,
            Some(_) => return Self::default(),
            None => return Self::default(),
        };
        let mut registry = Self::default();
        let mut section = Section::Header;
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                section = parse_section(&line[1..line.len() - 1]);
                // An empty device section still remembers the address.
                if let Section::Device(address) = &section {
                    registry.upsert(address.clone());
                }
                continue;
            }
            // Salvage: a line without a pair, or with unknown keys, is
            // skipped without discarding the rest of the file.
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let (key, value) = (key.trim(), value.trim());
            match key {
                "version" => {
                    if value != "2" {
                        // A future format must not be half-parsed.
                        return Self::default();
                    }
                }
                // v1 wrote `address=`; v2 writes `selected=`. Both name
                // the target and imply a device entry.
                "address" | "selected" => {
                    if let Ok(address) = BluetoothAddress::parse(value) {
                        registry.set_selected(&address);
                    }
                }
                "model" => {
                    // The v1 flat form keeps the historical rule: only
                    // verified identifiers count, anything else is noise.
                    if let Some(model) = Model::parse(value) {
                        if let Some(target) = section_address(&section, &registry) {
                            registry.upsert(target).model = Some(model);
                        }
                    }
                }
                "name" => {
                    if let Some(target) = section_address(&section, &registry) {
                        registry.upsert(target).name = Some(value.to_owned());
                    }
                }
                "channel" => {
                    if let Ok(channel) = value.parse::<u8>() {
                        if (1..=30).contains(&channel) {
                            registry.channel = Some(channel);
                        }
                    }
                }
                "timeout" => {
                    if let Ok(timeout) = value.parse::<u64>() {
                        if (1..=60).contains(&timeout) {
                            registry.timeout = Some(timeout);
                        }
                    }
                }
                "lang" => {
                    if let Ok(lang) = Lang::from_code(value) {
                        registry.lang = Some(lang);
                    }
                }
                "autoconnect" => registry.autoconnect = value == "1",
                _ => {}
            }
        }
        registry
    }
}

/// Where a device-scoped key (`model`, `name`) applies. The flat v1
/// format has no sections, so such keys follow the selected address.
enum Section {
    Header,
    Settings,
    Device(BluetoothAddress),
    Ignored,
}

fn section_address(section: &Section, registry: &DeviceRegistry) -> Option<BluetoothAddress> {
    match section {
        Section::Device(address) => Some(address.clone()),
        Section::Header => registry.selected.clone(),
        Section::Settings | Section::Ignored => None,
    }
}

fn parse_section(body: &str) -> Section {
    let body = body.trim();
    if body == "settings" {
        return Section::Settings;
    }
    if let Some(rest) = body.strip_prefix("device") {
        let rest = rest.trim();
        if rest.len() >= 2 && rest.starts_with('"') && rest.ends_with('"') {
            if let Ok(address) = BluetoothAddress::parse(&rest[1..rest.len() - 1]) {
                return Section::Device(address);
            }
        }
        // A malformed device header keeps its keys out of everything else.
        return Section::Ignored;
    }
    Section::Ignored
}

fn render_entry(entry: &DeviceEntry) -> String {
    let mut block = String::new();
    block.push_str("[device \"");
    block.push_str(entry.address.as_str());
    block.push_str("\"]\n");
    if let Some(name) = entry.name.as_deref().map(sanitize_name) {
        if !name.is_empty() {
            block.push_str("name=");
            block.push_str(&name);
            block.push('\n');
        }
    }
    if let Some(model) = entry.model {
        block.push_str("model=");
        block.push_str(model.as_str());
        block.push('\n');
    }
    block
}

/// Names are untrusted display text: control characters (including
/// newlines, which would forge file structure) are dropped and the
/// result is bounded.
fn sanitize_name(name: &str) -> String {
    name.chars()
        .filter(|ch| !ch.is_control())
        .take(MAX_NAME_CHARS)
        .collect::<String>()
        .trim()
        .to_owned()
}

fn state_path() -> Option<PathBuf> {
    // Unit tests must never read or touch the user's real state file;
    // they exercise the format through explicit paths instead.
    if cfg!(test) {
        return None;
    }
    if let Ok(path) = std::env::var("UGREEN_STATE_FILE") {
        if !path.is_empty() {
            return Some(PathBuf::from(path));
        }
    }
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("ugreen-cli").join("state"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("ugreen-registry-{}-{name}", std::process::id()));
        let _ = fs::remove_file(&path);
        path
    }

    #[test]
    fn v2_roundtrip_keeps_valid_values() {
        let path = temp_path("roundtrip");
        let mut registry = DeviceRegistry::default();
        let address = BluetoothAddress::parse("AA:BB:CC:DD:EE:FF").unwrap();
        registry.set_selected(&address);
        registry.set_model(&address, Some(Model::StudioPro));
        registry.upsert(address).name = Some("UGREEN Studio Pro".into());
        let other = BluetoothAddress::parse("11:22:33:44:55:66").unwrap();
        registry.upsert(other).name = Some("Other Buds".into());
        registry.channel = Some(7);
        registry.timeout = Some(5);
        registry.lang = Some(Lang::Es);
        registry.autoconnect = true;
        registry.save_to(&path).unwrap();
        assert_eq!(DeviceRegistry::load_from(Some(&path)), registry);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn v1_files_parse_through_the_same_path() {
        let path = temp_path("v1");
        fs::write(
            &path,
            "address=AA:BB:CC:DD:EE:FF\nmodel=studio-pro\nchannel=3\ntimeout=4\nlang=en\nautoconnect=1\n",
        )
        .unwrap();
        let registry = DeviceRegistry::load_from(Some(&path));
        assert_eq!(
            registry.selected,
            Some(BluetoothAddress::parse("AA:BB:CC:DD:EE:FF").unwrap())
        );
        assert_eq!(registry.selected_model(), Some(Model::StudioPro));
        assert_eq!(registry.channel, Some(3));
        assert_eq!(registry.timeout, Some(4));
        assert_eq!(registry.lang, Some(Lang::En));
        assert!(registry.autoconnect);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn invalid_entries_are_ignored() {
        let path = temp_path("invalid");
        fs::write(
            &path,
            "address=not-an-address\nmodel=max5c\nchannel=99\ntimeout=99\nlang=xx\nautoconnect=yes\n",
        )
        .unwrap();
        assert_eq!(
            DeviceRegistry::load_from(Some(&path)),
            DeviceRegistry::default()
        );
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn missing_file_is_empty() {
        assert_eq!(DeviceRegistry::load_from(None), DeviceRegistry::default());
    }

    #[test]
    fn malformed_lines_salvage_the_rest() {
        let path = temp_path("salvage");
        fs::write(
            &path,
            concat!(
                "version=2\n",
                "garbage without pair\n",
                "selected=AA:BB:CC:DD:EE:FF\n",
                "[device \"broken\"]\n",
                "model=studio-pro\n",
                "[device \"AA:BB:CC:DD:EE:FF\"]\n",
                "name=Studio Pro\n",
                "channel=2\n",
            ),
        )
        .unwrap();
        let registry = DeviceRegistry::load_from(Some(&path));
        let selected = BluetoothAddress::parse("AA:BB:CC:DD:EE:FF").unwrap();
        assert_eq!(registry.selected, Some(selected.clone()));
        assert_eq!(registry.model_of(&selected), None);
        assert_eq!(
            registry
                .entry(&selected)
                .and_then(|entry| entry.name.as_deref()),
            Some("Studio Pro")
        );
        assert_eq!(registry.channel, Some(2));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn unknown_versions_and_oversized_files_are_not_half_parsed() {
        let path = temp_path("future");
        fs::write(&path, "version=3\nselected=AA:BB:CC:DD:EE:FF\nchannel=9\n").unwrap();
        assert_eq!(
            DeviceRegistry::load_from(Some(&path)),
            DeviceRegistry::default()
        );
        let path = temp_path("oversized");
        fs::write(&path, format!("channel=1\n{}", "x".repeat(MAX_FILE_BYTES))).unwrap();
        assert_eq!(
            DeviceRegistry::load_from(Some(&path)),
            DeviceRegistry::default()
        );
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn duplicate_sections_merge_by_address() {
        let path = temp_path("duplicate");
        fs::write(
            &path,
            "[device \"AA:BB:CC:DD:EE:FF\"]\nname=First\n[device \"AA:BB:CC:DD:EE:FF\"]\nmodel=studio-pro\n",
        )
        .unwrap();
        let registry = DeviceRegistry::load_from(Some(&path));
        assert_eq!(registry.devices.len(), 1);
        let address = BluetoothAddress::parse("AA:BB:CC:DD:EE:FF").unwrap();
        assert_eq!(registry.model_of(&address), Some(Model::StudioPro));
        assert_eq!(
            registry
                .entry(&address)
                .and_then(|entry| entry.name.as_deref()),
            Some("First")
        );
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn written_files_stay_within_the_size_bound() {
        let path = temp_path("capped");
        let mut registry = DeviceRegistry::default();
        let selected = BluetoothAddress::parse("AA:BB:CC:DD:EE:FF").unwrap();
        registry.selected = Some(selected.clone());
        let mut devices = vec![DeviceEntry {
            address: selected.clone(),
            name: Some("UGREEN Studio Pro".into()),
            model: Some(Model::StudioPro),
        }];
        // Enough entries, with long names, to blow past 64 KiB. All of
        // them sort before the selected address, so naive trimming from
        // the end would reach it.
        for index in 0..600u16 {
            devices.push(DeviceEntry {
                address: BluetoothAddress::from_octets([
                    0,
                    0,
                    0,
                    0,
                    (index >> 8) as u8,
                    index as u8,
                ]),
                name: Some("N".repeat(96)),
                model: None,
            });
        }
        devices.sort_by(|a, b| a.address.cmp(&b.address));
        registry.devices = devices;
        registry.save_to(&path).unwrap();
        let bytes = fs::read(&path).unwrap();
        assert!(bytes.len() <= MAX_WRITE_BYTES, "{} bytes", bytes.len());
        let loaded = DeviceRegistry::load_from(Some(&path));
        assert_eq!(loaded.selected, Some(selected.clone()));
        assert_eq!(loaded.model_of(&selected), Some(Model::StudioPro));
        assert!(loaded.devices.len() < 601, "{}", loaded.devices.len());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn hostile_names_cannot_forge_file_structure() {
        let path = temp_path("hostile");
        let mut registry = DeviceRegistry::default();
        let address = BluetoothAddress::parse("AA:BB:CC:DD:EE:FF").unwrap();
        registry.set_selected(&address);
        registry.upsert(address.clone()).name =
            Some("Evil\n[device \"11:22:33:44:55:66\"]\nmodel=studio-pro".into());
        registry.save_to(&path).unwrap();
        let loaded = DeviceRegistry::load_from(Some(&path));
        // Newlines are dropped, so the injection lands inside one value.
        assert_eq!(loaded.devices.len(), 1);
        assert_eq!(
            loaded
                .entry(&address)
                .and_then(|entry| entry.name.as_deref()),
            Some("Evil[device \"11:22:33:44:55:66\"]model=studio-pro")
        );
        assert_eq!(
            loaded.model_of(&BluetoothAddress::parse("11:22:33:44:55:66").unwrap()),
            None
        );
        let _ = fs::remove_file(&path);
    }
}
