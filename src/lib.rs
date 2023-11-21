#![no_std]

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;
use core::ops::Add;
use sha2::{Digest, Sha256};

const CROCKFORD_CHARACTER_TABLE: &[u8; 32] = b"0123456789abcdefghjkmnpqrstvwxyz";

pub fn get_package_family_name(identity_name: &str, identity_publisher: &str) -> String {
    let result: String = identity_publisher
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .fold(Sha256::new(), |mut buf, byte| {
            buf.update([byte]);
            buf
        })
        .finalize()
        .into_iter()
        .take(8)
        .fold(String::with_capacity(65), |mut buf, byte| {
            let _ = write!(buf, "{:08b}", byte);
            buf
        })
        .add("0")
        .chars()
        .collect::<Vec<_>>()
        .chunks_exact(5)
        .map(|chunk| {
            let chunk_str = String::from_iter(chunk);
            let index = usize::from_str_radix(&chunk_str, 2).unwrap_or_default();
            CROCKFORD_CHARACTER_TABLE[index] as char
        })
        .collect();

    format!("{identity_name}_{}", result)
}

#[cfg(test)]
mod tests {
    use crate::get_package_family_name;

    #[test]
    fn test_package_family_name() {
        assert_eq!(
            get_package_family_name("AppName", "Publisher Software"),
            "AppName_zj75k085cmj1a"
        );
    }
}
