use std::time::{SystemTime, UNIX_EPOCH};

use base64::{Engine, prelude::BASE64_STANDARD};
use data_encoding::BASE32_NOPAD;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use surrealdb_types::SurrealValue;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use unicode_normalization::UnicodeNormalization;

use crate::{
    db::{Timestamp, user::I2PAddress},
    errors::{DecodeError, I2PParseError},
};

mod bloom_filter;
mod byteable;
mod lifo;
pub use byteable::{Byteable, Decodeable, Encodeable};
pub use lifo::LiFo;

#[derive(Debug, Clone)]
pub struct SanitizedString(String);

impl SanitizedString {
    pub fn new(s: &String) -> Self {
        let normalized: String = s
            .to_lowercase() // lowercase everything
            .nfd() // decompose accents
            .filter(|c| {
                c.is_ascii_alphanumeric() // keep only a-z, 0-9
            })
            .collect();

        SanitizedString(normalized)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub fn to_inner(self) -> String {
        self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue, byteable_derive::Byteable)]
#[repr(u16)]
pub enum Language {
    Japanese,
    English,
    French,
    Portuguese,
    Unknown,
}

pub fn now_timestamp() -> Timestamp {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

fn i2p_b64_fix(s: &str) -> String {
    s.trim().replace('-', "+").replace('~', "/")
}

pub fn b32_from_pub_b64(pub_b64: &str) -> Result<I2PAddress, I2PParseError> {
    let b64 = pub_b64
        .trim()
        .trim_end_matches(".b64.i2p")
        .trim_end_matches(".i2p");
    let fixed = i2p_b64_fix(b64);
    let decoded = BASE64_STANDARD.decode(fixed.as_bytes())?;
    let hash = Sha256::digest(&decoded);
    let b32 = BASE32_NOPAD.encode(&hash).to_lowercase();
    let b32_52 = b32.chars().take(52).collect::<String>();
    Ok(I2PAddress::new(format!("{}.b32.i2p", b32_52)))
}
