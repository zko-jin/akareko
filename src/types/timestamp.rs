use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use surrealdb_types::{ConversionError, Number, SurrealValue, Value};

use crate::db::ToBytes;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    byteable_derive::Byteable,
    Serialize,
    Deserialize,
)]
#[repr(transparent)]
pub struct Timestamp(i64);

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Add<i64> for Timestamp {
    type Output = Timestamp;

    fn add(self, rhs: i64) -> Self::Output {
        Timestamp(self.0 + rhs)
    }
}

impl std::ops::Add<Timestamp> for Timestamp {
    type Output = Timestamp;

    fn add(self, rhs: Timestamp) -> Self::Output {
        Timestamp(self.0 + rhs.0)
    }
}

impl std::ops::Sub<i64> for Timestamp {
    type Output = Timestamp;

    fn sub(self, rhs: i64) -> Self::Output {
        Timestamp(self.0 - rhs)
    }
}

impl std::ops::Sub<Timestamp> for Timestamp {
    type Output = Timestamp;

    fn sub(self, rhs: Timestamp) -> Self::Output {
        Timestamp(self.0 - rhs.0)
    }
}

impl Timestamp {
    pub fn new(timestamp: i64) -> Self {
        Timestamp(timestamp)
    }

    pub fn now() -> Timestamp {
        Timestamp(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs() as i64,
        )
    }
}

impl ToBytes for Timestamp {
    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }
}

impl SurrealValue for Timestamp {
    fn kind_of() -> surrealdb_types::Kind {
        surrealdb_types::Kind::Int
    }

    fn into_value(self) -> surrealdb_types::Value {
        Value::Number(Number::Int(self.0))
    }

    fn from_value(value: surrealdb_types::Value) -> Result<Self, surrealdb::Error>
    where
        Self: Sized,
    {
        let Value::Number(Number::Int(i)) = value else {
            return Err(ConversionError::from_value(Self::kind_of(), &value).into());
        };

        Ok(Timestamp(i))
    }
}
