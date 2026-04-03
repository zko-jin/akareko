use std::fmt::{Display, Formatter};
use std::str::FromStr;

use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use base64::{Engine as _, engine::general_purpose::STANDARD_NO_PAD};

#[cfg(feature = "diesel")]
use ::diesel::deserialize::FromSqlRow;
#[cfg(feature = "diesel")]
use diesel::expression::AsExpression;
use ed25519_dalek::{SigningKey, ed25519::signature::SignerMut};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use surrealdb::types::{SerializationError, SurrealValue};
use zeroize::ZeroizeOnDrop;

use crate::errors::Base64Error;

#[derive(Serialize, Deserialize, Debug, Clone, ZeroizeOnDrop)]
#[serde(transparent)]
pub struct PrivateKey(#[serde(with = "serde_bytes")] [u8; 32]);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, byteable_derive::Byteable)]
#[serde(transparent)]
#[cfg_attr(
    feature = "diesel",
    sql_type = "diesel::sql_types::Binary",
    derive(FromSqlRow, AsExpression)
)]
pub struct PublicKey(#[serde(with = "serde_bytes")] pub(super) [u8; 32]);

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl SurrealValue for PublicKey {
    fn kind_of() -> surrealdb::types::Kind {
        surrealdb::types::Kind::String
    }

    fn into_value(self) -> surrealdb::types::Value {
        // surrealdb::types::Value::Bytes(bytes::Bytes::from_owner(self).into())
        surrealdb::types::Value::String(self.to_base64())
    }

    fn from_value(value: surrealdb::types::Value) -> Result<Self, surrealdb::Error>
    where
        Self: Sized,
    {
        match value.kind() {
            surrealdb::types::Kind::String => {
                Ok(PublicKey::from_base64(value.as_string().unwrap()).unwrap())
            }
            surrealdb_types::Kind::Record(_) => Ok(PublicKey::from_base64(
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
                "PublicKey can only be made from string".to_string(),
                Some(SerializationError::Deserialization),
            )),
        }
        // let bytes = match value.as_bytes() {
        //     Some(b) => b,
        //     None => {
        //         return Err(surrealdb::Error::serialization(
        //             "PublicKey can only be made from bytes".to_string(),
        //             Some(SerializationError::Deserialization),
        //         ));
        //     }
        // };

        // if bytes.len() != 32 {
        //     return Err(surrealdb::Error::serialization(
        //         "PublicKey needs 32 bytes".to_string(),
        //         Some(SerializationError::Deserialization),
        //     ));
        // }

        // //TODO: zero copy
        // let b: &[u8] = bytes.as_ref();

        // Ok(PublicKey(b.try_into().unwrap()))
    }
}

#[derive(Debug, Hash, Clone, PartialEq, Eq, byteable_derive::Byteable)]
#[cfg_attr(
    feature = "diesel",
    sql_type = "diesel::sql_types::Binary",
    derive(FromSqlRow, AsExpression)
)]
pub struct Signature(pub(super) [u8; 64]);

#[cfg(feature = "sqlite")]
pub mod sqlite {
    use diesel::{
        deserialize::FromSql,
        serialize::{self, IsNull, Output, ToSql},
        sql_types::Binary,
        sqlite::{Sqlite, SqliteValue},
    };

    use crate::hash::{PublicKey, Signature};

    impl FromSql<Binary, Sqlite> for Signature {
        fn from_sql(bytes: SqliteValue) -> diesel::deserialize::Result<Signature> {
            let value = match <Vec<u8> as FromSql<Binary, Sqlite>>::from_sql(bytes)?.try_into() {
                Ok(value) => value,
                Err(e) => return Err(format!("Invalid Signature size").into()),
            };

            Ok(Signature(value))
        }
    }

    impl ToSql<Binary, Sqlite> for Signature {
        fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
            out.set_value(&self.0[..]);
            Ok(IsNull::No)
        }
    }

    impl FromSql<Binary, Sqlite> for PublicKey {
        fn from_sql(bytes: SqliteValue) -> diesel::deserialize::Result<PublicKey> {
            let value = match <Vec<u8> as FromSql<Binary, Sqlite>>::from_sql(bytes)?.try_into() {
                Ok(value) => value,
                Err(e) => return Err(format!("Invalid PublicKey size").into()),
            };

            Ok(PublicKey(value))
        }
    }

    impl ToSql<Binary, Sqlite> for PublicKey {
        fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
            out.set_value(&self.0[..]);
            Ok(IsNull::No)
        }
    }
}

impl Display for Signature {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_base64())
    }
}

impl FromStr for Signature {
    type Err = Base64Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_base64(s)
    }
}

impl Signature {
    pub fn empty() -> Self {
        Signature([0u8; 64])
    }

    pub fn to_inner(self) -> [u8; 64] {
        self.0
    }

    pub fn as_base64(&self) -> String {
        BASE64_URL_SAFE_NO_PAD.encode(&self.0)
    }

    pub fn from_base64(base64: &str) -> Result<Self, Base64Error> {
        let bytes = BASE64_URL_SAFE_NO_PAD.decode(base64)?;

        match bytes.try_into() {
            Ok(hash) => Ok(Signature(hash)),
            Err(b) => Err(Base64Error::InvalidLength {
                expected: 64,
                actual: b.len(),
            }),
        }
    }

    pub unsafe fn from_bytes_unchecked(bytes: [u8; 64]) -> Self {
        Signature(bytes)
    }
}

impl AsRef<[u8]> for Signature {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl SurrealValue for Signature {
    fn kind_of() -> surrealdb::types::Kind {
        surrealdb::types::Kind::String
    }

    fn into_value(self) -> surrealdb::types::Value {
        // surrealdb::types::Value::Bytes(bytes::Bytes::from_owner(self).into())
        surrealdb::types::Value::String(self.as_base64())
    }

    fn from_value(value: surrealdb::types::Value) -> Result<Self, surrealdb::Error>
    where
        Self: Sized,
    {
        match value.kind() {
            surrealdb::types::Kind::String => {
                Ok(Self::from_base64(value.as_string().unwrap()).unwrap())
            }
            surrealdb_types::Kind::Record(_) => Ok(Self::from_base64(
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
                "Signature can only be made from string".to_string(),
                Some(SerializationError::Deserialization),
            )),
        }

        // let bytes = match value.as_bytes() {
        //     Some(b) => b,
        //     None => {
        //         return Err(surrealdb::Error::serialization(
        //             "Signature can only be made from bytes".to_string(),
        //             Some(SerializationError::Deserialization),
        //         ));
        //     }
        // };

        // if bytes.len() != 64 {
        //     return Err(surrealdb::Error::serialization(
        //         "Signature needs 64 bytes".to_string(),
        //         Some(SerializationError::Deserialization),
        //     ));
        // }

        // //TODO: zero copy
        // let b: &[u8] = bytes.as_ref();

        // Ok(Signature(b.try_into().unwrap()))
    }
}

impl PrivateKey {
    pub fn new() -> Self {
        let mut csprng = OsRng;
        let signing_key: SigningKey = SigningKey::generate(&mut csprng);

        PrivateKey(signing_key.to_bytes())
    }

    pub fn sign(&self, msg: &[u8]) -> Signature {
        let mut signing_key = ed25519_dalek::SigningKey::from_bytes(&self.0);
        let signature = signing_key.sign(msg);

        Signature(signature.to_bytes())
    }

    pub fn public_key(&self) -> PublicKey {
        let signing_key = ed25519_dalek::SigningKey::from(&self.0);
        PublicKey(signing_key.verifying_key().to_bytes())
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_base64(&self) -> String {
        STANDARD_NO_PAD.encode(&self.0)
    }

    pub fn from_base64(base64: &str) -> Result<Self, Base64Error> {
        let bytes = STANDARD_NO_PAD.decode(base64)?;

        match bytes.try_into() {
            Ok(hash) => Ok(PrivateKey(hash)),
            Err(b) => Err(Base64Error::InvalidLength {
                expected: 32,
                actual: b.len(),
            }),
        }
    }
}

impl PublicKey {
    pub fn verify(&self, msg: &[u8], signature: &Signature) -> bool {
        let signature = ed25519_dalek::Signature::from_bytes(&signature.0);
        let verifying_key = match ed25519_dalek::VerifyingKey::from_bytes(&self.0) {
            Ok(key) => key,
            Err(_) => return false,
        };
        verifying_key.verify_strict(msg, &signature).is_ok()
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_base64(&self) -> String {
        STANDARD_NO_PAD.encode(&self.0)
    }

    pub fn from_base64(base64: &str) -> Result<Self, Base64Error> {
        let bytes = STANDARD_NO_PAD.decode(base64)?;

        match bytes.try_into() {
            Ok(hash) => Ok(PublicKey(hash)),
            Err(b) => Err(Base64Error::InvalidLength {
                expected: 32,
                actual: b.len(),
            }),
        }
    }

    pub unsafe fn from_bytes_unchecked(bytes: [u8; 32]) -> Self {
        PublicKey(bytes)
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut str = String::new();
        for i in self.0 {
            str.push_str(&format!("{:02x}", i));
        }
        write!(f, "{}", str)
    }
}

pub trait Signable {
    fn sign(&self, private_key: &PrivateKey) -> Signature;
    fn verify(&self, public_key: &PublicKey, signature: &Signature) -> bool;
}
