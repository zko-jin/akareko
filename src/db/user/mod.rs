use std::fmt::{Display, Formatter};

#[cfg(feature = "diesel")]
use diesel::{
    Selectable,
    deserialize::FromSqlRow,
    expression::AsExpression,
    prelude::{Insertable, Queryable, QueryableByName},
};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;
use surrealdb::types::SurrealValue;

use crate::{
    db::{Timestamp, ToBytes},
    types::{PrivateKey, PublicKey, Signable, Signature, String8},
};

#[cfg(feature = "sqlite")]
mod sqlite;
#[cfg(feature = "sqlite")]
pub use sqlite::UserRepository;
#[cfg(feature = "surrealdb")]
mod surreal;
#[cfg(feature = "surrealdb")]
pub use surreal::UserRepository;

#[derive(
    Debug,
    Clone,
    Copy,
    IntoPrimitive,
    TryFromPrimitive,
    Hash,
    PartialEq,
    Eq,
    Default, // FromSqlRow,
    // AsExpression,
    EnumIter,
)]
// #[diesel(sql_type = diesel::sql_types::Integer)]
#[repr(u8)]
pub enum TrustLevel {
    Ignore, // Also used for your own user
    #[default]
    Unverified, // Default for users we haven't verified the address
    Untrusted, // Default for users we have verified the address
    Trusted,
    FullTrust, // Set manually for sources
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
#[cfg_attr(
    feature = "diesel",
    sql_type = "diesel::sql_types::Text",
    derive(FromSqlRow, AsExpression)
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

#[derive(Debug, Clone, byteable_derive::Byteable)]
#[cfg_attr(feature = "surrealdb", derive(SurrealValue))]
pub struct User {
    #[cfg_attr(feature = "surrealdb", surreal(rename = "id"))]
    pub_key: PublicKey,
    name: String8,
    timestamp: Timestamp,
    signature: Signature,
    /// To prevent a user from faking the address of another user we need to  confirm the address
    /// by directly querying the address and asking for confirmation.
    /// To check if the address has been confirmed, we check the trust level.
    address: I2PAddress,

    // Unsigned fields
    #[byteable(skip)]
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
        name: String8,
        timestamp: Timestamp,
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
        name: String8,
        timestamp: Timestamp,
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
        let mut bytes = self.name.inner().as_bytes().to_vec();
        bytes.extend(self.timestamp.to_bytes());
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

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    pub fn address(&self) -> &I2PAddress {
        &self.address
    }

    pub fn into_address(self) -> I2PAddress {
        self.address
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

    pub fn into_pub_key(self) -> PublicKey {
        self.pub_key
    }

    pub fn trust(&self) -> &TrustLevel {
        &self.trust
    }

    pub fn set_trust(&mut self, trust: TrustLevel) {
        self.trust = trust;
    }
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
