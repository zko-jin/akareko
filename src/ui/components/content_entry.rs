use std::time::Duration;

use anawt::InfoHash;
use freya::{prelude::*, query::*};

use crate::{
    db::index::{
        content::Content,
        tags::{IndexTag, MangaTag},
    },
    ui::{
        Route, RouteContext, icons,
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
            let mut watch_icon = Button::new();
            let content = self.content.clone();

            if content.progress < content.count {
                watch_icon = watch_icon.child(svg(icons::EYE_ICON)).on_press(move |_| {
                    seen_mutation.mutate((content.signature().clone(), content.count));
                });
            } else {
                watch_icon = watch_icon
                    .child(svg(icons::EYE_SLASH_ICON))
                    .on_press(move |_| {
                        seen_mutation.mutate((content.signature().clone(), 0));
                    });
            };

            watch_icon
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
                                .child(svg(icons::DOWNLOAD_ICON).on_press(download_torrent.clone()))
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
            .child(
                label()
                    .text(self.content.title().to_string())
                    .maybe(on_press_title.is_some(), move |l| {
                        l.on_press(on_press_title.unwrap())
                    }),
            )
            .child(torrent_status_icon)
            .child(watch_icon);

        rect().child(first_line)
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
