//! Project prefixes and issue IDs.

use std::fmt;

use crate::error::{Error, ErrorCode, Result};

/// The lowercase Crockford base32 alphabet that ID characters are drawn from.
pub const ALPHABET: &[u8; 32] = b"0123456789abcdefghjkmnpqrstvwxyz";

/// The number of random characters after `<prefix>-`.
pub const RANDOM_LEN: usize = 6;

/// The longest allowed prefix.
const PREFIX_MAX_LEN: usize = 8;

/// A project's ID prefix: 1-8 lowercase ASCII letters or digits, starting with
/// a letter.
// @mara implements DES-PROJECT-LAYOUT
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Prefix(String);

impl Prefix {
    pub fn parse(value: &str) -> Result<Self> {
        let valid = value.len() <= PREFIX_MAX_LEN
            && value.starts_with(|c: char| c.is_ascii_lowercase())
            && value
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit());
        if valid {
            Ok(Self(value.to_owned()))
        } else {
            Err(Error::invalid_value(format!(
                "invalid prefix {value:?}: use 1-{PREFIX_MAX_LEN} lowercase ASCII letters or digits, starting with a letter"
            )))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for Prefix {
    fn default() -> Self {
        Self("zb".to_owned())
    }
}

impl fmt::Display for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// An issue ID: `<prefix>-` followed by six characters from [`ALPHABET`].
// @mara implements DES-ISSUE-ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id(String);

impl Id {
    /// Parses an ID exactly as it must appear in a filename.
    pub fn parse(prefix: &Prefix, value: &str) -> Option<Self> {
        let random = value.strip_prefix(prefix.as_str())?.strip_prefix('-')?;
        let valid = random.len() == RANDOM_LEN && random.bytes().all(|b| ALPHABET.contains(&b));
        valid.then(|| Self(value.to_owned()))
    }

    /// Parses an ID given as input, matching it case-insensitively.
    pub fn parse_input(prefix: &Prefix, value: &str) -> Option<Self> {
        Self::parse(prefix, &value.to_ascii_lowercase())
    }

    /// Builds the ID whose random part encodes the low 30 bits of `bits`.
    pub fn from_bits(prefix: &Prefix, bits: u32) -> Self {
        let mut id = String::with_capacity(prefix.as_str().len() + 1 + RANDOM_LEN);
        id.push_str(prefix.as_str());
        id.push('-');
        for i in (0..RANDOM_LEN).rev() {
            id.push(ALPHABET[(bits >> (5 * i)) as usize & 31] as char);
        }
        Self(id)
    }

    /// Draws a fresh ID from operating-system randomness.
    pub fn generate(prefix: &Prefix) -> Result<Self> {
        let bits = getrandom::u32().map_err(|error| {
            Error::new(
                ErrorCode::Io,
                format!("cannot read operating-system randomness: {error}"),
            )
        })?;
        Ok(Self::from_bits(prefix, bits))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for Id {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zb() -> Prefix {
        Prefix::default()
    }

    #[test]
    fn prefix_rule() {
        for valid in ["zb", "a", "web", "a1b2c3d4", "x9"] {
            assert!(Prefix::parse(valid).is_ok(), "{valid}");
        }
        for invalid in [
            "",
            "Zb",
            "1zb",
            "-zb",
            "zb-",
            "toolongpx",
            "z_b",
            "zé",
            "z b",
        ] {
            let error = Prefix::parse(invalid).unwrap_err();
            assert_eq!(error.code, ErrorCode::InvalidValue, "{invalid}");
        }
    }

    #[test]
    fn id_rule() {
        assert!(Id::parse(&zb(), "zb-n938cw").is_some());
        for invalid in [
            "zb-n938c",
            "zb-n938cww",
            "zb-N938CW",
            "zb-n938cu",
            "zb_n938cw",
            "web-n938cw",
            "zbn938cw",
            "zb-i938cw",
            "zb-l938cw",
            "zb-o938cw",
        ] {
            assert!(Id::parse(&zb(), invalid).is_none(), "{invalid}");
        }
    }

    #[test]
    fn input_ids_match_case_insensitively() {
        assert_eq!(
            Id::parse_input(&zb(), "ZB-N938CW").unwrap().as_str(),
            "zb-n938cw"
        );
    }

    #[test]
    fn bits_map_onto_the_alphabet() {
        assert_eq!(Id::from_bits(&zb(), 0).as_str(), "zb-000000");
        assert_eq!(Id::from_bits(&zb(), (1 << 30) - 1).as_str(), "zb-zzzzzz");
        assert_eq!(Id::from_bits(&zb(), u32::MAX).as_str(), "zb-zzzzzz");
        assert_eq!(Id::from_bits(&zb(), 31 << 25 | 1).as_str(), "zb-z00001");
    }
}
