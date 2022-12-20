use std::str::FromStr;

use serde::{de, Deserialize, Serialize};

use crate::util::SerdeVisitor;

#[derive(Debug, Clone, Eq, PartialOrd, Ord)]
pub struct UtcDateTime(hifitime::Epoch);

impl UtcDateTime {
    pub fn now() -> Option<Self> {
        let Ok(epoch) = hifitime::Epoch::now() else {
            return None;
        };

        Some(Self(epoch))
    }

    pub fn epoch(&self) -> &hifitime::Epoch {
        &self.0
    }
}

impl std::hash::Hash for UtcDateTime {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_string().hash(state);
    }
}

impl std::cmp::PartialEq for UtcDateTime {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Serialize for UtcDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{}", self.0))
    }
}

impl<'de> Deserialize<'de> for UtcDateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = deserializer.deserialize_str(SerdeVisitor)?;

        let epoch = hifitime::Epoch::from_str(&s).map_err(de::Error::custom)?;

        Ok(Self(epoch))
    }
}
