use anawt::{InfoHash, RemoveFlags};
use freya::query::*;

use crate::{
    daemon::resource_state::ResourceState,
    errors::TorrentError,
    ui::{
        AppResources,
        queries::{FetchTorrentStatus, FetchTorrentWatchers},
    },
};

#[derive(PartialEq, Eq, Clone, Hash)]
pub struct RemoveTorrent;

impl MutationCapability for RemoveTorrent {
    type Ok = ();
    type Err = TorrentError;
    type Keys = (InfoHash, RemoveFlags);

    async fn run(&self, keys: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        match AppResources::get_torrent_client() {
            ResourceState::Loaded(c) => c
                .remove_torrent(keys.0, keys.1)
                .await
                .map_err(|_| TorrentError::Unknown),
            _ => Err(TorrentError::NotInitialized),
        }
    }

    async fn on_settled(&self, keys: &Self::Keys, result: &Result<Self::Ok, Self::Err>) {
        if result.is_ok() {
            QueriesStorage::<FetchTorrentStatus>::invalidate_matching(keys.0).await;
            QueriesStorage::<FetchTorrentWatchers>::invalidate_all().await;
        }
    }
}
