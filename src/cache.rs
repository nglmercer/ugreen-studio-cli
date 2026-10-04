//! Remembers the last target so the next start can connect directly.
//! The file is a small key=value list, written atomically. A cache is
//! only a convenience: the model is still user-selected, never detected
//! from the device, and every connection still runs its preflight.
use crate::i18n::Lang;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Cache {
    pub address: Option<String>,
    pub model_confirmed: bool,
    pub channel: Option<u8>,
    pub lang: Option<Lang>,
    pub autoconnect: bool,
}
fn state_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("UGREEN_STATE_FILE") {
        if !p.is_empty() {
            return Some(PathBuf::from(p));
        }
    }
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("ugreen-cli").join("state"))
}
pub fn load() -> Cache {
    load_from(state_path().as_deref())
}
pub fn save(cache: &Cache) {
    if let Some(path) = state_path() {
        let _ = save_to(&path, cache);
    }
}
fn valid_address(address: &str) -> bool {
    let octets: Vec<&str> = address.split(':').collect();
    octets.len() == 6
        && address.len() == 17
        && octets
            .iter()
            .all(|o| o.len() == 2 && o.bytes().all(|b| b.is_ascii_hexdigit()))
}
pub fn load_from(path: Option<&Path>) -> Cache {
    let mut cache = Cache::default();
    let text = match path.and_then(|p| fs::read_to_string(p).ok()) {
        Some(text) => text,
        None => return cache,
    };
    for line in text.lines().take(64) {
        let (key, value) = match line.split_once('=') {
            Some(pair) => pair,
            None => continue,
        };
        let value = value.trim();
        match key.trim() {
            "address" if valid_address(value) => cache.address = Some(value.to_string()),
            "model" if value == "studio-pro" => cache.model_confirmed = true,
            "channel" => {
                if let Ok(channel) = value.parse::<u8>() {
                    if (1..=30).contains(&channel) {
                        cache.channel = Some(channel);
                    }
                }
            }
            "lang" => {
                if let Ok(lang) = Lang::from_code(value) {
                    cache.lang = Some(lang);
                }
            }
            "autoconnect" => cache.autoconnect = value == "1",
            _ => {}
        }
    }
    cache
}
pub fn save_to(path: &Path, cache: &Cache) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut text = String::new();
    if let Some(address) = &cache.address {
        if valid_address(address) {
            text.push_str("address=");
            text.push_str(address);
            text.push('\n');
        }
    }
    if cache.model_confirmed {
        text.push_str("model=studio-pro\n");
    }
    if let Some(channel) = cache.channel {
        text.push_str(&format!("channel={channel}\n"));
    }
    if let Some(lang) = cache.lang {
        text.push_str("lang=");
        text.push_str(lang.code());
        text.push('\n');
    }
    if cache.autoconnect {
        text.push_str("autoconnect=1\n");
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, text)?;
    fs::rename(&tmp, path)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn temp_path(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("ugreen-cache-{}-{name}", std::process::id()));
        let _ = fs::remove_file(&path);
        path
    }
    #[test]
    fn roundtrip_keeps_valid_values() {
        let path = temp_path("roundtrip");
        let cache = Cache {
            address: Some("AA:BB:CC:DD:EE:FF".into()),
            model_confirmed: true,
            channel: Some(7),
            lang: Some(Lang::Es),
            autoconnect: true,
        };
        save_to(&path, &cache).unwrap();
        assert_eq!(load_from(Some(&path)), cache);
        let _ = fs::remove_file(&path);
    }
    #[test]
    fn invalid_entries_are_ignored() {
        let path = temp_path("invalid");
        fs::write(
            &path,
            "address=not-an-address\nmodel=max5c\nchannel=99\nlang=fr\nautoconnect=yes\n",
        )
        .unwrap();
        assert_eq!(load_from(Some(&path)), Cache::default());
        let _ = fs::remove_file(&path);
    }
    #[test]
    fn missing_file_is_empty() {
        assert_eq!(load_from(None), Cache::default());
    }
}
