use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod cover;

pub const BASE_URL: &'static str = "https://api.mangadex.org";
pub const UPLOADS_BASE_URL: &'static str = "https://uploads.mangadex.org";

pub struct MangadexClient;
impl MangadexClient {
    pub async fn manga(id: &str) -> Result<MangaData, reqwest::Error> {
        let res = reqwest::get(format!("{}/manga/{}", BASE_URL, id)).await?;

        let MangaResponse { data, .. } = res.json()?;

        Ok(data)
    }

    pub async fn cover(code: &str) {
        let res = reqwest::get(format!("{}/covers/{}", UPLOADS_BASE_URL, code)).await;
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MangaResponse {
    pub result: String,
    pub response: String,
    pub data: MangaData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MangaData {
    pub id: String,
    #[serde(rename = "type")]
    pub data_type: String,
    pub attributes: MangaAttributes,
    pub relationships: Vec<Relationship>,
}

impl MangaData {
    pub async fn cover(&self) -> Option<()> {
        let Some(relation) = self
            .relationships
            .iter()
            .find(|r| r.rel_type == RelationType::CoverArt)
        else {
            return None;
        };

        Some(MangadexClient::cover(&relation.id))
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MangaAttributes {
    // Uses HashMap because keys like "ja-ro" aren't valid Rust identifiers
    pub title: HashMap<String, String>,
    pub alt_titles: Vec<HashMap<String, String>>,
    pub description: HashMap<String, String>,
    pub is_locked: bool,
    pub links: Option<HashMap<String, String>>,
    pub official_links: Option<HashMap<String, String>>,
    pub original_language: String,
    pub last_volume: Option<String>,
    pub last_chapter: Option<String>,
    pub publication_demographic: Option<String>,
    pub status: String,
    pub year: Option<i32>,
    pub content_rating: String,
    pub tags: Vec<Tag>,
    pub state: String,
    pub created_at: String,
    pub updated_at: String,
    pub version: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    #[serde(rename = "type")]
    pub tag_type: String,
    pub attributes: TagAttributes,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TagAttributes {
    pub name: HashMap<String, String>,
    pub group: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    Author,
    Artist,
    CoverArt,
    Creator,
    // This attribute tells Serde to map any unrecognized strings to this variant
    // instead of failing the whole deserialization.
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Relationship {
    pub id: String,
    #[serde(rename = "type")]
    pub rel_type: RelationType,
}
