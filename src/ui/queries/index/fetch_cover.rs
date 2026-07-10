use std::{cell::RefCell, rc::Rc};

use freya::{elements::image::ImageHolder, query::QueryCapability};

use crate::{config::MetadataSource, db::index::IndexLinks, ui::AppResources};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FetchCover;
impl QueryCapability for FetchCover {
    type Ok = ImageHolder;

    type Err = mangadex_api::error::Error;

    type Keys = IndexLinks;

    async fn run(&self, keys: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        match AppResources::get_config()
            .unwrap_ref()
            .metadata_source
            .clone()
        {
            MetadataSource::LocalOnly => todo!(),
            MetadataSource::Mangadex => {
                let Some(uuid) = keys.mangadex else { todo!() };

                let client = mangadex_api::v5::MangaDexClient::default();

                let (_, bytes) = client
                    .download()
                    .cover()
                    .build()?
                    .via_manga_id(uuid)
                    .await?;

                let bytes = bytes?;

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
