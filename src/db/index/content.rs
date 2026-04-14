use std::marker::PhantomData;

use surrealdb_types::SurrealValue;

use crate::{
    db::{Magnet, ToBytes, index::tags::IndexTag},
    helpers::Byteable,
    types::{Hash, PrivateKey, PublicKey, Signature, Timestamp},
};

// ==================== End Imports ====================

pub trait ContentType<I: IndexTag>: PartialEq + Eq + 'static {
    type SourceType: std::fmt::Debug + Clone + SurrealValue + Byteable + ToBytes + PartialEq;
}

#[derive(Debug, Clone, SurrealValue, byteable_derive::Byteable, PartialEq, Eq)]
pub struct InternalContent;
#[derive(Debug, Clone, SurrealValue, byteable_derive::Byteable, PartialEq, Eq)]
pub struct ExternalContent;

impl<I: IndexTag> ContentType<I> for InternalContent {
    type SourceType = String;
}

impl<I: IndexTag> ContentType<I> for ExternalContent {
    type SourceType = I::ExternalSourceType;
}

#[derive(Debug, Clone, SurrealValue, byteable_derive::Byteable)]
pub struct Content<T: IndexTag, S: ContentType<T> = InternalContent> {
    #[surreal(rename = "id")]
    signature: Signature,
    poster: PublicKey,

    // Signed Fields
    index_hash: Hash,
    pub timestamp: Timestamp,

    // Only downloads the path from torrent
    pub magnet_link: Magnet,
    pub source: S::SourceType,

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
impl<I: IndexTag, S: ContentType<I>> PartialEq for Content<I, S> {
    fn eq(&self, other: &Self) -> bool {
        self.signature() == other.signature()
    }
}

impl<I: IndexTag> std::hash::Hash for Content<I> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.signature.hash(state)
    }
}

impl<T: IndexTag, S: ContentType<T>> Content<T, S> {
    pub fn new(
        signature: Signature,
        poster: PublicKey,
        index_hash: Hash,
        timestamp: Timestamp,
        magnet_link: Magnet,
        source: S::SourceType,
        title: String,
        enumeration: f32,
        end: Option<f32>,
        extra_metadata: T::ExtraMetadata,
    ) -> Self {
        Self {
            signature,
            poster,
            index_hash,
            timestamp,
            magnet_link,
            source,
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
        source: &S::SourceType,
        title: &str,
        enumeration: f32,
        end: Option<f32>,
        extra_metadata: &T::ExtraMetadata,
    ) -> Vec<u8> {
        let mut bytes: Vec<u8> = index_hash.inner().to_vec().to_vec();
        bytes.extend(timestamp.to_bytes());
        bytes.extend(magnet_link.0.as_bytes());
        bytes.extend(source.to_bytes());
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
        source: S::SourceType,
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
            &source,
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
            source,
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
            &self.source,
            &self.title,
            self.enumeration,
            self.end,
            &self.extra_metadata,
        );
        self.poster.verify(&to_verify, &self.signature)
    }

    pub fn source(&self) -> &S::SourceType {
        &self.source
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

    pub fn calculate_progress(&self) -> f32 {
        self.progress as f32 / self.count as f32 * 100.0
    }
}
