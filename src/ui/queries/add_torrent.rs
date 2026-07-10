use anawt::InfoHash;
use freya::query::*;

use crate::{
    daemon::resource_state::ResourceState,
    db::Magnet,
    errors::TorrentError,
    ui::{
        AppResources,
        queries::{FetchTorrentStatus, FetchTorrentWatchers},
    },
};

#[derive(PartialEq, Eq, Clone, Hash)]
pub struct AddTorrent;

impl MutationCapability for AddTorrent {
    type Ok = InfoHash;
    type Err = TorrentError;
    type Keys = (Magnet, String /* path */);

    async fn run(&self, keys: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        match AppResources::get_torrent_client() {
            ResourceState::Loaded(c) => c
                .add_magnet(&keys.0.0, &keys.1)
                .await
                .map_err(|_| TorrentError::Unknown),
            _ => Err(TorrentError::NotInitialized),
        }
    }

    async fn on_settled(&self, _keys: &Self::Keys, result: &Result<Self::Ok, Self::Err>) {
        if let Ok(hash) = result {
            QueriesStorage::<FetchTorrentStatus>::invalidate_matching(hash.clone()).await;
            QueriesStorage::<FetchTorrentWatchers>::invalidate_all().await;
        }
    }
}
