use anawt::{AnawtTorrentStatus, InfoHash, TorrentState};
use iced::{
    Color, Element, Length, Subscription, Task,
    widget::{Column, button, progress_bar, row, svg, text},
};
use tokio::sync::watch;
use tracing::info;

use crate::{
    db::{
        comments::Topic,
        follow_index::IndexFollow,
        index::{
            Index,
            content::Content,
            tags::{IndexTag as _, MangaTag},
        },
    },
    helpers::SanitizedString,
    ui::{
        AppState,
        components::toast::Toast,
        icons::{
            BOOK_BOOKMARK_ICON, CHAT_ICON, CHECK_CIRCLE_ICON, DOWNLOAD_ICON, SEEN_ICON, UNSEEN_ICON,
        },
        message::Message,
        style,
        views::{
            View, ViewMessage, add_chapter::AddMangaChapterView, image_viewer::ImageViewerView,
            post::PostView,
        },
    },
};

// ==================== End Imports ====================

#[derive(Debug, Clone)]
pub struct MangaView {
    follow: bool,
    manga: Index<MangaTag>,
    chapters: Vec<Content<MangaTag>>,
    pub torrents: Vec<Option<watch::Receiver<AnawtTorrentStatus>>>,
}

#[derive(Debug, Clone)]
pub enum MangaMessage {
    ContentLoaded(Vec<Content<MangaTag>>),
    LoadedTorrentWatcher(Vec<Option<watch::Receiver<AnawtTorrentStatus>>>),
    ReloadTorrents,
    DownloadTorrentAndReload { magnet: String, path: String },
    UpdateProgress(usize, usize, f32),
    TorrentStatusUpdated,
    LoadedFollow(bool),
    ToggleFollow,
}

impl From<MangaMessage> for Message {
    fn from(m: MangaMessage) -> Self {
        Message::ViewMessage(ViewMessage::Manga(m))
    }
}

impl MangaView {
    pub fn new(novel: Index<MangaTag>) -> Self {
        Self {
            follow: false,
            manga: novel,
            chapters: vec![],
            torrents: Vec::new(),
        }
    }

    pub fn on_enter(state: &mut AppState) -> Task<Message> {
        if let View::Novel(v) = &mut state.view {
            if let Some(repositories) = &state.repositories {
                let repositories = repositories.clone();
                let novel_hash = v.manga.hash().clone();
                return Task::future(async move {
                    let chapters = repositories
                        .index()
                        .get_filtered_index_contents(novel_hash, 0, None)
                        .await;
                    MangaMessage::ContentLoaded(chapters.unwrap()).into()
                });
            }
        }
        Task::none()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        Subscription::none()
    }

    pub fn view(&self, _: &AppState) -> iced::Element<'_, Message> {
        let mut column = Column::new().push(row![
            text(self.manga.title().clone()),
            button(
                svg(svg::Handle::from_memory(BOOK_BOOKMARK_ICON))
                    .height(Length::Fixed(24.0))
                    .width(Length::Fixed(24.0))
                    .style(|_, _| svg::Style {
                        color: Some(Color::WHITE),
                    })
            )
            .style(style::icon_button)
            .on_press(MangaMessage::ToggleFollow.into())
        ]);

        column = column.push(button(text("Add Chapter")).on_press(Message::ChangeView(
            View::AddChapter(AddMangaChapterView::new(self.manga.clone())),
        )));

        for i in 0..self.chapters.len() {
            let chapter = &self.chapters[i];
            let rx = self.torrents[i].as_ref();
            enum ContentState {
                Downloading(f64),
                Download,
                Ready,
            }
            let select_message: ContentState = match rx {
                Some(rx) => {
                    let status = rx.borrow();

                    match status.state {
                        TorrentState::Finished | TorrentState::Seeding => ContentState::Ready,
                        _ => ContentState::Downloading(status.progress),
                    }
                }
                None => ContentState::Download,
            };

            for (j, e) in chapter.entries().iter().enumerate() {
                let download_element: Element<Message> = match select_message {
                    ContentState::Downloading(p) => progress_bar(0.0..=1.0, p as f32).into(),
                    ContentState::Download => button(
                        svg(svg::Handle::from_memory(DOWNLOAD_ICON))
                            .height(Length::Fixed(24.0))
                            .width(Length::Fixed(24.0))
                            .style(|_, _| svg::Style {
                                color: Some(Color::WHITE),
                            }),
                    )
                    .style(style::icon_button)
                    .on_press(
                        MangaMessage::DownloadTorrentAndReload {
                            magnet: chapter.magnet_link.clone().0,
                            path: format!(
                                "./data/{}/{}/{}",
                                MangaTag::TAG,
                                SanitizedString::new(self.manga.title()).as_str(),
                                chapter.signature().as_base64_url()
                            ),
                        }
                        .into(),
                    )
                    .into(),
                    ContentState::Ready => button(
                        svg(svg::Handle::from_memory(CHECK_CIRCLE_ICON))
                            .height(24.0)
                            .width(24.0)
                            .style(|_, _| svg::Style {
                                color: Some(Color::WHITE),
                            }),
                    )
                    .style(style::icon_button)
                    .into(),
                };

                column = column.push(row![
                    button(text(e.title.clone()))
                        .on_press_maybe(match select_message {
                            ContentState::Downloading(_) => None,
                            ContentState::Download => None,
                            ContentState::Ready => Some(Message::ChangeView(View::ImageViewer(
                                // TODO: Instead of using the chapter signature for the path
                                // we should use the hash of the torrent
                                ImageViewerView::new(
                                    format!(
                                        "./data/{}/{}/{}/{}",
                                        MangaTag::TAG,
                                        SanitizedString::new(self.manga.title()).as_str(),
                                        chapter.signature().as_base64_url(),
                                        chapter.entries()[j].path
                                    )
                                    .into(),
                                )
                            ))),
                        })
                        .width(Length::Fill),
                    download_element,
                    if e.progress < 1.0 {
                        button(
                            svg(svg::Handle::from_memory(UNSEEN_ICON))
                                .height(Length::Fixed(24.0))
                                .width(Length::Fixed(24.0))
                                .style(|_, _| svg::Style {
                                    color: Some(Color::WHITE),
                                }),
                        )
                        .style(style::icon_button)
                        .on_press(MangaMessage::UpdateProgress(j, i, 1.0).into())
                    } else {
                        button(
                            svg(svg::Handle::from_memory(SEEN_ICON))
                                .height(Length::Fixed(24.0))
                                .width(Length::Fixed(24.0))
                                .style(|_, _| svg::Style {
                                    color: Some(Color::WHITE),
                                }),
                        )
                        .style(style::icon_button)
                        .on_press(MangaMessage::UpdateProgress(j, i, 0.0).into())
                    },
                    button(
                        svg(svg::Handle::from_memory(CHAT_ICON))
                            .height(Length::Fixed(24.0))
                            .width(Length::Fixed(24.0))
                            .style(|_, _| svg::Style {
                                color: Some(Color::WHITE),
                            }),
                    )
                    .style(style::icon_button)
                    .on_press(Message::ChangeView(View::Post(PostView::new(
                        Topic::from_entry(&self.manga, e.enumeration)
                    ))))
                ]);
            }
        }

        column.width(Length::Fill).into()
    }

    pub fn update(m: MangaMessage, state: &mut AppState) -> Task<Message> {
        if let View::Novel(v) = &mut state.view {
            match m {
                MangaMessage::ContentLoaded(chapters) => {
                    v.torrents = vec![None; chapters.len()];
                    v.chapters = chapters;
                    return Task::done(MangaMessage::ReloadTorrents.into());
                }
                MangaMessage::UpdateProgress(j, i, p) => {
                    v.chapters[i].update_entry_progress(j, p);
                }
                MangaMessage::ToggleFollow => {
                    v.follow = true;
                    if let Some(repositories) = &state.repositories {
                        let repositories = repositories.clone();
                        let index = v.manga.hash().clone();
                        return Task::future(async move {
                            let f = match repositories
                                .index_follow()
                                .get_index_follow::<MangaTag>(index)
                                .await
                            {
                                Ok(f) => f.is_some(),
                                Err(e) => {
                                    return Message::PostToast(Toast::error(
                                        "Failed to get follow status".into(),
                                        e.to_string(),
                                    ));
                                }
                            };

                            MangaMessage::LoadedFollow(f).into()
                        });
                    }
                }
                MangaMessage::LoadedFollow(f) => {
                    v.follow = f;
                }
                MangaMessage::LoadedTorrentWatcher(watchers) => {
                    info!("Loaded torrent watcher");
                    v.torrents = watchers;
                }
                MangaMessage::DownloadTorrentAndReload { magnet, path } => {
                    info!("Downloading and reloading: {}", magnet);
                    return Task::done(Message::DownloadTorrent { magnet, path })
                        .chain(Task::done(MangaMessage::ReloadTorrents.into()));
                }
                MangaMessage::ReloadTorrents => {
                    info!("Reloading torrents");
                    let torrent_client = state.torrent_client.clone();
                    if let Some(torrent_client) = torrent_client {
                        let chapters = v.chapters.clone();
                        let len = chapters.len();
                        return Task::future(async move {
                            let mut watchers = vec![None; len];

                            for (i, chapter) in chapters.iter().enumerate() {
                                let info_hash = match InfoHash::from_magnet(&chapter.magnet_link.0)
                                {
                                    Ok(info_hash) => info_hash,
                                    Err(_) => continue, // TODO: Invalid magnet, issue chapter deletion
                                };
                                let rx = torrent_client.subscribe_torrent(info_hash).await;
                                watchers[i] = rx;
                            }

                            MangaMessage::LoadedTorrentWatcher(watchers).into()
                        });
                    }
                }
                MangaMessage::TorrentStatusUpdated => {}
            }
        }

        Task::none()
    }
}
