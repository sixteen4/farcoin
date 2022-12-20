use hex::{FromHex, ToHex};
use serde::{de, Deserialize, Serialize};

use crate::{util::SerdeVisitor, Hash};

#[derive(Debug, Clone, Eq)]
pub struct PublicKey(k256::ecdsa::VerifyingKey);

#[derive(Debug, Clone, Eq)]
pub struct PrivateKey(k256::ecdsa::SigningKey);

#[derive(Debug, Clone, Eq)]
pub struct Signature(k256::ecdsa::Signature);

impl PublicKey {
    pub fn verify(&self, hash: &Hash, signature: &Signature) -> bool {
        use k256::ecdsa::signature::Verifier;

        self.0.verify(hash.bytes(), &signature.0).is_ok()
    }
}

impl From<&PrivateKey> for PublicKey {
    fn from(key: &PrivateKey) -> Self {
        Self(key.0.verifying_key())
    }
}

impl From<&PublicKey> for String {
    fn from(key: &PublicKey) -> Self {
        key.0.to_bytes().encode_hex::<String>()
    }
}

impl<'a> TryFrom<&'a str> for PublicKey {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let Ok(bytes) = Vec::<u8>::from_hex(value) else {
            return Err(());
        };

        let Ok(key) = k256::ecdsa::VerifyingKey::from_sec1_bytes(&bytes) else {
            return Err(());
        };

        Ok(Self(key))
    }
}

impl std::hash::Hash for PublicKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let bytes = self.0.to_bytes();

        state.write(&bytes);
    }
}

impl std::cmp::PartialEq for PublicKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl PrivateKey {
    pub fn random(rng: impl rand_core::CryptoRng + rand_core::RngCore) -> Self {
        Self(k256::ecdsa::SigningKey::random(rng))
    }

    pub fn sign(&self, hash: &Hash) -> Option<Signature> {
        use k256::ecdsa::signature::Signer;

        let Ok(signature) = self.0.try_sign(hash.bytes()) else {
            return None;
        };

        Some(Signature(signature))
    }

    pub fn sign_with_rng(
        &self,
        rng: impl rand_core::CryptoRng + rand_core::RngCore,
        hash: &Hash,
    ) -> Option<Signature> {
        use k256::ecdsa::signature::RandomizedSigner;

        let Ok(signature) = self.0.try_sign_with_rng(rng, hash.bytes()) else {
            return None;
        };

        Some(Signature(signature))
    }
}

impl From<&PrivateKey> for String {
    fn from(key: &PrivateKey) -> Self {
        key.0.to_bytes().encode_hex::<String>()
    }
}

impl<'a> TryFrom<&'a str> for PrivateKey {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let Ok(bytes) = Vec::<u8>::from_hex(value) else {
            return Err(());
        };

        let Ok(key) = k256::ecdsa::SigningKey::from_bytes(&bytes) else {
            return Err(());
        };

        Ok(Self(key))
    }
}

impl std::hash::Hash for PrivateKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let bytes = self.0.to_bytes();

        state.write(&bytes);
    }
}

impl std::cmp::PartialEq for PrivateKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl From<&Signature> for String {
    fn from(signature: &Signature) -> Self {
        signature.0.to_der().as_bytes().encode_hex::<String>()
    }
}

impl<'a> TryFrom<&'a str> for Signature {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let Ok(bytes) = Vec::<u8>::from_hex(value) else {
            return Err(());
        };

        let Ok(key) = k256::ecdsa::Signature::from_der(&bytes) else {
            return Err(());
        };

        Ok(Self(key))
    }
}

impl std::hash::Hash for Signature {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write(self.0.to_der().as_bytes());
    }
}

impl std::cmp::PartialEq for Signature {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let bytes = self.0.to_bytes();

        serializer.serialize_str(&bytes.encode_hex::<String>())
    }
}

impl Serialize for PrivateKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let bytes = self.0.to_bytes();

        serializer.serialize_str(&bytes.encode_hex::<String>())
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self.0.to_der().as_bytes().encode_hex::<String>();

        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = deserializer.deserialize_str(SerdeVisitor)?;

        let bytes = Vec::<u8>::from_hex(&s).map_err(de::Error::custom)?;

        let key = k256::ecdsa::VerifyingKey::from_sec1_bytes(&bytes).map_err(de::Error::custom)?;

        Ok(Self(key))
    }
}

impl<'de> Deserialize<'de> for PrivateKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = deserializer.deserialize_str(SerdeVisitor)?;

        let bytes = Vec::<u8>::from_hex(&s).map_err(de::Error::custom)?;

        let key = k256::ecdsa::SigningKey::from_bytes(&bytes).map_err(de::Error::custom)?;

        Ok(Self(key))
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = deserializer.deserialize_str(SerdeVisitor)?;

        let bytes = Vec::<u8>::from_hex(&s).map_err(de::Error::custom)?;

        let signature = k256::ecdsa::Signature::from_der(&bytes).map_err(de::Error::custom)?;

        Ok(Self(signature))
    }
}
