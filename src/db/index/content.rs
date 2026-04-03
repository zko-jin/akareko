use surrealdb_core::api::path;
use surrealdb_types::SurrealValue;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    db::{Magnet, ToBytes, index::tags::IndexTag},
    errors::{DecodeError, EncodeError},
    helpers::Byteable,
    types::{Hash, PrivateKey, PublicKey, Signature, Timestamp},
};

// ==================== End Imports ====================

#[derive(Debug, Clone, SurrealValue, byteable_derive::Byteable)]
pub struct Content<T: IndexTag> {
    #[surreal(rename = "id")]
    signature: Signature,
    source: PublicKey,

    // Signed Fields
    index_hash: Hash,
    pub timestamp: Timestamp,

    // Only downloads the path from torrent
    pub magnet_link: Magnet,
    pub path: String,

    pub title: String,

    pub enumeration: f32,
    /// If this entry covers multiple enumerations (entire volumes), set this to
    /// the last one.
    pub end: Option<f32>,

    pub extra_metadata: T::ExtraMetadata,

    /// Each tag will use this differently, videos will count seconds, comics
    /// will count pages, etc.
    /// If count is 0 any progress above 0 will be considered as fully seen.
    #[byteable(skip)]
    pub progress: u32,
    /// Max progress, by default it's set to 1 and will be update whenever you
    /// open the content.
    #[byteable(skip)]
    pub count: u32,
}

impl<I: IndexTag> PartialEq for Content<I> {
    fn eq(&self, other: &Self) -> bool {
        self.signature() == other.signature()
    }
}

impl<I: IndexTag> std::hash::Hash for Content<I> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.signature.hash(state)
    }
}

impl<T: IndexTag> Content<T> {
    pub fn new(
        signature: Signature,
        source: PublicKey,
        index_hash: Hash,
        timestamp: Timestamp,
        magnet_link: Magnet,
        path: String,
        title: String,
        enumeration: f32,
        end: Option<f32>,
        extra_metadata: T::ExtraMetadata,
    ) -> Self {
        Self {
            signature,
            source,
            index_hash,
            timestamp,
            magnet_link,
            path,
            title,
            enumeration,
            end,
            extra_metadata,
            progress: 0,
            count: 1,
        }
    }

    pub fn id_bytes(
        index_hash: &Hash,
        timestamp: &Timestamp,
        magnet_link: &Magnet,
        path: &str,
        title: &str,
        enumeration: f32,
        end: Option<f32>,
        extra_metadata: &T::ExtraMetadata,
    ) -> Vec<u8> {
        let mut bytes: Vec<u8> = index_hash.inner().to_vec().to_vec();
        bytes.extend(timestamp.to_bytes());
        bytes.extend(magnet_link.0.as_bytes());
        bytes.extend(path.as_bytes());
        bytes.extend(title.as_bytes());
        bytes.extend(enumeration.to_le_bytes());
        if let Some(end) = end {
            bytes.extend(end.to_le_bytes());
        }
        bytes.extend(extra_metadata.to_bytes());
        bytes
    }

    pub fn new_signed(
        index_hash: Hash,
        timestamp: Timestamp,
        magnet_link: Magnet,
        path: String,
        title: String,
        enumeration: f32,
        end: Option<f32>,
        extra_metadata: T::ExtraMetadata,
        priv_key: &PrivateKey,
    ) -> Self {
        let to_sign = Self::id_bytes(
            &index_hash,
            &timestamp,
            &magnet_link,
            &path,
            &title,
            enumeration,
            end,
            &extra_metadata,
        );
        let signature = priv_key.sign(&to_sign);

        Self::new(
            signature,
            priv_key.public_key(),
            index_hash,
            timestamp,
            magnet_link,
            path,
            title,
            enumeration,
            end,
            extra_metadata,
        )
    }

    pub fn verify(&self) -> bool {
        let to_verify = Self::id_bytes(
            &self.index_hash,
            &self.timestamp,
            &self.magnet_link,
            &self.path,
            &self.title,
            self.enumeration,
            self.end,
            &self.extra_metadata,
        );
        self.source.verify(&to_verify, &self.signature)
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn enumeration(&self) -> f32 {
        self.enumeration
    }

    pub fn end(&self) -> Option<f32> {
        self.end
    }

    pub fn extra_metadata(&self) -> &T::ExtraMetadata {
        &self.extra_metadata
    }

    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    pub fn update_progress(&mut self, progress: u32) {
        self.progress = progress;
    }

    pub fn index_hash(&self) -> &Hash {
        &self.index_hash
    }
}
