use std::time::Duration;

use anawt::InfoHash;
use freya::{prelude::*, query::*};

use crate::{
    db::index::{
        content::Content,
        tags::{IndexTag, MangaTag},
    },
    ui::{
        DEFAULT_CORNER_RADIUS, Route, RouteContext,
        components::{Spacer, svg_button},
        icons::{self, EYE_ICON},
        queries::{AddTorrent, FetchTorrentStatus, UpdateContentProgress},
    },
};

mod sealed {
    pub trait VisualizeRouteSealed {}
}

pub struct ContentEntry<I: IndexTag + VisualizeRoute<I>> {
    content: Content<I>,
}

impl<I: IndexTag + VisualizeRoute<I>> Component for ContentEntry<I> {
    fn render(&self) -> impl IntoElement {
        let info_hash = InfoHash::from_magnet(&self.content.magnet_link.0).unwrap();
        let torrent_status = use_query(
            Query::new(info_hash, FetchTorrentStatus).interval_time(Duration::from_millis(500)),
        );

        let seen_mutation = use_mutation(Mutation::new(UpdateContentProgress::<I>::new()));
        let download_mutation = use_mutation(Mutation::new(AddTorrent));

        let watch_icon = {
            let content = self.content.clone();

            if content.progress < content.count {
                svg_button(icons::EYE_ICON, 20., Color::WHITE)
                    .on_press(move |_| {
                        seen_mutation.mutate((content.signature().clone(), content.count));
                    })
                    .hover_background(Color::TRANSPARENT)
            } else {
                svg_button(icons::EYE_SLASH_ICON, 20., Color::LIGHT_GRAY)
                    .on_press(move |_| {
                        seen_mutation.mutate((content.signature().clone(), 0));
                    })
                    .hover_background(Color::TRANSPARENT)
            }
        };

        let (torrent_status_icon, on_press_title): (
            Element,
            Option<EventHandler<Event<PressEventData>>>,
        ) = match &*torrent_status.read().state() {
            QueryStateData::Settled {
                res: Ok(status), ..
            }
            | QueryStateData::Loading {
                res: Some(Ok(status)),
            } => {
                let content = self.content.clone();
                let open_file = move |_| {
                    RouteContext::get().push(I::visualize_route(content.clone()));
                };
                match status {
                    Some(s) => match &s.state {
                        anawt::TorrentState::CheckingFiles => (rect().into_element(), None),
                        anawt::TorrentState::DownloadingMetadata => (rect().into_element(), None),
                        anawt::TorrentState::Downloading => (
                            ProgressBar::new(s.progress as f32 * 100.0).into_element(),
                            None,
                        ),
                        anawt::TorrentState::Finished => {
                            (rect().child("✓").into_element(), Some(open_file.into()))
                        }
                        anawt::TorrentState::Seeding => {
                            (rect().child("✓").into_element(), Some(open_file.into()))
                        }
                        anawt::TorrentState::CheckingResumeData => (rect().into_element(), None),
                    },
                    None => {
                        let keys = (
                            self.content.magnet_link.clone(),
                            format!("./data/mangas/{}", self.content.signature().as_base64()),
                        );
                        let download_torrent: EventHandler<Event<PressEventData>> = (move |_| {
                            download_mutation.mutate(keys.clone());
                        })
                        .into();
                        (
                            Button::new()
                                .child(
                                    svg(icons::DOWNLOAD_ICON)
                                        .on_press(download_torrent.clone())
                                        .color(Color::WHITE),
                                )
                                .into_element(),
                            Some(download_torrent),
                        )
                    }
                }
            }
            QueryStateData::Pending { .. } | QueryStateData::Loading { .. } => {
                (CircularLoader::new().into_element(), None)
            }
            QueryStateData::Settled { res: Err(e), .. } => (
                TooltipContainer::new(Tooltip::new(e.to_string()))
                    .child("X")
                    .into_element(),
                None,
            ),
        };

        let first_line = rect()
            .horizontal()
            .content(freya::prelude::Content::Flex)
            .cross_align(Alignment::Center)
            .child(
                rect()
                    .child(
                        label()
                            .text(self.content.title().to_string())
                            .color(Color::WHITE),
                    )
                    .on_pointer_enter(move |_| {
                        if true {
                            Cursor::set(CursorIcon::Pointer);
                        } else {
                            Cursor::set(CursorIcon::NotAllowed);
                        }
                    })
                    .on_pointer_leave(move |_| {
                        Cursor::set(CursorIcon::default());
                    })
                    .maybe(on_press_title.is_some(), move |l| {
                        l.on_press(on_press_title.unwrap())
                    }),
            )
            .child(Spacer::horizontal_fill())
            .child(torrent_status_icon)
            .child(watch_icon);

        rect()
            .width(Size::Fill)
            .child(first_line.padding(5.))
            .child(
                rect()
                    .width(Size::Fill)
                    .background(Color::GRAY)
                    .child(
                        label()
                            .text("Group: Anon")
                            .color(Color::WHITE)
                            .font_size(14),
                    )
                    .padding((0., 5.)),
            )
            .child(
                ProgressBar::new(50.)
                    .show_progress(false)
                    .color(Color::TRANSPARENT)
                    .width(Size::Fill)
                    .height(10.),
            )
            .corner_radius(DEFAULT_CORNER_RADIUS)
            .background(Color::DARK_GRAY)
    }
}

impl<I: IndexTag + VisualizeRoute<I>> ContentEntry<I> {
    pub fn new(content: Content<I>) -> Self {
        Self { content }
    }
}

pub trait VisualizeRoute<I: IndexTag>: sealed::VisualizeRouteSealed {
    fn visualize_route(content: Content<I>) -> Route;
}

impl sealed::VisualizeRouteSealed for MangaTag {}
impl VisualizeRoute<MangaTag> for MangaTag {
    fn visualize_route(content: Content<MangaTag>) -> Route {
        Route::ChapterViewer { content }
    }
}

impl<I: IndexTag + VisualizeRoute<I>> PartialEq for ContentEntry<I> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
