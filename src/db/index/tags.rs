use std::{fmt::Debug, hash::Hash};
use surrealdb_types::SurrealValue;

use crate::{
    db::{ToBytes, event::EventType},
    helpers::{Byteable, Language},
};

// ==================== End Imports ====================

pub trait IndexTag: Send + Clone + Debug + PartialEq + Eq + Hash + 'static {
    const TAG: &'static str; // Acts like table name
    const CONTENT_TABLE: &'static str;
    type ExtraMetadata: Send + Clone + Debug + ToBytes + Byteable + SurrealValue;

    const EVENT_TYPE: EventType;
    const CONTENT_EVENT_TYPE: EventType;
}

// ==============================================================================
//                                 MangaTag
// ==============================================================================
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MangaTag;

impl IndexTag for MangaTag {
    const TAG: &'static str = "mangas";
    const CONTENT_TABLE: &'static str = "manga_chapters";
    type ExtraMetadata = MangaChapter;

    const EVENT_TYPE: EventType = EventType::Manga;
    const CONTENT_EVENT_TYPE: EventType = EventType::MangaContent;
}

// ==================== Manga Chapter ====================
#[derive(Debug, Clone, SurrealValue, byteable_derive::Byteable)]
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

    type ExtraMetadata = ();

    const EVENT_TYPE: EventType = EventType::Invalid;
    const CONTENT_EVENT_TYPE: EventType = EventType::Invalid;
}
