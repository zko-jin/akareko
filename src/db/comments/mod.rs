use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use serde::{Deserialize, Serialize};
use surrealdb::types::{SerializationError, SurrealValue};

use crate::{
    db::{
        Timestamp,
        index::{Index, content::Content, tags::IndexTag},
        user::User,
    },
    hash::{Hash, PublicKey, Signature},
};

// ==================== End Imports ====================

#[cfg(feature = "surrealdb")]
mod surreal;
#[cfg(feature = "surrealdb")]
pub use surreal::PostRepository;

#[derive(Clone, Debug, PartialEq, Hash, Eq, Serialize, Deserialize, byteable_derive::Byteable)]
pub struct Topic(#[serde(with = "serde_bytes")] [u8; 64]);

impl AsRef<[u8]> for Topic {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Topic {
    pub fn from_index<I: IndexTag>(index: &Index<I>) -> Self {
        Self(index.hash().inner().clone())
    }

    pub fn from_post(post: &Post) -> Self {
        Self(post.signature.clone().to_inner())
    }

    pub fn from_content<I: IndexTag>(content: &Content<I>) -> Self {
        Self(content.signature().clone().to_inner())
    }

    pub fn from_user(user: &User) -> Self {
        let mut bytes: [u8; 64] = [0; 64];
        bytes[..32].copy_from_slice(user.pub_key().as_bytes());
        Self(bytes)
    }

    pub fn from_entry<I: IndexTag>(index: &Index<I>, enumeration: f32) -> Self {
        let mut bytes = index.hash().inner().to_vec();
        bytes.extend(enumeration.to_le_bytes());
        Self(Hash::digest(&bytes).to_inner())
    }

    pub fn from_bytes(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }

    pub fn as_base64(&self) -> String {
        BASE64_STANDARD_NO_PAD.encode(&self.0)
    }

    pub fn inner(&self) -> &[u8; 64] {
        &self.0
    }

    pub fn to_inner(&self) -> [u8; 64] {
        self.0
    }
}

impl SurrealValue for Topic {
    fn kind_of() -> surrealdb::types::Kind {
        surrealdb::types::Kind::Bytes
    }

    fn into_value(self) -> surrealdb::types::Value {
        surrealdb::types::Value::Bytes(bytes::Bytes::from_owner(self).into())
    }

    fn from_value(value: surrealdb::types::Value) -> Result<Self, surrealdb::Error>
    where
        Self: Sized,
    {
        let bytes = match value.as_bytes() {
            Some(b) => b,
            None => {
                return Err(surrealdb::Error::serialization(
                    "Topic can only be made from bytes".to_string(),
                    Some(SerializationError::Deserialization),
                ));
            }
        };

        if bytes.len() != 64 {
            return Err(surrealdb::Error::serialization(
                "Topic needs 64 bytes".to_string(),
                Some(SerializationError::Deserialization),
            ));
        }

        //TODO: zero copy
        let b: &[u8] = bytes.as_ref();

        Ok(Topic(b.try_into().unwrap()))
    }
}

pub struct CachedSyncs {
    pub topic: Topic,
    pub source: PublicKey,
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, SurrealValue, byteable_derive::Byteable)]
pub struct Post {
    #[surreal(rename = "id")]
    pub signature: Signature,

    // #[cfg_attr(
    //     feature = "surrealdb",
    //     serde(
    //         serialize_with = "serialize_pubkey_as_user_id",
    //         deserialize_with = "deserialize_record_id_as_pubkey",
    //     )
    // )]
    /// Who posted
    pub source: PublicKey,

    pub topic: Topic,

    pub timestamp: Timestamp,
    pub content: String,

    // Unsigned
    pub received_at: Timestamp,
}

impl std::hash::Hash for Post {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.signature.hash(state);
    }
}

// fn serialize_pubkey_as_user_id<S>(key: &PublicKey, serializer: S) -> Result<S::Ok, S::Error>
// where
//     S: serde::Serializer,
// {
//     let record_id = RecordId::from_table_key(User::TABLE_NAME, key.to_base64());
//     record_id.serialize(serializer)
// }

// fn deserialize_record_id_as_pubkey<'de, D>(deserializer: D) -> Result<PublicKey, D::Error>
// where
//     D: serde::Deserializer<'de>,
// {
//     let id = RecordId::deserialize(deserializer)?;
//     let key = id.key.into_value().as_string().unwrap();
//     let trimmed = key.trim_start_matches("`").trim_end_matches("`");

//     PublicKey::from_base64(&trimmed).map_err(serde::de::Error::custom)
// }

impl Post {
    pub const TABLE_NAME: &str = "posts";

    pub fn new(
        content: String,
        timestamp: Timestamp,
        received_at: Timestamp,
        source: PublicKey,
        topic: Topic,
        signature: Signature,
    ) -> Self {
        Self {
            source,
            signature,
            topic,
            timestamp,
            content,
            received_at,
        }
    }

    pub fn new_signed(
        content: String,
        timestamp: Timestamp,
        received_at: Timestamp,
        topic: Topic,
        priv_key: &crate::hash::PrivateKey,
    ) -> Self {
        let mut comment = Self::new(
            content,
            timestamp,
            received_at,
            priv_key.public_key(),
            topic,
            Signature::empty(),
        );
        comment.sign(priv_key);
        comment
    }

    fn sign_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = self.topic.inner().to_vec();
        bytes.extend(self.content.as_bytes());
        bytes.extend(self.timestamp.to_le_bytes());
        bytes
    }

    fn sign(&mut self, priv_key: &crate::hash::PrivateKey) {
        let to_sign = self.sign_bytes();
        self.signature = priv_key.sign(&to_sign);
    }

    pub fn verify(&self) -> bool {
        let to_verify = self.sign_bytes();
        self.source.verify(&to_verify, &self.signature)
    }
}
