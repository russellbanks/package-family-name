/*!
Package Family Name is a Rust crate for calculating MSIX Package Family Name values.

Every MSIX application has a package family name value, which looks a bit like
`AppName_zj75k085cmj1a`. This value can easily be found by running `Get-AppxPackage <name>` in
PowerShell for an installed MSIX package and scrolling to `PackageFullName`.

However, we can work out a package family name value without needing to install the package at all.
That's where this library comes into play.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
package-family-name = "2"
```

```
# use package_family_name::PackageFamilyName;
let package_family_name = PackageFamilyName::new(
    "Microsoft.PowerShell",
    "CN=Microsoft Corporation, O=Microsoft Corporation, L=Redmond, S=Washington, C=US"
);

assert_eq!(package_family_name.to_string(), "Microsoft.PowerShell_8wekyb3d8bbwe");
```

## How a package family name is calculated

In short, a package family name is made up of two parts:

- Identity name (`Microsoft.PowerShell`)
- Identity publisher (`CN=Microsoft Corporation, O=Microsoft Corporation, L=Redmond, S=Washington, C=US`)

These steps are then taken:

1. UTF-16 encode the identity publisher
2. Calculate a SHA256 hash of the encoded publisher
3. Take the first 8 bytes of the hash
4. Encode the result with [Douglas Crockford Base32](http://www.crockford.com/base32.html)
5. Join the identity name and the encoded value with an underscore (`Microsoft.PowerShell_8wekyb3d8bbwe`)

### Why would I need to calculate a package family name?

Whilst this is a niche library, there are use cases. For example, when submitting an MSIX package to
[winget-pkgs](https://github.com/microsoft/winget-pkgs), a package family name value is a required
as part of the manifest.

## Acknowledgements

[@marcinotorowski](https://github.com/marcinotorowski) has produced a step by step explanation of
how to calculate the hash part of the package family name.
This post can be found
[here](https://marcinotorowski.com/2021/12/19/calculating-hash-part-of-msix-package-family-name).
*/

#![doc(html_root_url = "https://docs.rs/package-family-name")]
#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;

use alloc::borrow::ToOwned;
use alloc::string::String;
use core::fmt;

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
    #[must_use]
    pub fn new(identity_name: &str, identity_publisher: &str) -> Self {
        Self {
            identity_name: identity_name.to_owned(),
            publisher_id: Self::get_id(identity_publisher),
        }
    }

    #[must_use]
    pub fn get_id(identity_publisher: &str) -> PublisherId {
        const HASH_TRUNCATION_LENGTH: usize = 8;

        let publisher_sha_256 = identity_publisher
            .encode_utf16()
            .fold(Sha256::new(), |hasher, char| {
                hasher.chain_update(char.to_le_bytes())
            })
            .finalize();

        let truncated_hash = &publisher_sha_256[..HASH_TRUNCATION_LENGTH];
        let crockford_encoded = CROCKFORD_LOWER.encode(truncated_hash);

        // An 8-byte array encoded with crockford base32 always has an expected length of 13
        crockford_encoded.parse().unwrap()
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
