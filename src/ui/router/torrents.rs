use anawt::AnawtTorrentStatus;
use freya::{
    prelude::*,
    query::{Query, QueryStateData, use_query},
    sdk::use_track_watcher,
};
use tokio::sync::watch;

use crate::ui::queries::FetchTorrentWatchers;

#[derive(PartialEq)]
pub struct Torrents;

impl Component for Torrents {
    fn render(&self) -> impl IntoElement {
        let watchers_query = use_query(Query::new((), FetchTorrentWatchers));

        let torrent_list = match &*watchers_query.read().state() {
            QueryStateData::Settled {
                res: Ok(watchers), ..
            } => {
                let children = watchers
                    .iter()
                    .map(|w| TorrentEntry::new(w.clone()).into_element())
                    .collect::<Vec<_>>();
                rect().vertical().children(children).into_element()
            }
            QueryStateData::Settled { res: Err(e), .. } => {
                rect().child(label().text(e.to_string())).into_element()
            }
            _ => CircularLoader::new().into_element(),
        };

        torrent_list
    }
}

pub struct TorrentEntry {
    watcher: watch::Receiver<AnawtTorrentStatus>,
}

impl TorrentEntry {
    pub fn new(watcher: watch::Receiver<AnawtTorrentStatus>) -> Self {
        Self { watcher }
    }
}

impl PartialEq for TorrentEntry {
    fn eq(&self, other: &Self) -> bool {
        self.watcher.same_channel(&other.watcher)
    }
}

impl Component for TorrentEntry {
    fn render(&self) -> impl IntoElement {
        use_track_watcher(&self.watcher);
        let status = self.watcher.borrow().clone();

        rect()
            .horizontal()
            .child(status.name)
            .child(ProgressBar::new(status.progress as f32 * 100.).show_progress(false))
            .width(Size::Fill)
    }
}
