use core::{
    char,
    cmp::Ordering,
    fmt,
    fmt::{Debug, Display, Formatter},
    hash::{Hash, Hasher},
    str::FromStr,
};

use fast32::base32::CROCKFORD_LOWER;
use heapless::String;
use sha2::{Digest, Sha256};
use thiserror::Error;

const CROCKFORD_OMITTED_CHARACTERS: [char; 4] = ['i', 'l', 'o', 'u'];

/// A Crockford Base32 encoded 13-character long [Publisher Id] derived from a Publisher.
///
/// [Publisher Id]: https://learn.microsoft.com/windows/apps/desktop/modernize/package-identity-overview#publisher-id
#[derive(Clone, Debug, Eq)]
pub struct PublisherId(String<{ PublisherId::LENGTH }>);

impl PublisherId {
    /// The constant length of a Publisher Id.
    ///
    /// # Examples
    ///
    /// ```
    /// # use package_family_name::PublisherId;
    /// assert_eq!(PublisherId::LENGTH, 13);
    /// ```
    pub const LENGTH: usize = 13;

    #[must_use]
    pub fn new<S>(identity_publisher: S) -> Self
    where
        S: AsRef<str>,
    {
        const HASH_TRUNCATION_LENGTH: usize = 8;

        let publisher_sha_256 = identity_publisher
            .as_ref()
            .encode_utf16()
            .map(u16::to_le_bytes)
            .fold(Sha256::new(), Sha256::chain_update)
            .finalize();

        let truncated_hash = &publisher_sha_256[..HASH_TRUNCATION_LENGTH];
        let crockford_encoded = CROCKFORD_LOWER.encode(truncated_hash);

        crockford_encoded
            .parse()
            .unwrap_or_else(|_| unreachable!("An 8-byte array encoded with Crockford Base32 should always have an expected length of 13"))
    }

    /// Extracts a string slice containing the entire Publisher Id.
    ///
    /// # Examples
    ///
    /// ```
    /// # use package_family_name::PublisherId;
    /// let publisher_id = PublisherId::new("CN=Microsoft Corporation, O=Microsoft Corporation, L=Redmond, S=Washington, C=US");
    ///
    /// assert_eq!(publisher_id.as_str(), "8wekyb3d8bbwe");
    /// ```
    #[must_use]
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Returns the length of `self`.
    ///
    /// This will always be equal to 13.
    ///
    /// This length is in bytes, not [`prim@char`]s or graphemes. In other words, it might not be
    /// what a human considers the length of the string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use package_family_name::PublisherId;
    /// let publisher_id = PublisherId::new("CN=Microsoft Corporation, O=Microsoft Corporation, L=Redmond, S=Washington, C=US");
    ///
    /// assert_eq!(publisher_id.len(), 13);
    /// ```
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if `self` has a length of zero bytes.
    ///
    /// This will always be false.
    ///
    /// # Examples
    ///
    /// ```
    /// # use package_family_name::PublisherId;
    /// let publisher_id = PublisherId::new("CN=Microsoft Corporation, O=Microsoft Corporation, L=Redmond, S=Washington, C=US");
    ///
    /// assert_eq!(publisher_id.is_empty(), false);
    /// ```
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl AsRef<str> for PublisherId {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Default for PublisherId {
    fn default() -> Self {
        // This isn't an ideal default but ensures that it will still have a fixed length of 13
        Self(core::iter::repeat_n('0', Self::LENGTH).collect::<_>())
    }
}

impl Display for PublisherId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.as_str(), f)
    }
}

impl PartialEq for PublisherId {
    /// Tests for `self` and `other` values to be equal, and is used by `==`.
    ///
    /// Publisher Id is compared case-insensitively.
    ///
    /// # Examples
    ///
    /// ```
    /// # use package_family_name::{PublisherId, PublisherIdError};
    /// # fn main() -> Result<(), PublisherIdError> {
    /// let publisher_id_1 = "8wekyb3d8bbwe".parse::<PublisherId>()?;
    /// let publisher_id_2 = "8WEKYB3D8BBWE".parse::<PublisherId>()?;
    ///
    /// assert_eq!(publisher_id_1, publisher_id_2);
    /// # Ok(())
    /// # }
    /// ```
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

#[derive(Error, Debug, Eq, PartialEq)]
pub enum PublisherIdError {
    /// The length of the publisher Id is not 13.
    #[error("Publisher Id length was not {}", PublisherId::LENGTH)]
    InvalidLength,

    /// The Publisher Id contains characters disallowed in a Publisher Id.
    #[error("Expected Crockford Base-32 string (A-Z0-9 except no I, L, O, or U)")]
    InvalidCharacters,
}

impl FromStr for PublisherId {
    type Err = PublisherIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.chars().any(|char| {
            !char.is_ascii_alphanumeric()
                || CROCKFORD_OMITTED_CHARACTERS.contains(&char.to_ascii_lowercase())
        }) {
            return Err(PublisherIdError::InvalidCharacters);
        }

        if s.len() != Self::LENGTH {
            // We can check byte length because at this point we know the string is ASCII
            return Err(PublisherIdError::InvalidLength);
        }

        Ok(Self(
            s.parse().map_err(|_| PublisherIdError::InvalidLength)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use core::{
        cmp::Ordering,
        hash::{BuildHasher, Hash, Hasher},
    };

    use super::{PublisherId, PublisherIdError};

    #[test]
    fn from_identity_publisher() {
        let publisher_id = PublisherId::new("Publisher Software");
        assert_eq!(publisher_id.as_str(), "zj75k085cmj1a");
    }

    #[test]
    fn from_different_identity_publishers() {
        assert_ne!(
            PublisherId::new("Publisher Software"),
            PublisherId::new("Another Publisher")
        );
    }

    #[test]
    fn from_str() {
        assert!("zj75k085cmj1a".parse::<PublisherId>().is_ok());
    }

    #[test]
    fn too_short() {
        assert_eq!(
            "1".repeat(3).parse::<PublisherId>().err(),
            Some(PublisherIdError::InvalidLength)
        );
    }

    #[test]
    fn too_long() {
        assert_eq!(
            "1".repeat(20).parse::<PublisherId>().err(),
            Some(PublisherIdError::InvalidLength)
        );
    }

    #[test]
    fn invalid_characters() {
        assert_eq!(
            "zI75KO85cmL1U".parse::<PublisherId>().err(),
            Some(PublisherIdError::InvalidCharacters)
        );

        assert_eq!(
            r#"z?75%O/5\mL"U"#.parse::<PublisherId>().err(),
            Some(PublisherIdError::InvalidCharacters)
        );
    }

    #[test]
    fn default() {
        assert_eq!(
            PublisherId::default(),
            "0000000000000".parse::<PublisherId>().unwrap()
        );
    }

    #[test]
    fn equality() {
        let lower_id = "zj75k085cmj1a".parse::<PublisherId>().unwrap();
        let upper_id = "ZJ75K085CMJ1A".parse::<PublisherId>().unwrap();

        assert_eq!(lower_id, lower_id);
        assert_eq!(lower_id, upper_id);
        assert_ne!(lower_id, "yjp7t9tn9g0z0".parse::<PublisherId>().unwrap());
    }

    #[test]
    fn comparison() {
        let lower_id = "zj75k085cmj1a".parse::<PublisherId>().unwrap();
        let upper_id = "ZJ75K085CMJ1A".parse::<PublisherId>().unwrap();

        assert_eq!(lower_id.cmp(&lower_id), Ordering::Equal);
        assert_eq!(lower_id.cmp(&upper_id), Ordering::Equal);

        let other_id = "yjp7t9tn9g0z0".parse::<PublisherId>().unwrap();
        assert_eq!(lower_id.cmp(&other_id), Ordering::Greater);
        assert_eq!(other_id.cmp(&lower_id), Ordering::Less);
    }

    #[test]
    fn hash() {
        // If two keys are equal, their hashes must also be equal
        // https://doc.rust-lang.org/std/hash/trait.Hash.html#hash-and-eq

        let lower_id = "zj75k085cmj1a".parse::<PublisherId>().unwrap();
        let upper_id = "ZJ75K085CMJ1A".parse::<PublisherId>().unwrap();
        assert_eq!(lower_id, upper_id);

        let state = foldhash::fast::RandomState::default();
        let mut id_1_hasher = state.build_hasher();
        lower_id.hash(&mut id_1_hasher);

        let mut id_2_hasher = state.build_hasher();
        upper_id.hash(&mut id_2_hasher);

        assert_eq!(id_1_hasher.finish(), id_2_hasher.finish());
    }
}
