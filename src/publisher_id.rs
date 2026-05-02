use core::{
    char,
    cmp::Ordering,
    fmt,
    fmt::{Debug, Formatter},
    hash::{Hash, Hasher},
    str::FromStr,
};

use sha2::{Digest, Sha256};
use thiserror::Error;

use super::crockford;

/// A Crockford Base32 encoded 13-character long [Publisher Id] derived from a Publisher.
///
///
/// [Publisher Id]: https://learn.microsoft.com/windows/apps/desktop/modernize/package-identity-overview#publisher-id
#[derive(Clone, Debug, Eq)]
#[repr(transparent)]
pub struct PublisherId([u8; Self::LENGTH]);

impl PublisherId {
    /// The constant length of a [Publisher Id].
    ///
    /// [Publisher Id]: PublisherId
    ///
    /// # Examples
    ///
    /// ```
    /// # use package_family_name::PublisherId;
    /// assert_eq!(PublisherId::LENGTH, 13);
    /// ```
    pub const LENGTH: usize = 13;

    /// Creates a new [Publisher Id] from an publisher
    ///
    /// [Publisher Id]: PublisherId
    #[must_use]
    pub fn new(publisher: &str) -> Self {
        // SHA-256 hash the UTF-16LE-encoded publisher
        let publisher_sha_256 = publisher
            .encode_utf16()
            .map(u16::to_le_bytes)
            .fold(Sha256::new(), Sha256::chain_update)
            .finalize();

        // Crockford Base32 encode the first 8 bytes of the SHA-256 hash
        let crockford_encoded = crockford::encode_lower(publisher_sha_256[..8].try_into().unwrap());

        Self(crockford_encoded)
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
    pub const fn as_str(&self) -> &str {
        // SAFETY: Inner bytes are Crockford Base32 characters and are therefore always valid UTF-8
        unsafe { str::from_utf8_unchecked(&self.0) }
    }

    /// Returns a byte slice of this `PublisherId`'s contents.
    ///
    /// # Examples
    ///
    /// ```
    /// # use package_family_name::PublisherId;
    /// let publisher_id = PublisherId::new("CN=Microsoft Corporation, O=Microsoft Corporation, L=Redmond, S=Washington, C=US");
    ///
    /// assert_eq!(publisher_id.as_bytes(), b"8wekyb3d8bbwe");
    #[must_use]
    #[inline]
    pub const fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<str> for PublisherId {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<[u8]> for PublisherId {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl Default for PublisherId {
    fn default() -> Self {
        // This isn't an ideal default but ensures that it will still have a fixed length of 13
        Self([b'0'; Self::LENGTH])
    }
}

impl fmt::Display for PublisherId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
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
            .iter()
            .map(u8::to_ascii_lowercase)
            .cmp(other.0.iter().map(u8::to_ascii_lowercase))
    }
}

impl Hash for PublisherId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for byte in self.as_bytes() {
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
        const CROCKFORD_OMITTED_CHARACTERS: [char; 4] = ['i', 'l', 'o', 'u'];

        if s.len() != Self::LENGTH {
            return Err(PublisherIdError::InvalidLength);
        }

        if s.chars().any(|char| {
            !char.is_ascii_alphanumeric()
                || CROCKFORD_OMITTED_CHARACTERS.contains(&char.to_ascii_lowercase())
        }) {
            return Err(PublisherIdError::InvalidCharacters);
        }

        Ok(Self(s.as_bytes().try_into().unwrap()))
    }
}

impl TryFrom<[u8; 13]> for PublisherId {
    type Error = PublisherIdError;

    fn try_from(value: [u8; 13]) -> Result<Self, Self::Error> {
        const CROCKFORD_OMITTED_CHARACTERS: [u8; 4] = [b'i', b'l', b'o', b'u'];

        if value.iter().any(|byte| {
            !byte.is_ascii_alphanumeric() || CROCKFORD_OMITTED_CHARACTERS.contains(byte)
        }) {
            return Err(PublisherIdError::InvalidCharacters);
        }

        Ok(Self(value))
    }
}

#[cfg(test)]
mod tests {
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

        // Case-insensitive equality
        assert_eq!(lower_id, upper_id);

        // Inequality
        assert_ne!(lower_id, "yjp7t9tn9g0z0".parse::<PublisherId>().unwrap());
    }

    #[test]
    fn ordering() {
        use core::cmp::Ordering;

        let lower_id = "zj75k085cmj1a".parse::<PublisherId>().unwrap();
        let upper_id = "ZJ75K085CMJ1A".parse::<PublisherId>().unwrap();

        assert_eq!(lower_id.cmp(&lower_id), Ordering::Equal);

        // Case-insensitive equality
        assert_eq!(lower_id.cmp(&upper_id), Ordering::Equal);

        let other_id = "yjp7t9tn9g0z0".parse::<PublisherId>().unwrap();
        assert_eq!(lower_id.cmp(&other_id), Ordering::Greater);
        assert_eq!(other_id.cmp(&lower_id), Ordering::Less);
    }

    #[test]
    fn hash() {
        use core::hash::BuildHasher;

        use rustc_hash::FxBuildHasher;

        // If two keys are equal, their hashes must also be equal
        // https://doc.rust-lang.org/std/hash/trait.Hash.html#hash-and-eq

        let lower_id = "zj75k085cmj1a".parse::<PublisherId>().unwrap();
        let upper_id = "ZJ75K085CMJ1A".parse::<PublisherId>().unwrap();
        assert_eq!(lower_id, upper_id);

        assert_eq!(
            FxBuildHasher.hash_one(lower_id),
            FxBuildHasher.hash_one(upper_id)
        );
    }

    #[test]
    fn size() {
        assert_eq!(size_of::<PublisherId>(), PublisherId::LENGTH);
    }

    #[test]
    fn alignment() {
        assert_eq!(align_of::<PublisherId>(), 1);
    }
}
