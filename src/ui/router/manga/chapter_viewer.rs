use std::{cell::RefCell, path::PathBuf, rc::Rc};

use freya::{
    elements::image::{ImageData, ImageHolder, image},
    prelude::*,
    query::*,
    radio::use_radio,
    router::RouterContext,
};
use image::EncodableLayout;
use tracing::error;

use crate::{
    db::index::{content::Content, tags::MangaTag},
    types::Signature,
};

#[derive(PartialEq)]
pub struct ChapterViewer {
    pub content: Content<MangaTag>,
}
impl Component for ChapterViewer {
    fn render(&self) -> impl IntoElement {
        let mut images = use_state(Vec::<Option<ImageHolder>>::new);
        let mut cur_page = use_state(|| 0);

        let path = PathBuf::from(self.content.path());
        let chapter_loader = use_hook(move || {
            spawn(async move {
                // if !path.exists() {
                //     return Err("Path does not exist");
                // }

                if path.is_dir() {
                    let mut dir = tokio::fs::read_dir(path).await.unwrap();
                    let mut paths = Vec::new();
                    while let Ok(entry) = dir.next_entry().await {
                        let entry = entry.unwrap();
                        if entry.file_type().await.unwrap().is_file() {
                            paths.push(entry.path());
                        }
                    }

                    *images.write() = vec![None; paths.len()];

                    for (i, path) in paths.iter().enumerate() {
                        let bytes: Bytes = tokio::fs::read(path).await.unwrap().into();
                        let (image, bytes) = blocking::unblock(move || {
                            let image = skia_safe::Image::from_encoded(unsafe {
                                skia_safe::Data::new_bytes(&bytes)
                            })
                            .unwrap();
                            (image, bytes)
                        })
                        .await;

                        images.write()[i] = Some(ImageHolder {
                            image: Rc::new(RefCell::new(image)),
                            bytes,
                        });
                    }
                }

                //     if let Some(extension) = path.extension() {
                //         if extension == "cbz" {
                //             let mut file =
                // BufReader::new(File::open(path).await.unwrap());
                //             let mut zip = ZipFileReader::with_tokio(&mut
                // file).await.unwrap();

                //             // TODO: Check how many actual images and ignore
                // other files             let total_images =
                // zip.file().entries().len();

                //             match output
                //                 .send(ImageViewerMessage::PreloadImages {
                // total_images }.into())                 .await
                //             {
                //                 Ok(()) => {}
                //                 Err(e) => {
                //                     error!("Error preloading images: {}", e);
                //                 }
                //             }

                //             // Add priority system so files near the current
                // page are loaded first             for i in
                // 0..total_images {                 let mut f =
                // zip.reader_with_entry(i).await.unwrap();
                // let mut buffer = vec![];
                // f.read_to_end(&mut buffer).await.unwrap();
                //                 let image = match
                // image::load_from_memory(&buffer) {
                // Ok(image) => image.to_rgba8(),
                // Err(e) => {                         error!(
                //                             "Error loading image {}: {}",
                //
                // f.entry().filename().as_str().unwrap(),
                // e                         );
                //                         let _ = output
                //
                // .send(Message::PostToast(Toast::error(
                // "Could not load image",
                // format!("Error loading image: {}", e),
                // )))                             .await;

                //                         continue;
                //                     }
                //                 };
                //                 let (width, height) = image.dimensions();

                //                 match output
                //                     .send(
                //                         ImageViewerMessage::LoadedImage {
                //                             handle: Handle::from_rgba(
                //                                 width,
                //                                 height,
                //                                 image.into_raw(),
                //                             ),
                //                             height,
                //                             index: i,
                //                         }
                //                         .into(),
                //                     )
                //                     .await
                //                 {
                //                     Ok(()) => {}
                //                     Err(e) => {
                //                         error!("Error loading image: {}", e);
                //                     }
                //                 }
                //             }
                //         }
                //     }
                // },
                // )
            });
        });

        let image_viewer = match images.read().get(*cur_page.read()) {
            Some(Some(img)) => image(img.clone()).into_element(),
            _ => CircularLoader::new().into_element(),
        };

        rect().child(image_viewer)
    }
}
