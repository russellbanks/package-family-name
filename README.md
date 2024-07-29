# Package Family Name

[![Test Status](https://github.com/russellbanks/package-family-name/workflows/Tests/badge.svg)](https://github.com/russellbanks/package-family-name/actions)

A Rust library for calculating MSIX Package Family Name values.

This is a `#![no_std]` library.

Every MSIX application has a package family name value, which looks a bit like `AppName_zj75k085cmj1a`. This value can
easily be found by running `Get-AppxPackage <name>` in PowerShell for an installed MSIX package and scrolling to
`PackageFullName`.

However, we can work out a package family name value without needing to install the package at all. That's where this
library comes into play.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
package-family-name = "2"
```

```rust
let package_family_name = PackageFamilyName::new("AppName", "Publisher Software"); // AppName_zj75k085cmj1a
```

## How a package family name is calculated

In short, a package family name is made up of two parts:

- Identity name (`AppName`)
- Identity publisher (`Publisher Software`)

These steps are then taken:

1. UTF-16 encode the identity publisher
2. Calculate a SHA256 hash of the encoded publisher
3. Take the first 8 bytes of the hash
4. Encode the result with [Douglas Crockford Base32](http://www.crockford.com/base32.html)
5. Join the identity name and the encoded value with an underscore (`AppName_zj75k085cmj1a`)

### Why would I need to calculate a package family name?

Whilst this is a niche library, there are use cases. For example, when submitting an MSIX package to
[winget-pkgs](https://github.com/microsoft/winget-pkgs), a package family name value is a required as part of the
manifest.

## Acknowledgements

[@marcinotorowski](https://github.com/marcinotorowski) has produced a step by step explanation of how to calculate the
hash part of the package family name.
This post can be found
[here](https://marcinotorowski.com/2021/12/19/calculating-hash-part-of-msix-package-family-name).

## License

Licensed under either of:

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE.md) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT.md) or https://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
