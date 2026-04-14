use std::{cell::RefCell, rc::Rc};

use freya::{
    elements::image::ImageHolder, prelude::try_consume_root_context, query::QueryCapability,
    radio::RadioStation,
};
use mangadex_api::utils::download::chapter::DownloadMode;
use mangadex_api_types_rust::{IncludeExternalUrl, IncludeUnvailable};
use uuid::Uuid;

use crate::{
    config::MetadataSource,
    db::{
        Magnet,
        index::{
            IndexLinks,
            content::{Content, ExternalContent},
            tags::{ChapterExternalSource, MangaChapter, MangaTag},
        },
    },
    helpers::Language,
    types::{Hash, PublicKey, Signature, Timestamp},
    ui::{AppChannel, AppState},
};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FetchMangadexChapters;
impl QueryCapability for FetchMangadexChapters {
    type Ok = Vec<Content<MangaTag, ExternalContent>>;

    type Err = mangadex_api::error::Error;

    type Keys = Uuid;

    async fn run(&self, keys: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        // TODO: Check if it exists in local storage
        let client = mangadex_api::v5::MangaDexClient::default();

        let res = client
            .chapter()
            .get()
            .manga_id(keys.clone())
            .translated_languages(vec!["pt-br".into()])
            .include_unavailable(IncludeUnvailable::Exclude)
            .include_external_url(IncludeExternalUrl::Exclude)
            .offset(0u32)
            .limit(50u32)
            .build()?
            .send()
            .await?;

        let mut chapters = Vec::with_capacity(res.data.len());
        for c in res.data {
            chapters.push(Content::new(
                Signature::empty(),
                unsafe { PublicKey::from_bytes_unchecked([0; 32]) },
                Hash::new([0; 64]),
                Timestamp::new(0),
                Magnet(String::new()),
                ChapterExternalSource::MangaDex(c.id),
                c.attributes.title.unwrap_or_else(String::new),
                if let Some(num) = c.attributes.chapter {
                    num.parse().unwrap_or(0.)
                } else {
                    0.
                },
                None,
                MangaChapter::new(Language::English),
            ));
        }

        chapters.sort_by(|c, o| c.enumeration().total_cmp(&o.enumeration()));

        Ok(chapters)
    }
}
