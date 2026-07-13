use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{fmt::Debug, hash::Hash};
use surrealdb_types::{File, SurrealValue};
use uuid::Uuid;

use crate::{
    db::{ToBytes, event::EventType},
    helpers::Language,
};

// ==================== End Imports ====================

pub trait IndexTag: Send + Clone + Debug + PartialEq + Eq + Hash + 'static {
    const TAG: &'static str; // Acts like table name
    const CONTENT_TABLE: &'static str;
    type ExtraContentMetadata: Send
        + Clone
        + Debug
        + ToBytes
        + Serialize
        + DeserializeOwned
        + SurrealValue;
    type ExternalSourceType: Debug
        + Clone
        + SurrealValue
        + Serialize
        + DeserializeOwned
        + ToBytes
        + PartialEq;

    const EVENT_TYPE: EventType;
    const CONTENT_EVENT_TYPE: EventType;
}

// ==============================================================================
//                                 MangaTag
// ==============================================================================
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MangaTag;

#[derive(Debug, PartialEq, Clone, SurrealValue, Serialize, Deserialize)]
pub enum ChapterExternalSource {
    MangaDex(Uuid),
}

impl ToBytes for ChapterExternalSource {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::MangaDex(u) => u.as_bytes().to_vec(),
        }
    }
}

impl IndexTag for MangaTag {
    const TAG: &'static str = "mangas";
    const CONTENT_TABLE: &'static str = "manga_chapters";
    type ExtraContentMetadata = MangaChapter;
    type ExternalSourceType = ChapterExternalSource;

    const EVENT_TYPE: EventType = EventType::Manga;
    const CONTENT_EVENT_TYPE: EventType = EventType::MangaContent;
}

// ==================== Manga Chapter ====================
#[derive(Debug, Clone, SurrealValue, Serialize, Deserialize)]
pub struct MangaChapter {
    pub language: Language,
}

impl MangaChapter {
    pub fn new(language: Language) -> MangaChapter {
        MangaChapter { language }
    }
}

impl ToBytes for MangaChapter {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend((self.language.clone() as u16).to_be_bytes());
        bytes
    }
}

// ==============================================================================
//                                    NoTag
// ==============================================================================
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NoTag;
impl IndexTag for NoTag {
    const TAG: &'static str = "";

    const CONTENT_TABLE: &'static str = "";

    type ExternalSourceType = ();
    type ExtraContentMetadata = ();

    const EVENT_TYPE: EventType = EventType::Invalid;
    const CONTENT_EVENT_TYPE: EventType = EventType::Invalid;
}
