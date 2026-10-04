//! A validated, canonical Bluetooth device address.
//!
//! Device identity is the address, never the name: names are editable,
//! collide between vendors and change after firmware resets. All address
//! parsing in the project goes through this type so the strictness rules
//! live in exactly one place.

use std::fmt;
use std::io;
use std::str::FromStr;

const INVALID: &str =
    "Bluetooth address must contain exactly six colon-separated hex octets, e.g. AA:BB:CC:DD:EE:FF";

/// Six colon-separated hexadecimal octets, stored in canonical uppercase
/// form. Accepts lower/mixed case input; equality, ordering and hashing
/// therefore behave consistently for the same physical device.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct BluetoothAddress {
    text: String,
    octets: [u8; 6],
}

impl BluetoothAddress {
    /// Parse `XX:XX:XX:XX:XX:XX` exactly: 17 ASCII bytes, two hex digits
    /// per octet, colon separators. Rejects shorter/longer forms, wrong
    /// separators, whitespace and any non-ASCII input.
    pub fn parse(value: &str) -> io::Result<Self> {
        let octets = parse_octets(value)?;
        Ok(Self {
            text: format_octets(&octets),
            octets,
        })
    }
    /// Build the canonical display form from raw octets.
    pub fn from_octets(octets: [u8; 6]) -> Self {
        Self {
            text: format_octets(&octets),
            octets,
        }
    }
    /// The six octets in display order (first octet printed first).
    pub fn octets(&self) -> [u8; 6] {
        self.octets
    }
    /// Canonical uppercase `XX:XX:XX:XX:XX:FF` text; never empty.
    pub fn as_str(&self) -> &str {
        &self.text
    }
}

impl fmt::Display for BluetoothAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

impl FromStr for BluetoothAddress {
    type Err = io::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

fn parse_octets(value: &str) -> io::Result<[u8; 6]> {
    let invalid = || io::Error::new(io::ErrorKind::InvalidInput, INVALID);
    let bytes = value.as_bytes();
    if bytes.len() != 17 {
        return Err(invalid());
    }
    let mut result = [0u8; 6];
    for (index, octet) in result.iter_mut().enumerate() {
        let offset = index * 3;
        if index != 5 && bytes[offset + 2] != b':' {
            return Err(invalid());
        }
        let high = hex_digit(bytes[offset]).ok_or_else(invalid)?;
        let low = hex_digit(bytes[offset + 1]).ok_or_else(invalid)?;
        *octet = high * 16 + low;
    }
    Ok(result)
}

fn hex_digit(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn format_octets(octets: &[u8; 6]) -> String {
    format!(
        "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        octets[0], octets[1], octets[2], octets[3], octets[4], octets[5]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_mixed_case_and_canonicalizes() {
        let address = BluetoothAddress::parse("01:23:aB:cD:eF:45").unwrap();
        assert_eq!(address.as_str(), "01:23:AB:CD:EF:45");
        assert_eq!(address.octets(), [0x01, 0x23, 0xab, 0xcd, 0xef, 0x45]);
        assert_eq!(
            address,
            BluetoothAddress::parse("01:23:AB:CD:EF:45").unwrap()
        );
        assert_eq!(address.to_string(), "01:23:AB:CD:EF:45");
    }

    #[test]
    fn octet_roundtrip_is_lossless() {
        for octets in [[0u8; 6], [0xff; 6], [0x01, 0x23, 0x45, 0x67, 0x89, 0xab]] {
            let address = BluetoothAddress::from_octets(octets);
            assert_eq!(address.octets(), octets);
            assert_eq!(BluetoothAddress::parse(address.as_str()).unwrap(), address);
        }
    }

    #[test]
    fn rejects_invalid_forms() {
        for invalid in [
            "",
            "1:23:AB:CD:EF:45",
            "01:23:AB:CD:EF:4",
            "01-23-AB-CD-EF-45",
            "01.23.AB.CD.EF.45",
            "01:23:AB:CD:EF:4G",
            "01:23:AB:CD:EF:45:67",
            " 01:23:AB:CD:EF:45",
            "01:23:AB:CD:EF:45\n",
            "01:23:AB:CD:EF:45 ",
            "é1:23:AB:CD:EF:4",
            "Ａ1:23:AB:CD:EF:4",
        ] {
            let error = BluetoothAddress::parse(invalid).unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::InvalidInput, "{invalid:?}");
        }
    }

    #[test]
    fn unicode_rejection_is_not_byte_length_luck() {
        // Two-byte é keeps the total at 17 bytes; byte-wise parsing must
        // still reject the non-hex leading bytes.
        let tricky = "\u{e9}1:23:AB:CD:EF:4";
        assert_eq!(tricky.len(), 17);
        assert!(BluetoothAddress::parse(tricky).is_err());
    }

    #[test]
    fn ordering_and_hashing_follow_the_address() {
        let a = BluetoothAddress::parse("AA:AA:AA:AA:AA:01").unwrap();
        let b = BluetoothAddress::parse("AA:AA:AA:AA:AA:02").unwrap();
        assert!(a < b);
        let mut set = std::collections::HashSet::new();
        set.insert(a.clone());
        set.insert(BluetoothAddress::parse("aa:aa:aa:aa:aa:01").unwrap());
        assert_eq!(set.len(), 1);
        set.insert(b);
        assert_eq!(set.len(), 2);
    }
}
