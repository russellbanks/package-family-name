#![no_std]

extern crate alloc;

use alloc::borrow::ToOwned;
use alloc::string::String;
use core::fmt;
use core::str::FromStr;

use fast32::base32::CROCKFORD_LOWER;
use sha2::{Digest, Sha256};

use crate::publisher_id::PublisherId;

mod publisher_id;

#[cfg(feature = "serde")]
mod serde;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct PackageFamilyName {
    identity_name: String,
    publisher_id: PublisherId,
}

impl PackageFamilyName {
    pub fn new(identity_name: &str, identity_publisher: &str) -> Self {
        PackageFamilyName {
            identity_name: identity_name.to_owned(),
            publisher_id: Self::get_id(identity_publisher),
        }
    }

    pub fn get_id(identity_publisher: &str) -> PublisherId {
        const HASH_TRUNCATION_LENGTH: usize = 8;

        let publisher_sha_256 = identity_publisher
            .encode_utf16()
            .fold(Sha256::new(), |hasher, char| hasher.chain_update(char.to_le_bytes()))
            .finalize();

        let crockford_encoded = CROCKFORD_LOWER.encode(&publisher_sha_256[..HASH_TRUNCATION_LENGTH]);

        PublisherId(heapless::String::from_str(&crockford_encoded).unwrap())
    }
}

impl fmt::Display for PackageFamilyName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}_{}", self.identity_name, self.publisher_id)
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use crate::PackageFamilyName;

    #[test]
    fn test_package_family_name() {
        let package_family_name = PackageFamilyName::new("AppName", "Publisher Software");
        assert_eq!(package_family_name.to_string(), "AppName_zj75k085cmj1a");
    }

    #[test]
    fn test_publisher_id() {
        let publisher_id = PackageFamilyName::get_id("Publisher Software");
        assert_eq!(publisher_id.to_string(), "zj75k085cmj1a");
    }

    #[test]
    fn test_different_publishers() {
        let publisher_id1 = PackageFamilyName::get_id("Publisher Software");
        let publisher_id2 = PackageFamilyName::get_id("Another Publisher");
        assert_ne!(publisher_id1, publisher_id2);
    }
}
