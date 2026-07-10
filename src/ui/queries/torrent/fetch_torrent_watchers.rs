use anawt::AnawtTorrentStatus;
use freya::query::QueryCapability;
use tokio::sync::watch;

use crate::{daemon::resource_state::ResourceState, errors::TorrentError, ui::AppResources};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct FetchTorrentWatchers;

impl QueryCapability for FetchTorrentWatchers {
    type Ok = Vec<watch::Receiver<AnawtTorrentStatus>>;
    type Err = TorrentError;
    type Keys = ();

    async fn run(&self, _keys: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        match AppResources::get_torrent_client() {
            ResourceState::Loaded(c) => Ok(c.subscribe_all().await),
            _ => Err(TorrentError::NotInitialized),
        }
    }
}
