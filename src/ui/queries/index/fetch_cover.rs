use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use freya::{elements::image::ImageHolder, query::QueryCapability};
use skia_safe::svg::fe::Fe;
use tracing::warn;

use crate::{
    config::MetadataSource,
    daemon::resource_state::ResourceState,
    db::index::{Index, IndexLinks, tags::IndexTag},
    ui::{AppResources, UNKNOWN_COVER},
};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FetchCover<T: IndexTag>(PhantomData<T>);
impl<T: IndexTag> FetchCover<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> QueryCapability for FetchCover<T>
where
    T: IndexTag,
{
    type Ok = ImageHolder;

    type Err = mangadex_api::error::Error;

    type Keys = Index<T>;

    async fn run(&self, keys: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        let hash_str = &keys.hash().as_base64();
        let repo = AppResources::get_repositories();
        if let ResourceState::Loaded(repo) = &repo {
            if let Ok(cover) = repo.retrieve_cover(&hash_str).await {
                let (image, bytes) = blocking::unblock(move || {
                    let image = skia_safe::Image::from_encoded(unsafe {
                        skia_safe::Data::new_bytes(&cover)
                    })
                    .unwrap();
                    (image, cover)
                })
                .await;

                return Ok(ImageHolder {
                    image: Rc::new(RefCell::new(image)),
                    bytes,
                });
            }
        }

        match AppResources::get_config()
            .unwrap_ref()
            .metadata_source
            .clone()
        {
            MetadataSource::LocalOnly => todo!(),
            MetadataSource::Mangadex => {
                let Some(uuid) = keys.out_links().mangadex else {
                    todo!()
                };

                let client = mangadex_api::v5::MangaDexClient::default();

                let (_, bytes) = client
                    .download()
                    .cover()
                    .build()?
                    .via_manga_id(uuid)
                    .await?;

                let bytes = bytes?;

                if let ResourceState::Loaded(repo) = &repo {
                    // We don't really care if this fails
                    if let Err(e) = repo.save_cover(&hash_str, &bytes).await {
                        warn!("Failed to save cover: {:?}", e);
                    }
                }

                let (image, bytes) = blocking::unblock(move || {
                    let image = skia_safe::Image::from_encoded(unsafe {
                        skia_safe::Data::new_bytes(&bytes)
                    })
                    .unwrap();
                    (image, bytes)
                })
                .await;

                Ok(ImageHolder {
                    image: Rc::new(RefCell::new(image)),
                    bytes,
                })
            }
        }
    }
}
