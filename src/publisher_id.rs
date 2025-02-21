use core::cmp::Ordering;
use core::fmt;
use core::fmt::{Debug, Display, Formatter};
use core::hash::{Hash, Hasher};
use core::str::FromStr;
use heapless::String;
use thiserror::Error;

const PUBLISHER_ID_LENGTH: usize = 13;

/// A Crockford Base32 encoded 13-character long Publisher Id derived from a Publisher
///
/// [Publisher Id](https://learn.microsoft.com/windows/apps/desktop/modernize/package-identity-overview#publisher-id)
#[derive(Clone, Debug, Eq)]
pub struct PublisherId(pub(crate) String<PUBLISHER_ID_LENGTH>);

impl PublisherId {
    const CROCKFORD_OMITTED_CHARACTERS: [char; 4] = ['i', 'l', 'o', 'u'];
}

#[derive(Error, Debug, Eq, PartialEq)]
pub enum PublisherIdError {
    #[error("Expected Publisher Id length of {PUBLISHER_ID_LENGTH}")]
    InvalidLength,
    #[error("Expected Crockford Base-32 string (A-Z0-9 except no I, L, O, or U)")]
    InvalidCharacters,
}

impl FromStr for PublisherId {
    type Err = PublisherIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.chars().any(|char| {
            !char.is_ascii_alphanumeric()
                || Self::CROCKFORD_OMITTED_CHARACTERS.contains(&char.to_ascii_lowercase())
        }) {
            return Err(PublisherIdError::InvalidCharacters);
        } else if s.len() != PUBLISHER_ID_LENGTH {
            // We can check byte length because at this point we know the string is ASCII
            return Err(PublisherIdError::InvalidLength);
        }

        Ok(Self(
            s.parse().map_err(|()| PublisherIdError::InvalidLength)?,
        ))
    }
}

impl Default for PublisherId {
    fn default() -> Self {
        // This isn't an ideal default but ensures that it will still have a fixed length of 13.
        Self(core::iter::repeat_n('0', PUBLISHER_ID_LENGTH).collect::<_>())
    }
}

impl PartialEq for PublisherId {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

impl PartialOrd for PublisherId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PublisherId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .as_bytes()
            .iter()
            .map(u8::to_ascii_lowercase)
            .cmp(other.0.as_bytes().iter().map(u8::to_ascii_lowercase))
    }
}

impl Hash for PublisherId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for byte in self.0.as_bytes() {
            state.write_u8(byte.to_ascii_lowercase());
        }
    }
}

impl Display for PublisherId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::publisher_id::{PublisherId, PublisherIdError};
    use core::cmp::Ordering;
    use core::hash::{BuildHasher, Hash, Hasher};
    use foldhash::fast::RandomState;

    #[test]
    fn publisher_id_from_str() {
        assert!("zj75k085cmj1a".parse::<PublisherId>().is_ok());
    }

    #[test]
    fn publisher_id_too_short() {
        assert_eq!(
            "1".repeat(3).parse::<PublisherId>().err().unwrap(),
            PublisherIdError::InvalidLength
        );
    }

    #[test]
    fn publisher_id_too_long() {
        assert_eq!(
            "1".repeat(20).parse::<PublisherId>().err().unwrap(),
            PublisherIdError::InvalidLength
        );
    }

    #[test]
    fn invalid_publisher_id_characters() {
        assert_eq!(
            "zI75KO85cmL1U".parse::<PublisherId>().err().unwrap(),
            PublisherIdError::InvalidCharacters
        )
    }

    #[test]
    fn default_publisher_id() {
        assert_eq!(
            PublisherId::default(),
            "0000000000000".parse::<PublisherId>().unwrap()
        );
    }

    #[test]
    fn publisher_id_equality() {
        let lower_id = "zj75k085cmj1a".parse::<PublisherId>().unwrap();
        let upper_id = "ZJ75K085CMJ1A".parse::<PublisherId>().unwrap();

        assert_eq!(lower_id, lower_id);
        assert_eq!(lower_id, upper_id);
        assert_ne!(lower_id, "yjp7t9tn9g0z0".parse::<PublisherId>().unwrap());
    }

    #[test]
    fn publisher_id_comparison() {
        let lower_id = "zj75k085cmj1a".parse::<PublisherId>().unwrap();
        let upper_id = "ZJ75K085CMJ1A".parse::<PublisherId>().unwrap();

        assert_eq!(lower_id.cmp(&lower_id), Ordering::Equal);
        assert_eq!(lower_id.cmp(&upper_id), Ordering::Equal);

        let other_id = "yjp7t9tn9g0z0".parse::<PublisherId>().unwrap();
        assert_eq!(lower_id.cmp(&other_id), Ordering::Greater);
        assert_eq!(other_id.cmp(&lower_id), Ordering::Less);
    }

    #[test]
    fn publisher_id_hash() {
        // If two keys are equal, their hashes must also be equal
        // https://doc.rust-lang.org/std/hash/trait.Hash.html#hash-and-eq

        let lower_id = "zj75k085cmj1a".parse::<PublisherId>().unwrap();
        let upper_id = "ZJ75K085CMJ1A".parse::<PublisherId>().unwrap();
        assert_eq!(lower_id, upper_id);

        let state = RandomState::default();
        let mut id_1_hasher = state.build_hasher();
        lower_id.hash(&mut id_1_hasher);

        let mut id_2_hasher = state.build_hasher();
        upper_id.hash(&mut id_2_hasher);

        assert_eq!(id_1_hasher.finish(), id_2_hasher.finish());
    }
}
