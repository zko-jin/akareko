use std::{fmt::Display, str::FromStr};

use base64::{
    Engine as _, engine::general_purpose::STANDARD_NO_PAD, prelude::BASE64_URL_SAFE_NO_PAD,
};
#[cfg(feature = "diesel")]
use diesel::{deserialize::FromSqlRow, expression::AsExpression};
use serde::{Deserialize, Serialize};
use sha2::Digest;

use surrealdb_types::{SerializationError, SurrealValue};

use crate::errors::Base64Error;

mod keys;
mod string;
mod timestamp;
mod topic;
pub use keys::{PrivateKey, PublicKey, Signable, Signature};
pub use string::*;
pub use timestamp::Timestamp;
pub use topic::Topic;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    // AsExpression,
    // FromSqlRow,
    byteable_derive::Byteable,
)]
// #[sql_type = "diesel::sql_types::Binary"]
pub struct Hash(#[serde(with = "serde_bytes")] [u8; 64]);

impl FromStr for Hash {
    type Err = Base64Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Hash::from_base64(s)
    }
}

#[cfg(feature = "sqlite")]
pub mod sqlite {
    use diesel::{
        deserialize::FromSql,
        serialize::{self, IsNull, Output, ToSql},
        sql_types::Binary,
        sqlite::{Sqlite, SqliteValue},
    };

    use crate::hash::Hash;

    impl FromSql<Binary, Sqlite> for Hash {
        fn from_sql(bytes: SqliteValue) -> diesel::deserialize::Result<Hash> {
            let value = match <Vec<u8> as FromSql<Binary, Sqlite>>::from_sql(bytes)?.try_into() {
                Ok(value) => value,
                Err(e) => return Err(format!("Invalid hash size").into()),
            };

            Ok(Hash(value))
        }
    }

    impl ToSql<Binary, Sqlite> for Hash {
        fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
            out.set_value(&self.0[..]);
            Ok(IsNull::No)
        }
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl SurrealValue for Hash {
    fn kind_of() -> surrealdb_types::Kind {
        surrealdb_types::Kind::String
    }

    fn into_value(self) -> surrealdb_types::Value {
        // surrealdb_types::Value::Bytes(Bytes::from(bytes::Bytes::from_owner(self)))
        surrealdb_types::Value::String(self.as_base64())
    }

    fn from_value(value: surrealdb_types::Value) -> Result<Self, surrealdb::Error>
    where
        Self: Sized,
    {
        match value.kind() {
            surrealdb::types::Kind::String => {
                Ok(Hash::from_base64(value.as_string().unwrap()).unwrap())
            }
            surrealdb_types::Kind::Record(_) => Ok(Hash::from_base64(
                value
                    .into_record()
                    .unwrap()
                    .key
                    .into_value()
                    .as_string()
                    .unwrap(),
            )
            .unwrap()),
            _ => Err(surrealdb::Error::serialization(
                "Hash can only be made from string".to_string(),
                Some(SerializationError::Deserialization),
            )),
        }
        // let bytes = value.as_bytes().unwrap();
        // let hash = bytes.as_ref().try_into().unwrap();
        // Ok(Hash(hash))
    }
}

impl std::hash::Hash for Hash {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_base64())
    }
}

impl Hash {
    pub fn new(hash: [u8; 64]) -> Self {
        Hash(hash)
    }

    pub fn digest(bytes: &[u8]) -> Self {
        let hash = sha2::Sha512::digest(bytes);
        Hash(hash.into())
    }

    pub fn inner(&self) -> &[u8; 64] {
        &self.0
    }

    pub fn to_inner(&self) -> [u8; 64] {
        self.0
    }

    pub fn as_base64(&self) -> String {
        BASE64_URL_SAFE_NO_PAD.encode(&self.0)
    }

    pub fn from_base64(base64: &str) -> Result<Self, Base64Error> {
        let bytes = BASE64_URL_SAFE_NO_PAD.decode(base64)?;

        match bytes.try_into() {
            Ok(hash) => Ok(Hash(hash)),
            Err(b) => Err(Base64Error::InvalidLength {
                actual: b.len(),
                expected: 64,
            }), //TODO: Add proper error
        }
    }
}
