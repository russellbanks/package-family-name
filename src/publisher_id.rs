use core::fmt;
use core::fmt::Display;
use core::str::FromStr;

const PUBLISHER_ID_LENGTH: usize = 13;

/// A 13-character long publisher ID
///
/// [Publisher Id](https://learn.microsoft.com/windows/apps/desktop/modernize/package-identity-overview#publisher-id)
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct PublisherId(pub heapless::String<PUBLISHER_ID_LENGTH>);

impl Display for PublisherId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for PublisherId {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != PUBLISHER_ID_LENGTH {
            return Err("expected publisher id length of 13");
        }
        Ok(PublisherId(heapless::String::from_str(s).unwrap()))
    }
}

#[cfg(test)]
mod tests {
    use crate::publisher_id::PublisherId;
    use core::str::FromStr;

    #[test]
    fn test_publisher_id_from_str() {
        assert!(PublisherId::from_str("zj75k085cmj1a").is_ok());
    }

    #[test]
    fn test_publisher_id_too_short() {
        assert!(PublisherId::from_str(&"1".repeat(3)).is_err());
    }

    #[test]
    fn test_publisher_id_too_long() {
        assert!(PublisherId::from_str(&"1".repeat(20)).is_err());
    }
}
