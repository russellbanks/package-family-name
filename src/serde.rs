use crate::PackageFamilyName;
use crate::publisher_id::PublisherId;
use alloc::borrow::ToOwned;
use alloc::string::ToString;
use core::str::FromStr;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for PackageFamilyName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for PackageFamilyName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let deserialized_name = <&str>::deserialize(deserializer)?;
        let (identity_name, identity_id) =
            deserialized_name
                .split_once('_')
                .ok_or(serde::de::Error::custom(
                    "Invalid format for PackageFamilyName",
                ))?;
        let publisher_id = PublisherId::from_str(identity_id).map_err(serde::de::Error::custom)?;
        Ok(PackageFamilyName {
            identity_name: identity_name.to_owned(),
            publisher_id,
        })
    }
}

impl Serialize for PublisherId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for PublisherId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let deserialized_id = <&str>::deserialize(deserializer)?;
        PublisherId::from_str(deserialized_id).map_err(serde::de::Error::custom)
    }
}
