use hex::{FromHex, ToHex};
use serde::{de, Deserialize, Serialize};

use crate::util::SerdeVisitor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hash(Vec<u8>);

impl Hash {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn empty() -> Self {
        Self(vec![])
    }

    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}

impl Serialize for Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.encode_hex::<String>())
    }
}

impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = deserializer.deserialize_str(SerdeVisitor)?;

        let bytes = Vec::<u8>::from_hex(&s).map_err(de::Error::custom)?;

        Ok(Self(bytes))
    }
}
