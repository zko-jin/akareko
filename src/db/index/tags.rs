use std::fmt::Debug;
use surrealdb_types::SurrealValue;

use crate::{
    db::ToBytes,
    helpers::{Byteable, Language},
};

// ==================== End Imports ====================

pub trait IndexTag: Send + Clone + Debug {
    const TAG: &'static str; // Acts like table name
    const CONTENT_TABLE: &'static str;
    type Content: Send + Clone + Debug + ToBytes + Byteable + SurrealValue;
}

// ==============================================================================
//                                 MangaTag
// ==============================================================================
#[derive(Debug, Clone)]
pub struct MangaTag;

impl IndexTag for MangaTag {
    const TAG: &'static str = "mangas";
    const CONTENT_TABLE: &'static str = "manga_chapters";
    type Content = MangaChapter;
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
#[derive(Debug, Clone)]
pub struct NoTag;
impl IndexTag for NoTag {
    const TAG: &'static str = "";

    const CONTENT_TABLE: &'static str = "";

    type Content = ();
}
