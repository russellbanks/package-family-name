#![no_std]

extern crate alloc;

use alloc::format;
use alloc::string::String;
use fast32::base32::CROCKFORD_LOWER;
use sha2::{Digest, Sha256};

pub fn get_package_family_name(identity_name: &str, identity_publisher: &str) -> String {
    format!("{identity_name}_{}", get_publisher_hash(identity_publisher))
}

pub fn get_publisher_hash(identity_publisher: &str) -> String {
    let publisher_sha_256 = identity_publisher
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .fold(Sha256::new(), |buf, byte| buf.chain_update([byte]))
        .finalize();

    CROCKFORD_LOWER.encode(&publisher_sha_256[..8])
}

#[cfg(test)]
mod tests {
    use crate::{get_package_family_name, get_publisher_hash};

    #[test]
    fn test_package_family_name() {
        assert_eq!(
            get_package_family_name("AppName", "Publisher Software"),
            "AppName_zj75k085cmj1a"
        );
    }

    #[test]
    fn test_publisher_hash() {
        assert_eq!(get_publisher_hash("Publisher Software"), "zj75k085cmj1a");
    }
}
