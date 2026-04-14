use surrealdb_types::SurrealValue;
use uuid::Uuid;

use crate::{
    db::{SurrealPhantom, ToBytes, index::tags::IndexTag},
    helpers::SanitizedString,
    types::{Hash, PrivateKey, PublicKey, Signature},
};

// ==================== End Imports ====================

pub mod content;
pub mod metadata;
pub mod tags;

#[cfg(feature = "sqlite")]
mod sqlite;
#[cfg(feature = "sqlite")]
pub use sqlite::IndexRepository;
#[cfg(feature = "surrealdb")]
mod surreal;
#[cfg(feature = "surrealdb")]
pub use surreal::IndexRepository;

#[derive(Debug, Clone, SurrealValue, byteable_derive::Byteable, PartialEq, Hash)]
pub struct IndexLinks {
    pub myanimelist: Option<String>,
    pub mangadex: Option<Uuid>,
}

impl ToBytes for IndexLinks {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        if let Some(s) = &self.myanimelist {
            bytes.extend_from_slice(s.as_bytes());
        }
        if let Some(s) = &self.mangadex {
            bytes.extend_from_slice(s.as_bytes());
        }

        bytes
    }
}

#[derive(Debug, Clone, byteable_derive::Byteable)]
#[cfg_attr(feature = "surrealdb", derive(SurrealValue))]
pub struct Index<T: IndexTag> {
    #[cfg_attr(feature = "surrealdb", surreal(rename = "id"))]
    hash: Hash, // Primary Key
    title: String,
    release_date: i32,
    source: PublicKey,
    signature: Signature,

    out_links: IndexLinks,

    #[byteable(skip)]
    _phantom: SurrealPhantom<T>,
}

impl<T: IndexTag> std::hash::Hash for Index<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl<T: IndexTag> PartialEq for Index<T> {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl<T: IndexTag> Index<T> {
    pub fn new(
        title: String,
        release_date: i32,
        out_links: IndexLinks,
        source: PublicKey,
        signature: Signature,
    ) -> Self {
        let hash = Hash::digest(&Self::id_bytes(&title, &release_date));

        Self {
            hash,
            title,
            release_date,
            out_links,
            source,
            signature,
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

    pub fn new_signed(
        title: String,
        release_date: i32,
        out_links: IndexLinks,
        priv_key: &PrivateKey,
    ) -> Self {
        let mut index = Self::new(
            title,
            release_date,
            out_links,
            priv_key.public_key(),
            Signature::empty(),
        );

        index.sign(priv_key);

        index
    }

    fn sign(&mut self, priv_key: &PrivateKey) {
        let mut to_sign = Self::id_bytes(&self.title, &self.release_date);
        to_sign.extend(self.out_links.to_bytes());
        self.signature = priv_key.sign(&to_sign);
    }

    pub fn verify(&self) -> bool {
        let mut to_verify = Self::id_bytes(&self.title, &self.release_date);
        to_verify.extend(self.out_links.to_bytes());
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

    pub fn out_links(&self) -> &IndexLinks {
        &self.out_links
    }

    pub fn source(&self) -> &PublicKey {
        &self.source
    }

    pub fn signature(&self) -> &Signature {
        &self.signature
    }
}
