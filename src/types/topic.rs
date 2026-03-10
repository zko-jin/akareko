use base64::{Engine as _, prelude::BASE64_STANDARD_NO_PAD};

use crate::{
    db::{
        comments::Post,
        index::{Index, content::Content, tags::IndexTag},
        user::User,
    },
    types::{Hash, Signature},
};

#[derive(Clone, Debug, PartialEq, std::hash::Hash, Eq, byteable_derive::Byteable)]
pub struct Topic([u8; 64]);

impl AsRef<[u8]> for Topic {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Topic {
    pub fn from_index<I: IndexTag>(index: &Index<I>) -> Self {
        Self(index.hash().inner().clone())
    }

    pub fn from_signature(signature: Signature) -> Self {
        Self(signature.to_inner())
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

impl surrealdb_types::SurrealValue for Topic {
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
                    Some(surrealdb_types::SerializationError::Deserialization),
                ));
            }
        };

        if bytes.len() != 64 {
            return Err(surrealdb::Error::serialization(
                "Topic needs 64 bytes".to_string(),
                Some(surrealdb_types::SerializationError::Deserialization),
            ));
        }

        //TODO: zero copy
        let b: &[u8] = bytes.as_ref();

        Ok(Topic(b.try_into().unwrap()))
    }
}
