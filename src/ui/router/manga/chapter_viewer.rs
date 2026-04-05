use std::{cell::RefCell, ops::Deref, path::PathBuf, rc::Rc};

use async_zip::tokio::read::seek::ZipFileReader;
use freya::{
    animation::{use_animation, use_animation_transition},
    elements::image::{ImageData, ImageHolder, image},
    prelude::*,
    query::*,
    radio::use_radio,
    router::RouterContext,
};
use futures::AsyncReadExt as _;
use image::EncodableLayout;
use tokio::{fs::File, io::BufReader};
use tracing::error;

use crate::{
    db::index::{content::Content, tags::MangaTag},
    types::Signature,
    ui::{AppChannel, ResourceState, components::AkLayers},
};

#[derive(PartialEq)]
pub struct ChapterViewer {
    pub content: Content<MangaTag>,
}
impl Component for ChapterViewer {
    fn render(&self) -> impl IntoElement {
        let images = use_state(Vec::<Option<ImageHolder>>::new);
        let mut cur_page = use_state(|| 0);
        let mut show_sidebar = use_state(|| true);

        let path = PathBuf::from(format!(
            "./data/mangas/{}/{}",
            self.content.signature().as_base64(),
            self.content.path()
        ));
        image_loader(path, images);
        let mut config = use_radio(AppChannel::Config);

        let mut scroll_controller = use_scroll_controller(ScrollConfig::default);

        let mut back_page = move || {
            let mut cur_page = cur_page.write();
            if *cur_page > 0 {
                *cur_page -= 1;
                scroll_controller.scroll_to(ScrollPosition::Start, Direction::Vertical);
            }
        };
        let mut forward_page = move || {
            let mut cur_page = cur_page.write();
            let total_pages: usize = images.read().len();
            if *cur_page + 1 < total_pages {
                *cur_page += 1;
                scroll_controller.scroll_to(ScrollPosition::Start, Direction::Vertical);
            }
        };

        let on_key_down = move |e: Event<KeyboardEventData>| {
            match &e.code {
                // On submit
                Code::ArrowLeft => {
                    e.stop_propagation();
                    back_page();
                }
                Code::ArrowRight => {
                    e.stop_propagation();
                    forward_page();
                }
                Code::ArrowDown => {
                    // e.stop_propagation();
                }
                Code::ArrowUp => {
                    // e.stop_propagation();
                }
                Code::PageUp => {
                    e.stop_propagation();
                    scroll_controller.scroll_to(ScrollPosition::Start, Direction::Vertical);
                }
                Code::PageDown => {
                    e.stop_propagation();
                    scroll_controller.scroll_to(ScrollPosition::End, Direction::Vertical);
                }
                Code::Equal if e.modifiers.ctrl() => {
                    e.stop_propagation();
                    match &mut config.write().config {
                        ResourceState::Loaded(c) => {
                            c.set_zoom(c.zoom() + 5);
                        }
                        _ => {}
                    }
                }
                Code::Minus if e.modifiers.ctrl() => {
                    e.stop_propagation();
                    match &mut config.write().config {
                        ResourceState::Loaded(c) => {
                            c.set_zoom(c.zoom() - 5);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        };

        let zoom = config
            .read()
            .config
            .unwrap_ref()
            .image_viewer_preferences()
            .zoom();

        let image_viewer = rect()
            .center()
            .horizontal()
            .min_height(Size::Fill)
            .width(Size::Fill)
            .child(match images.read().get(*cur_page.read()) {
                Some(Some(img)) => image(img.clone())
                    .height(Size::px(img.image.borrow().height() as f32 * zoom))
                    .into_element(),
                _ => CircularLoader::new().into_element(),
            });

        let page_counter = label()
            .width(Size::Fill)
            .text(format!("{}/{}", *cur_page.read() + 1, images.read().len()))
            .text_align(TextAlign::Center)
            .font_size(21);

        let right_side_bar = rect()
            .layer(AkLayers::Sidebars)
            .width(Size::px(200.0))
            .height(Size::percent(100.0))
            .position(Position::new_absolute().right(0.0))
            .background(Color::GRAY)
            .child(page_counter)
            .on_mouse_down(|e: Event<MouseEventData>| {
                e.stop_propagation();
            });

        let click_area_left = rect()
            .height(Size::Fill)
            .width(Size::flex(3.0))
            .on_mouse_down(move |e: Event<MouseEventData>| {
                if Some(MouseButton::Left) == e.button {
                    e.stop_propagation();
                    back_page();
                };
            });

        let click_area_center = rect()
            .height(Size::Fill)
            .width(Size::flex(1.0))
            .on_mouse_down(move |e: Event<MouseEventData>| {
                if Some(MouseButton::Left) == e.button {
                    e.stop_propagation();
                    let mut show_sidebar = show_sidebar.write();
                    *show_sidebar = !*show_sidebar;
                };
            });

        let click_area_right = rect()
            .height(Size::Fill)
            .width(Size::flex(3.0))
            .on_mouse_down(move |e: Event<MouseEventData>| {
                if Some(MouseButton::Left) == e.button {
                    e.stop_propagation();
                    forward_page();
                };
            });

        let click_areas = rect()
            .height(Size::percent(100.))
            .width(Size::percent(100.))
            .horizontal()
            .content(freya::prelude::Content::Flex)
            .position(Position::new_absolute())
            .child(click_area_left)
            .child(click_area_center)
            .child(click_area_right);

        rect()
            .width(Size::Fill)
            .height(Size::Fill)
            .content(freya::prelude::Content::Flex)
            .child(
                ScrollView::new_controlled(scroll_controller)
                    .child(image_viewer)
                    .show_scrollbar(false),
            )
            .child(right_side_bar)
            .child(click_areas)
            .on_global_key_down(on_key_down)
    }
}

fn image_loader(path: PathBuf, mut images: State<Vec<Option<ImageHolder>>>) -> TaskHandle {
    let chapter_loader = use_hook(move || {
        spawn(async move {
            if !path.exists() {
                error!("Path does not exist");
                return;
            }

            if path.is_dir() {
                let mut dir = tokio::fs::read_dir(&path).await.unwrap();
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

            if let Some(extension) = path.extension() {
                if extension == "cbz" {
                    let mut file = BufReader::new(File::open(path).await.unwrap());
                    let mut zip = ZipFileReader::with_tokio(&mut file).await.unwrap();

                    // TODO: Check how many actual images and ignore other files
                    let total_images = zip.file().entries().len();

                    *images.write() = vec![None; total_images];

                    // Add priority system so files near the current
                    // page are loaded first
                    for i in 0..total_images {
                        let mut f = zip.reader_with_entry(i).await.unwrap();
                        let mut buffer = vec![];
                        f.read_to_end(&mut buffer).await.unwrap();
                        let (image, bytes) = blocking::unblock(move || {
                            let image = skia_safe::Image::from_encoded(unsafe {
                                skia_safe::Data::new_bytes(&buffer)
                            })
                            .unwrap();
                            (image, buffer.into())
                        })
                        .await;

                        images.write()[i] = Some(ImageHolder {
                            image: Rc::new(RefCell::new(image)),
                            bytes,
                        });
                    }
                }
            }
        })
    });

    use_drop(move || {
        chapter_loader.try_cancel();
    });

    chapter_loader
}
