use std::fmt::{Display, Formatter};

use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use surrealdb::types::SurrealValue;

use crate::{
    db::Timestamp,
    hash::{PrivateKey, PublicKey, Signable, Signature},
};

#[cfg(feature = "sqlite")]
mod sqlite;
#[cfg(feature = "sqlite")]
pub use sqlite::UserRepository;
#[cfg(feature = "surrealdb")]
mod surreal;
#[cfg(feature = "surrealdb")]
pub use surreal::UserRepository;

#[derive(Debug, Clone, TryFromPrimitive, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum TrustLevel {
    Ignore,     // Also used for your own user
    Unverified, // Default for users we haven't verified the address
    Untrusted,  // Default for users we have verified the address
    Trusted,
    FullTrust, // Set manually for sources
}

impl SurrealValue for TrustLevel {
    fn kind_of() -> surrealdb_types::Kind {
        surrealdb_types::Kind::Number
    }

    fn into_value(self) -> surrealdb_types::Value {
        (self as u8).into_value()
    }

    fn from_value(value: surrealdb_types::Value) -> Result<Self, surrealdb::Error>
    where
        Self: Sized,
    {
        let value = u8::from_value(value)?;
        value
            .try_into()
            .map_err(|e: num_enum::TryFromPrimitiveError<TrustLevel>| {
                surrealdb::Error::internal(e.to_string())
            })
    }
}

impl TrustLevel {
    /// Used for selecting in UI
    pub const ALL: [TrustLevel; 5] = [
        TrustLevel::Ignore,
        TrustLevel::Unverified,
        TrustLevel::Untrusted,
        TrustLevel::Trusted,
        TrustLevel::FullTrust,
    ];
}

impl Display for TrustLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TrustLevel::Ignore => write!(f, "Ignored"),
            TrustLevel::Unverified => write!(f, "Unverified"),
            TrustLevel::Untrusted => write!(f, "Untrusted"),
            TrustLevel::Trusted => write!(f, "Trusted"),
            TrustLevel::FullTrust => write!(f, "Full trust"),
        }
    }
}

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    Hash,
    PartialEq,
    SurrealValue,
    Eq,
    byteable_derive::Byteable,
)]
pub struct I2PAddress(String);

impl I2PAddress {
    pub fn new(address: impl Into<String>) -> I2PAddress {
        I2PAddress(address.into())
    }

    pub fn inner(&self) -> &String {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Signable for I2PAddress {
    fn sign(&self, private_key: &PrivateKey) -> Signature {
        private_key.sign(self.0.as_bytes())
    }

    fn verify(&self, public_key: &PublicKey, signature: &Signature) -> bool {
        public_key.verify(self.0.as_bytes(), signature)
    }
}

impl Display for I2PAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, SurrealValue)]
pub struct User {
    #[surreal(rename = "id")]
    pub_key: PublicKey,
    name: String,
    timestamp: Timestamp,
    signature: Signature,
    /// To prevent a user from faking the address of another user we need to  confirm the address
    /// by directly querying the address and asking for confirmation.
    /// To check if the address has been confirmed, we check the trust level.
    address: I2PAddress,

    // Unsigned fields
    trust: TrustLevel,
}

// Convert "<table>:<base64>" -> PublicKey
// pub fn deserialize_pubkey_id<'de, D>(deserializer: D) -> Result<PublicKey, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let id = RecordId::deserialize(deserializer)?;
//     let key = id.key.into_value().as_string().unwrap();
//     dbg!(&key);
//     let trimmed = key.trim_start_matches("`").trim_end_matches("`");

//     PublicKey::from_base64(&trimmed)
//         .map_err(|e| serde::de::Error::custom(format!("Invalid public key: {}", e)))
// }

// pub fn deserialize_signature_id<'de, D>(deserializer: D) -> Result<Signature, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let id = RecordId::deserialize(deserializer)?;
//     let key = id.key.into_value().as_string().unwrap();
//     let trimmed = key.trim_start_matches("`").trim_end_matches("`");

//     Signature::from_base64(&trimmed)
//         .map_err(|e| serde::de::Error::custom(format!("Invalid signature: {}", e)))
// }

impl std::hash::Hash for User {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.pub_key.hash(state);
    }
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.pub_key == other.pub_key
    }
}

impl Eq for User {}

impl std::borrow::Borrow<PublicKey> for User {
    fn borrow(&self) -> &PublicKey {
        &self.pub_key
    }
}

impl User {
    pub const TABLE_NAME: &str = "users";

    pub fn new(
        name: String,
        timestamp: u64,
        pub_key: PublicKey,
        signature: Signature,
        address: I2PAddress,
    ) -> User {
        User {
            pub_key,
            name,
            timestamp,
            address,
            signature,
            trust: TrustLevel::Unverified,
        }
    }

    pub fn new_signed(
        name: String,
        timestamp: u64,
        priv_key: &PrivateKey,
        address: I2PAddress,
    ) -> User {
        let mut user = User::new(
            name,
            timestamp,
            priv_key.public_key(),
            Signature::empty(),
            address,
        );
        user.sign(priv_key);
        user
    }

    pub fn verification_bytes(&self) -> Vec<u8> {
        let mut bytes = self.name.as_bytes().to_vec();
        bytes.extend(self.timestamp.to_le_bytes());
        bytes.extend(self.address.inner().as_bytes());
        bytes
    }

    fn sign(&mut self, priv_key: &PrivateKey) {
        let to_sign = self.verification_bytes();
        self.signature = priv_key.sign(&to_sign);
    }

    pub fn verify(&self) -> bool {
        let to_verify = self.verification_bytes();
        self.pub_key.verify(&to_verify, &self.signature)
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn address(&self) -> &I2PAddress {
        &self.address
    }

    pub fn set_address(&mut self, address: I2PAddress) {
        self.address = address;
    }

    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    pub fn pub_key(&self) -> &PublicKey {
        &self.pub_key
    }

    pub fn trust(&self) -> &TrustLevel {
        &self.trust
    }

    pub fn set_trust(&mut self, trust: TrustLevel) {
        self.trust = trust;
    }

    pub fn as_tuple(self) -> (PublicKey, String, u64, I2PAddress, Signature, TrustLevel) {
        (
            self.pub_key,
            self.name,
            self.timestamp,
            self.address,
            self.signature,
            self.trust,
        )
    }
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
