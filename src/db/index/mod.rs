use surrealdb_types::SurrealValue;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    db::{SurrealPhantom, Timestamp, index::tags::IndexTag},
    errors::{DecodeError, EncodeError},
    hash::{Hash, PrivateKey, PublicKey, Signature},
    helpers::{Byteable, SanitizedString, now_timestamp},
};

// ==================== End Imports ====================

pub mod content;
pub mod tags;

#[cfg(feature = "sqlite")]
mod sqlite;
#[cfg(feature = "sqlite")]
pub use sqlite::IndexRepository;
#[cfg(feature = "surrealdb")]
mod surreal;
#[cfg(feature = "surrealdb")]
pub use surreal::IndexRepository;

#[derive(Debug, Clone, SurrealValue)]
pub struct Index<T: IndexTag> {
    hash: Hash, // Primary Key
    title: String,
    release_date: i32,
    source: PublicKey,
    received_at: Timestamp,
    signature: Signature,
    #[surreal(skip, default)]
    _phantom: SurrealPhantom<T>,
}

impl<T: IndexTag> std::hash::Hash for Index<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl<T: IndexTag> Index<T> {
    pub fn new(title: String, release_date: i32, source: PublicKey, signature: Signature) -> Self {
        let hash = Hash::digest(&Self::id_bytes(&title, &release_date));

        Self {
            hash,
            title,
            release_date,
            source,
            signature,
            received_at: now_timestamp(), // Indexes created by the user
            _phantom: SurrealPhantom::default(),
        }
    }

    pub fn transmute<T2: IndexTag>(self) -> Index<T2> {
        // SAFETY: They're literally the same type, just different tags
        unsafe { std::mem::transmute(self) }
    }

    fn id_bytes(title: &String, release_date: &i32) -> Vec<u8> {
        let sanitized_title = SanitizedString::new(&title);

        let mut bytes = T::TAG.as_bytes().to_vec();
        bytes.extend(sanitized_title.as_bytes());
        bytes.extend(release_date.to_le_bytes());
        bytes
    }

    pub fn new_signed(title: String, release_date: i32, priv_key: &PrivateKey) -> Self {
        let mut index = Self::new(
            title,
            release_date,
            priv_key.public_key(),
            Signature::empty(),
        );

        index.sign(priv_key);

        index
    }

    fn sign(&mut self, priv_key: &PrivateKey) {
        let to_sign = Self::id_bytes(&self.title, &self.release_date);
        self.signature = priv_key.sign(&to_sign);
    }

    pub fn verify(&self) -> bool {
        let to_verify = Self::id_bytes(&self.title, &self.release_date);
        self.source.verify(&to_verify, &self.signature)
    }

    pub fn hash(&self) -> &Hash {
        &self.hash
    }

    pub fn title(&self) -> &String {
        &self.title
    }

    pub fn release_date(&self) -> i32 {
        self.release_date
    }

    pub fn source(&self) -> &PublicKey {
        &self.source
    }

    pub fn signature(&self) -> &Signature {
        &self.signature
    }
}

impl<T: IndexTag> Byteable for Index<T> {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        self.hash.encode(writer).await?;
        self.title.encode(writer).await?;
        self.release_date.encode(writer).await?;
        self.source.encode(writer).await?;
        self.signature.encode(writer).await?;
        Ok(())
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        Ok(Index {
            hash: Hash::decode(reader).await?,
            title: String::decode(reader).await?,
            release_date: i32::decode(reader).await?,
            source: PublicKey::decode(reader).await?,
            signature: Signature::decode(reader).await?,
            received_at: now_timestamp(),
            _phantom: SurrealPhantom::default(),
        })
    }
}
