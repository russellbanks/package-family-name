use alloc::string::ToString;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{PackageFamilyName, PublisherId};

impl Serialize for PackageFamilyName<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de, 'ident> Deserialize<'de> for PackageFamilyName<'ident> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let deserialized_package_family_name = <&str>::deserialize(deserializer)?;

        deserialized_package_family_name
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

impl Serialize for PublisherId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for PublisherId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let deserialized_id = <&str>::deserialize(deserializer)?;
        deserialized_id.parse().map_err(serde::de::Error::custom)
    }
}
