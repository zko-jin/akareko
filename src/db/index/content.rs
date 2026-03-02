use surrealdb_types::SurrealValue;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    db::{Magnet, Timestamp, ToBytes, index::tags::IndexTag},
    errors::{DecodeError, EncodeError},
    hash::{Hash, PrivateKey, PublicKey, Signature},
    helpers::Byteable,
};

// ==================== End Imports ====================

#[derive(Debug, Clone, SurrealValue)]
pub struct Content<T: IndexTag> {
    #[surreal(rename = "id")]
    signature: Signature,
    source: PublicKey,

    // Signed Fields
    index_hash: Hash,
    pub timestamp: Timestamp,
    pub magnet_link: Magnet,
    entries: Vec<ContentEntry<T>>,
}

#[derive(Debug, Clone, SurrealValue)]
pub struct ContentEntry<T: IndexTag> {
    pub title: String,
    pub enumeration: f32,
    pub path: String,

    pub progress: f32,

    pub extra_metadata: T::ExtraMetadata,
}

impl<I: IndexTag> std::hash::Hash for Content<I> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index_hash.hash(state)
    }
}

impl<T: IndexTag> ToBytes for ContentEntry<T> {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = self.title.as_bytes().to_vec();
        bytes.extend(self.enumeration.to_be_bytes());
        bytes.extend(self.path.as_bytes());
        bytes.extend(self.extra_metadata.to_bytes());
        bytes
    }
}

impl<T: IndexTag> Byteable for ContentEntry<T> {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        self.title.encode(writer).await?;
        self.enumeration.encode(writer).await?;
        self.path.encode(writer).await?;
        self.extra_metadata.encode(writer).await?;
        Ok(())
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError>
    where
        Self: Sized,
    {
        Ok(ContentEntry {
            title: String::decode(reader).await?,
            enumeration: f32::decode(reader).await?,
            path: String::decode(reader).await?,
            extra_metadata: T::ExtraMetadata::decode(reader).await?,
            progress: 0.0,
        })
    }
}

impl<T: IndexTag> Content<T> {
    pub fn new(
        signature: Signature,
        source: PublicKey,
        index_hash: Hash,
        timestamp: Timestamp,
        magnet_link: Magnet,
        entries: Vec<ContentEntry<T>>,
    ) -> Self {
        Self {
            signature,
            source,
            index_hash,
            timestamp,
            magnet_link,
            entries,
        }
    }

    pub fn id_bytes(
        index_hash: &Hash,
        timestamp: &Timestamp,
        magnet_link: &Magnet,
        entries: &Vec<ContentEntry<T>>,
    ) -> Vec<u8> {
        let mut bytes: Vec<u8> = index_hash.inner().to_vec().to_vec();
        bytes.extend(timestamp.to_be_bytes());
        bytes.extend(magnet_link.0.as_bytes());
        for entry in entries {
            bytes.extend(entry.to_bytes());
        }
        bytes
    }

    pub fn new_signed(
        source: PublicKey,
        index_hash: Hash,
        timestamp: Timestamp,
        magnet_link: Magnet,
        entries: Vec<ContentEntry<T>>,
        priv_key: &PrivateKey,
    ) -> Self {
        let to_sign = Self::id_bytes(&index_hash, &timestamp, &magnet_link, &entries);
        let signature = priv_key.sign(&to_sign);

        Self::new(
            signature,
            source,
            index_hash,
            timestamp,
            magnet_link,
            entries,
        )
    }

    pub fn verify(&self) -> bool {
        let to_verify = Self::id_bytes(
            &self.index_hash,
            &self.timestamp,
            &self.magnet_link,
            &self.entries,
        );
        self.source.verify(&to_verify, &self.signature)
    }

    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    pub fn entries(&self) -> &Vec<ContentEntry<T>> {
        &self.entries
    }

    pub fn update_entry_progress(&mut self, index: usize, progress: f32) {
        self.entries[index].progress = progress;
    }

    pub fn index_hash(&self) -> &Hash {
        &self.index_hash
    }
}

impl<T: IndexTag> Byteable for Content<T> {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        self.signature.encode(writer).await?;
        self.source.encode(writer).await?;
        self.index_hash.encode(writer).await?;
        self.timestamp.encode(writer).await?;
        self.magnet_link.encode(writer).await?;
        self.entries.encode(writer).await?;
        Ok(())
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        Ok(Content {
            signature: Signature::decode(reader).await?,
            source: PublicKey::decode(reader).await?,
            index_hash: Hash::decode(reader).await?,
            timestamp: Timestamp::decode(reader).await?,
            magnet_link: Magnet::decode(reader).await?,
            entries: Vec::decode(reader).await?,
        })
    }
}
