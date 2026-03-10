use surrealdb::types::SurrealValue;

use crate::{
    db::{Timestamp, ToBytes},
    types::{PublicKey, Signature, String16, Topic},
};

// ==================== End Imports ====================

#[cfg(feature = "surrealdb")]
mod surreal;

// pub struct CachedSyncs {
//     pub topic: Topic,
//     pub source: PublicKey,
//     pub timestamp: Timestamp,
// }

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
    pub content: String16,
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
        content: String16,
        timestamp: Timestamp,
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
        }
    }

    pub fn new_signed(
        content: String16,
        timestamp: Timestamp,
        topic: Topic,
        priv_key: &crate::types::PrivateKey,
    ) -> Self {
        let mut comment = Self::new(
            content,
            timestamp,
            priv_key.public_key(),
            topic,
            Signature::empty(),
        );
        comment.sign(priv_key);
        comment
    }

    fn sign_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = self.topic.inner().to_vec();
        bytes.extend(self.content.inner().as_bytes());
        bytes.extend(self.timestamp.to_bytes());
        bytes
    }

    fn sign(&mut self, priv_key: &crate::types::PrivateKey) {
        let to_sign = self.sign_bytes();
        self.signature = priv_key.sign(&to_sign);
    }

    pub fn verify(&self) -> bool {
        let to_verify = self.sign_bytes();
        self.source.verify(&to_verify, &self.signature)
    }
}
