use std::{cell::RefCell, marker::PhantomData, path::PathBuf, rc::Rc};

use async_zip::tokio::read::seek::ZipFileReader;
use freya::{
    elements::image::{ImageHolder, image},
    prelude::*,
    query::{Mutation, use_mutation},
    radio::use_radio,
};
use futures::AsyncReadExt as _;
use mangadex_api::utils::download::chapter::DownloadMode;
use tokio::{fs::File, io::BufReader};
use tracing::error;

use crate::{
    config::ImageVisualizationType,
    db::index::{
        content::{Content, ContentType, ExternalContent, InternalContent},
        tags::{ChapterExternalSource, IndexTag, MangaTag},
    },
    ui::{
        AppChannel, ResourceState,
        components::AkLayers,
        queries::{UpdateContentCount, UpdateContentProgress},
    },
};

#[derive(PartialEq)]
pub struct ChapterViewer<S: ContentType<MangaTag> + ImageLoaderExt<S>> {
    pub content: Content<MangaTag, S>,
}
impl<S: ContentType<MangaTag> + ImageLoaderExt<S>> Component for ChapterViewer<S> {
    fn render(&self) -> impl IntoElement {
        let images = use_state(Vec::<Option<ImageHolder>>::new);
        let mut cur_page_index = use_state(|| {
            if self.content.progress == 0 || self.content.progress == self.content.count {
                0
            } else {
                self.content.progress - 1
            }
        });
        let mut show_sidebar = use_state(|| true);

        let count_mutation = use_mutation(Mutation::new(UpdateContentCount::<MangaTag>::new()));
        let progress_mutation =
            use_mutation(Mutation::new(UpdateContentProgress::<MangaTag>::new()));

        S::start_loader(&self.content, images);

        let mut config = use_radio(AppChannel::Config);

        let mut scroll_controller = use_scroll_controller(ScrollConfig::default);

        let signature = self.content.signature().clone();
        use_side_effect(move || {
            count_mutation.mutate((signature.clone(), images.read().len() as u32));
        });

        let signature = self.content.signature().clone();
        let prog = self.content.progress;
        use_side_effect(move || {
            let cur_page = cur_page_index() + 1;
            if cur_page > prog {
                progress_mutation.mutate((signature.clone(), cur_page));
            };
        });

        let mut back_page = move || {
            let mut cur_page = cur_page_index.write();
            if *cur_page > 0 {
                *cur_page -= 1;
                scroll_controller.scroll_to(ScrollPosition::Start, Direction::Vertical);
            }
        };
        let mut forward_page = move || {
            let mut cur_page = cur_page_index.write();
            let total_pages: u32 = images.read().len() as u32;
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
                    match config
                        .read()
                        .config
                        .unwrap_ref()
                        .image_viewer_preferences()
                        .visualization_type
                    {
                        ImageVisualizationType::LeftToRight => back_page(),
                        ImageVisualizationType::RightToLeft => forward_page(),
                        ImageVisualizationType::Scroll => todo!(),
                    }
                }
                Code::ArrowRight => {
                    e.stop_propagation();
                    match config
                        .read()
                        .config
                        .unwrap_ref()
                        .image_viewer_preferences()
                        .visualization_type
                    {
                        ImageVisualizationType::LeftToRight => forward_page(),
                        ImageVisualizationType::RightToLeft => back_page(),
                        ImageVisualizationType::Scroll => todo!(),
                    }
                }
                Code::ArrowDown => {
                    // e.stop_propagation();
                    // let (_, y) = scroll_controller.into();
                    // scroll_controller.scroll_to_y(y - 50);
                }
                Code::ArrowUp => {
                    // e.stop_propagation();
                    // let (_, y) = scroll_controller.into();
                    // scroll_controller.scroll_to_y(y + 50);
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
            .child(match images.read().get(*cur_page_index.read() as usize) {
                Some(Some(img)) => image(img.clone())
                    .height(Size::px(img.image.borrow().height() as f32 * zoom))
                    .into_element(),
                _ => CircularLoader::new().into_element(),
            });

        let page_counter = label()
            .width(Size::Fill)
            .text(format!(
                "{}/{}",
                *cur_page_index.read() + 1,
                images.read().len()
            ))
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
            })
            .a11y_focusable(Focusable::Disabled);

        let click_area_center = rect()
            .height(Size::Fill)
            .width(Size::flex(1.0))
            .on_mouse_down(move |e: Event<MouseEventData>| {
                if Some(MouseButton::Left) == e.button {
                    e.stop_propagation();
                    let mut show_sidebar = show_sidebar.write();
                    *show_sidebar = !*show_sidebar;
                };
            })
            .a11y_focusable(Focusable::Disabled);

        let click_area_right = rect()
            .height(Size::Fill)
            .width(Size::flex(3.0))
            .on_mouse_down(move |e: Event<MouseEventData>| {
                if Some(MouseButton::Left) == e.button {
                    e.stop_propagation();
                    forward_page();
                };
            })
            .a11y_focusable(Focusable::Disabled);

        let click_areas = rect()
            .height(Size::percent(100.))
            .width(Size::percent(100.))
            .horizontal()
            .content(freya::prelude::Content::Flex)
            .position(Position::new_absolute())
            .child(click_area_left)
            .child(click_area_center)
            .child(click_area_right)
            .a11y_focusable(Focusable::Disabled);

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

trait ImageLoaderExt<S: ContentType<MangaTag>> {
    fn start_loader(
        content: &Content<MangaTag, S>,
        images: State<Vec<Option<ImageHolder>>>,
    ) -> TaskHandle;
}

impl ImageLoaderExt<InternalContent> for InternalContent {
    fn start_loader(
        content: &Content<MangaTag, InternalContent>,
        mut images: State<Vec<Option<ImageHolder>>>,
    ) -> TaskHandle {
        let chapter_loader = use_hook(move || {
            let source: PathBuf = format!(
                "./data{}/{}/{}",
                MangaTag::TAG,
                content.signature(),
                content.source()
            )
            .into();

            spawn(async move {
                if !source.exists() {
                    error!("Path does not exist");
                    return;
                }

                if source.is_dir() {
                    let mut dir = tokio::fs::read_dir(&source).await.unwrap();
                    let mut paths = Vec::new();
                    while let Ok(entry) = dir.next_entry().await {
                        let entry = entry.unwrap();
                        if entry.file_type().await.unwrap().is_file() {
                            paths.push(entry.path());
                        }
                    }

                    *images.write() = vec![None; paths.len()];

                    for (i, source) in paths.iter().enumerate() {
                        let bytes: Bytes = tokio::fs::read(source).await.unwrap().into();
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
                    return;
                }

                if let Some(extension) = source.extension() {
                    if extension == "cbz" {
                        let mut file = BufReader::new(File::open(source).await.unwrap());
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
}

impl ImageLoaderExt<ExternalContent> for ExternalContent {
    fn start_loader(
        content: &Content<MangaTag, ExternalContent>,
        mut images: State<Vec<Option<ImageHolder>>>,
    ) -> TaskHandle {
        let source = content.source().clone();
        let chapter_loader = use_hook(move || {
            let source = source.clone();
            spawn(async move {
                match source {
                    ChapterExternalSource::MangaDex(uuid) => {
                        let client = mangadex_api::v5::MangaDexClient::default();
                        let res = client
                            .download()
                            .chapter(uuid)
                            .mode(DownloadMode::Normal)
                            .build()
                            .unwrap();

                        let file_names = res.build_at_home_urls().await.unwrap();
                        *images.write() = vec![None; file_names.len()];

                        for (i, filename) in file_names.iter().enumerate() {
                            let (_, bytes) = filename.download().await;
                            let bytes = bytes.unwrap();

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
                }
            })
        });

        use_drop(move || {
            chapter_loader.try_cancel();
        });

        chapter_loader
    }
}
