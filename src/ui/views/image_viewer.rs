use std::path::PathBuf;

use async_zip::base::read::seek::ZipFileReader;
use futures::{AsyncReadExt, SinkExt};
use iced::{
    Element, Event, Length, Subscription, Task, event,
    keyboard::{
        self, Key,
        key::{Code, Named},
    },
    stream,
    widget::{
        self, Scrollable, Space, button, center, column, container,
        image::Handle,
        mouse_area,
        operation::{scroll_by, snap_to},
        row,
        scrollable::{self},
        stack, text,
    },
};
use tokio::{
    fs::{File, read_dir},
    io::BufReader,
};
use tracing::error;

use crate::ui::{
    AppState,
    components::toast::Toast,
    message::Message,
    views::{View, ViewMessage},
};

const SCROLLABLE: &str = "image_scrollable";

#[derive(Debug, Clone)]
pub struct ImageViewerView {
    file_path: PathBuf,
    images: Vec<Option<(Handle, u32)>>, // Height

    // Starts at 1 and go up to len, use -1 to get index
    cur_page: usize,
}

#[derive(Debug, Clone)]
pub enum ImageViewerMessage {
    PreloadImages {
        total_images: usize,
    },
    LoadedImage {
        handle: Handle,
        height: u32,
        index: usize,
    },
    ScrollBy(scrollable::AbsoluteOffset),
    PrevPage,
    NextPage,
    ZoomIn,
    ZoomOut,
}

impl From<ImageViewerMessage> for Message {
    fn from(m: ImageViewerMessage) -> Self {
        Message::ViewMessage(ViewMessage::ImageViewer(m))
    }
}

impl ImageViewerView {
    const SCROLL_OFFSET: f32 = 64.0;

    pub fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            images: vec![],
            cur_page: 1,
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            event::listen_with(|event, _, _| {
                if let Event::Keyboard(keyboard::Event::KeyPressed {
                    physical_key,
                    modifiers,
                    ..
                }) = event
                {
                    match physical_key {
                        keyboard::key::Physical::Code(Code::ArrowRight) => {
                            Some(ImageViewerMessage::NextPage.into())
                        }
                        keyboard::key::Physical::Code(Code::ArrowLeft) => {
                            Some(ImageViewerMessage::PrevPage.into())
                        }
                        keyboard::key::Physical::Code(Code::ArrowUp) => Some(
                            ImageViewerMessage::ScrollBy(scrollable::AbsoluteOffset {
                                x: 0.0,
                                y: -Self::SCROLL_OFFSET,
                            })
                            .into(),
                        ),
                        keyboard::key::Physical::Code(Code::ArrowDown) => Some(
                            ImageViewerMessage::ScrollBy(scrollable::AbsoluteOffset {
                                x: 0.0,
                                y: Self::SCROLL_OFFSET,
                            })
                            .into(),
                        ),
                        keyboard::key::Physical::Code(Code::Minus) => {
                            if modifiers.control() {
                                Some(ImageViewerMessage::ZoomOut.into())
                            } else {
                                None
                            }
                        }
                        keyboard::key::Physical::Code(Code::Equal) => {
                            if modifiers.control() {
                                Some(ImageViewerMessage::ZoomIn.into())
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }),
            Subscription::run_with(self.file_path.clone(), |path| {
                let path = path.clone();
                stream::channel(
                    8,
                    |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
                        if !path.exists() {
                            match output
                                .send(Message::PostToast(Toast::error(
                                    "Could not load chapter",
                                    "Path does not exist",
                                )))
                                .await
                            {
                                Ok(()) => {}
                                Err(e) => {
                                    error!("Error sending toast: {}", e);
                                }
                            };
                            return;
                        }

                        if path.is_dir() {
                            // get file count and return PreloadImages, then iterate over them
                            let mut file_count = 0;
                            let mut dir = read_dir(path).await.unwrap();
                            let mut paths = Vec::new();
                            while let Ok(entry) = dir.next_entry().await {
                                let entry = entry.unwrap();
                                if entry.file_type().await.unwrap().is_file() {
                                    paths.push(entry.path());
                                    file_count += 1;
                                }
                            }

                            let _ = output
                                .send(
                                    ImageViewerMessage::PreloadImages {
                                        total_images: file_count,
                                    }
                                    .into(),
                                )
                                .await;

                            for (i, path) in paths.iter().enumerate() {
                                let contents = tokio::fs::read(path).await.unwrap();
                                let image = match image::load_from_memory(&contents) {
                                    Ok(image) => image.to_rgb8(),
                                    Err(e) => {
                                        error!("Error loading image {}: {}", path.display(), e);
                                        let _ = output
                                            .send(Message::PostToast(Toast::error(
                                                "Could not load image",
                                                format!("Error loading image: {}", e),
                                            )))
                                            .await;

                                        continue;
                                    }
                                };
                                let (width, height) = image.dimensions();

                                match output
                                    .send(
                                        ImageViewerMessage::LoadedImage {
                                            handle: Handle::from_rgba(
                                                width,
                                                height,
                                                image.into_raw(),
                                            ),
                                            height,
                                            index: i,
                                        }
                                        .into(),
                                    )
                                    .await
                                {
                                    Ok(()) => {}
                                    Err(e) => {
                                        error!("Error loading image: {}", e);
                                    }
                                }
                            }

                            return;
                        }

                        if let Some(extension) = path.extension() {
                            if extension == "cbz" {
                                let mut file = BufReader::new(File::open(path).await.unwrap());
                                let mut zip = ZipFileReader::with_tokio(&mut file).await.unwrap();

                                // TODO: Check how many actual images and ignore other files
                                let total_images = zip.file().entries().len();

                                match output
                                    .send(ImageViewerMessage::PreloadImages { total_images }.into())
                                    .await
                                {
                                    Ok(()) => {}
                                    Err(e) => {
                                        error!("Error preloading images: {}", e);
                                    }
                                }

                                // Add priority system so files near the current page are loaded first
                                for i in 0..total_images {
                                    let mut f = zip.reader_with_entry(i).await.unwrap();
                                    let mut buffer = vec![];
                                    f.read_to_end(&mut buffer).await.unwrap();
                                    let image = match image::load_from_memory(&buffer) {
                                        Ok(image) => image.to_rgba8(),
                                        Err(e) => {
                                            error!(
                                                "Error loading image {}: {}",
                                                f.entry().filename().as_str().unwrap(),
                                                e
                                            );
                                            let _ = output
                                                .send(Message::PostToast(Toast::error(
                                                    "Could not load image",
                                                    format!("Error loading image: {}", e),
                                                )))
                                                .await;

                                            continue;
                                        }
                                    };
                                    let (width, height) = image.dimensions();

                                    match output
                                        .send(
                                            ImageViewerMessage::LoadedImage {
                                                handle: Handle::from_rgba(
                                                    width,
                                                    height,
                                                    image.into_raw(),
                                                ),
                                                height,
                                                index: i,
                                            }
                                            .into(),
                                        )
                                        .await
                                    {
                                        Ok(()) => {}
                                        Err(e) => {
                                            error!("Error loading image: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                    },
                )
            }),
        ])
    }

    pub fn on_enter(_: &mut AppState) -> Task<Message> {
        Task::none()
    }

    pub fn view(&self, state: &AppState) -> iced::Element<'_, Message> {
        let clickable_area = container(row![
            mouse_area(
                Space::new()
                    .width(Length::FillPortion(2))
                    .height(Length::Fill)
            )
            .on_press(ImageViewerMessage::PrevPage.into()),
            mouse_area(
                Space::new()
                    .width(Length::FillPortion(1))
                    .height(Length::Fill)
            ),
            mouse_area(
                Space::new()
                    .width(Length::FillPortion(2))
                    .height(Length::Fill)
            )
            .on_press(ImageViewerMessage::NextPage.into())
        ]);

        let image: Element<Message> = match self.images.get(self.cur_page - 1) {
            Some(e) => match e {
                Some(i) => widget::image(i.0.clone())
                    .height(i.1 as f32 * state.config.image_viewer_preferences().zoom())
                    .into(),
                None => Space::new().width(Length::Fill).height(Length::Fill).into(),
            },
            None => Scrollable::new(text("Loading..."))
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .into(),
        };

        let image_area = Scrollable::new(stack![
            center(image).center_y(iced::Length::Shrink),
            clickable_area
        ])
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .id(SCROLLABLE);

        column![
            row![
                button(text("<")).on_press_maybe(if self.cur_page <= 1 {
                    None
                } else {
                    Some(ImageViewerMessage::PrevPage.into())
                }),
                text(format!("{} / {}", self.cur_page, self.images.len())),
                button(text(">")).on_press_maybe(if self.cur_page >= self.images.len() {
                    None
                } else {
                    Some(ImageViewerMessage::NextPage.into())
                }),
            ],
            image_area
        ]
        .align_x(iced::alignment::Horizontal::Center)
        .width(iced::Length::Fill)
        .into()
    }

    pub fn update(m: ImageViewerMessage, state: &mut AppState) -> Task<Message> {
        if let View::ImageViewer(v) = &mut state.view {
            match m {
                ImageViewerMessage::PreloadImages { total_images } => {
                    v.images = vec![None; total_images];
                }
                ImageViewerMessage::LoadedImage {
                    handle,
                    height,
                    index,
                } => {
                    v.images[index] = Some((handle, height));
                }
                ImageViewerMessage::PrevPage => {
                    if v.cur_page > 1 {
                        v.cur_page -= 1;
                        return snap_to(SCROLLABLE, scrollable::RelativeOffset::START);
                    }
                }
                ImageViewerMessage::NextPage => {
                    if v.cur_page < v.images.len() {
                        v.cur_page += 1;
                        return snap_to(SCROLLABLE, scrollable::RelativeOffset::START);
                    }
                }
                ImageViewerMessage::ScrollBy(offset) => {
                    return scroll_by(SCROLLABLE, offset);
                }
                ImageViewerMessage::ZoomIn => {
                    let new_zoom = state.config.image_viewer_preferences().zoom + 5;
                    state.config.set_zoom(new_zoom);
                }
                ImageViewerMessage::ZoomOut => {
                    let new_zoom = state.config.image_viewer_preferences().zoom - 5;
                    state.config.set_zoom(new_zoom);
                }
            }
        }
        Task::none()
    }
}
