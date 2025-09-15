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

mod publisher_id;

use alloc::borrow::{Cow, ToOwned};
use core::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    str::FromStr,
};

pub use publisher_id::{PublisherId, PublisherIdError};
use thiserror::Error;

#[cfg(feature = "serde")]
mod serde;

/// A [Package Family Name] is an opaque string derived from only two parts of a package identity -
/// name and publisher.
///
/// `<Name>_<PublisherId>`
///
/// For example, the Package Family Name of the Windows Photos app is
/// `Microsoft.Windows.Photos_8wekyb3d8bbwe`, where `Microsoft.Windows.Photos` is the name and
/// `8wekyb3d8bbwe` is the publisher ID for Microsoft.
///
/// Package Family Name is often referred to as a 'version-less Package Full Name'.
///
/// [Package Family Name]: https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/package-identity-overview#package-family-name
#[derive(Clone, Debug, Default, Eq)]
pub struct PackageFamilyName<'ident> {
    package_name: Cow<'ident, str>,
    publisher_id: PublisherId,
}

impl<'ident> PackageFamilyName<'ident> {
    /// Creates a new Package Family Name from a package name and an identity publisher.
    ///
    /// This is equivalent to the Windows function [`PackageNameAndPublisherIdFromFamilyName`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use package_family_name::PackageFamilyName;
    /// let package_family_name = PackageFamilyName::new(
    ///     "Microsoft.PowerShell",
    ///     "CN=Microsoft Corporation, O=Microsoft Corporation, L=Redmond, S=Washington, C=US"
    /// );
    ///
    /// assert_eq!(package_family_name.to_string(), "Microsoft.PowerShell_8wekyb3d8bbwe");
    /// ```
    ///
    /// [`PackageNameAndPublisherIdFromFamilyName`]: https://learn.microsoft.com/en-us/windows/win32/api/appmodel/nf-appmodel-packagenameandpublisheridfromfamilyname
    #[must_use]
    pub fn new<T, S>(package_name: T, identity_publisher: S) -> Self
    where
        T: Into<Cow<'ident, str>>,
        S: AsRef<str>,
    {
        Self {
            package_name: package_name.into(),
            publisher_id: PublisherId::new(identity_publisher),
        }
    }

    /// Returns the package name as a string slice.
    ///
    /// # Examples
    ///
    /// ```
    /// # use package_family_name::PackageFamilyName;
    /// let package_family_name = PackageFamilyName::new(
    ///     "Microsoft.PowerShell",
    ///     "CN=Microsoft Corporation, O=Microsoft Corporation, L=Redmond, S=Washington, C=US"
    /// );
    ///
    /// assert_eq!(package_family_name.package_name(), "Microsoft.PowerShell");
    /// ```
    #[must_use]
    #[inline]
    pub fn package_name(&self) -> &str {
        &self.package_name
    }

    /// Returns a reference to the [Publisher Id].
    ///
    /// # Examples
    ///
    /// ```
    /// # use package_family_name::PackageFamilyName;
    /// let package_family_name = PackageFamilyName::new(
    ///     "Microsoft.PowerShell",
    ///     "CN=Microsoft Corporation, O=Microsoft Corporation, L=Redmond, S=Washington, C=US"
    /// );
    ///
    /// assert_eq!(package_family_name.publisher_id().as_str(), "8wekyb3d8bbwe");
    /// ```
    ///
    /// [Publisher Id]: PublisherId
    #[must_use]
    #[inline]
    pub const fn publisher_id(&self) -> &PublisherId {
        &self.publisher_id
    }
}

impl fmt::Display for PackageFamilyName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}_{}", self.package_name, self.publisher_id)
    }
}

impl PartialEq for PackageFamilyName<'_> {
    /// Tests for `self` and `other` values to be equal, and is used by `==`.
    ///
    /// Package Family Name is compared case-insensitively.
    ///
    /// # Examples
    ///
    /// ```
    /// # use package_family_name::PackageFamilyName;
    /// let pfn_1 = PackageFamilyName::new("PowerShell", "CN=, O=, L=, S=, C=");
    /// let pfn_2 = PackageFamilyName::new("powershell", "CN=, O=, L=, S=, C=");
    ///
    /// assert_eq!(pfn_1, pfn_2);
    /// ```
    fn eq(&self, other: &Self) -> bool {
        self.package_name()
            .eq_ignore_ascii_case(other.package_name())
            && self.publisher_id().eq(other.publisher_id())
    }
}

impl PartialOrd for PackageFamilyName<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PackageFamilyName<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.package_name()
            .as_bytes()
            .iter()
            .map(u8::to_ascii_lowercase)
            .cmp(
                other
                    .package_name()
                    .as_bytes()
                    .iter()
                    .map(u8::to_ascii_lowercase),
            )
            .then_with(|| self.publisher_id().cmp(other.publisher_id()))
    }
}

impl Hash for PackageFamilyName<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for byte in self.package_name().as_bytes() {
            state.write_u8(byte.to_ascii_lowercase());
        }
        state.write_u8(b'_');
        self.publisher_id().hash(state);
    }
}

#[derive(Error, Debug, Eq, PartialEq)]
pub enum PackageFamilyNameError {
    #[error(
        "Package Family Name must have an underscore (`_`) between the package name and Publisher Id"
    )]
    NoUnderscore,
    #[error(transparent)]
    PublisherId(#[from] PublisherIdError),
}

impl FromStr for PackageFamilyName<'_> {
    type Err = PackageFamilyNameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (package_name, publisher_id) = s.split_once('_').ok_or(Self::Err::NoUnderscore)?;

        Ok(Self {
            package_name: package_name.to_owned().into(),
            publisher_id: publisher_id.parse()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use core::{
        cmp::Ordering,
        hash::{BuildHasher, Hash, Hasher},
    };

    use super::PackageFamilyName;

    #[test]
    fn microsoft_windows_photos() {
        let package_family_name = PackageFamilyName::new(
            "Microsoft.Windows.Photos",
            "CN=Microsoft Corporation, O=Microsoft Corporation, L=Redmond, S=Washington, C=US",
        );

        assert_eq!(
            package_family_name.to_string(),
            "Microsoft.Windows.Photos_8wekyb3d8bbwe"
        );
    }

    #[test]
    fn hydraulic_conveyor_15() {
        let package_family_name = PackageFamilyName::new(
            "Conveyor",
            "CN=Hydraulic Software AG, O=Hydraulic Software AG, L=Zürich, S=Zürich, C=CH, SERIALNUMBER=CHE-312.597.948, OID.1.3.6.1.4.1.311.60.2.1.2=Zürich, OID.1.3.6.1.4.1.311.60.2.1.3=CH, OID.2.5.4.15=Private Organization",
        );

        assert_eq!(package_family_name.to_string(), "Conveyor_fg3qp2cw01ypp");
    }

    #[test]
    fn hydraulic_conveyor_16() {
        let package_family_name = PackageFamilyName::new(
            "Conveyor",
            "CN=Hydraulic Software AG, O=Hydraulic Software AG, L=Zürich, S=Zürich, C=CH, SERIALNUMBER=CHE-312.597.948, OID.2.5.4.15=Private Organization, OID.1.3.6.1.4.1.311.60.2.1.2=Zürich, OID.1.3.6.1.4.1.311.60.2.1.3=CH",
        );

        assert_eq!(package_family_name.to_string(), "Conveyor_r94jb655n6kcp");
    }

    #[test]
    fn equality() {
        let powershell_pfn_1 = "Microsoft.PowerShell_8wekyb3d8bbwe"
            .parse::<PackageFamilyName>()
            .unwrap();
        let powershell_pfn_2 = "microsoft.powerShell_8WEKYB3D8BBWE"
            .parse::<PackageFamilyName>()
            .unwrap();

        assert_eq!(powershell_pfn_1, powershell_pfn_1);
        assert_eq!(powershell_pfn_1, powershell_pfn_2);
        assert_ne!(
            powershell_pfn_1,
            "Conveyor_fg3qp2cw01ypp"
                .parse::<PackageFamilyName>()
                .unwrap()
        );
    }

    #[test]
    fn comparison() {
        let powershell_pfn_1 = "Microsoft.PowerShell_8wekyb3d8bbwe"
            .parse::<PackageFamilyName>()
            .unwrap();
        let powershell_pfn_2 = "microsoft.powerShell_8WEKYB3D8BBWE"
            .parse::<PackageFamilyName>()
            .unwrap();

        assert_eq!(powershell_pfn_1.cmp(&powershell_pfn_1), Ordering::Equal);
        assert_eq!(powershell_pfn_1.cmp(&powershell_pfn_2), Ordering::Equal);

        let conveyor_pfn = "Conveyor_fg3qp2cw01ypp"
            .parse::<PackageFamilyName>()
            .unwrap();
        assert_eq!(powershell_pfn_1.cmp(&conveyor_pfn), Ordering::Greater);
        assert_eq!(conveyor_pfn.cmp(&powershell_pfn_1), Ordering::Less);
    }

    #[test]
    fn hash() {
        // If two keys are equal, their hashes must also be equal
        // https://doc.rust-lang.org/std/hash/trait.Hash.html#hash-and-eq

        let package_family_name_1 = "Microsoft.PowerShell_8wekyb3d8bbwe"
            .parse::<PackageFamilyName>()
            .unwrap();
        let package_family_name_2 = "microsoft.powerShell_8WEKYB3D8BBWE"
            .parse::<PackageFamilyName>()
            .unwrap();
        assert_eq!(package_family_name_1, package_family_name_2);

        let state = foldhash::fast::RandomState::default();
        let mut package_family_name_1_hasher = state.build_hasher();
        package_family_name_1.hash(&mut package_family_name_1_hasher);

        let mut package_family_name_2_hasher = state.build_hasher();
        package_family_name_2.hash(&mut package_family_name_2_hasher);

        assert_eq!(
            package_family_name_1_hasher.finish(),
            package_family_name_2_hasher.finish()
        );
    }
}
