use anawt::{AnawtTorrentStatus, InfoHash};
use freya::query::*;

use crate::{daemon::resource_state::ResourceState, errors::TorrentError, ui::AppResources};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct FetchTorrentStatus;

impl QueryCapability for FetchTorrentStatus {
    type Ok = Option<AnawtTorrentStatus>;
    type Err = TorrentError;
    type Keys = InfoHash;

    async fn run(&self, keys: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        match AppResources::get_torrent_client() {
            ResourceState::Loaded(r) => Ok(r.get_status(keys.clone()).await),
            _ => Err(TorrentError::NotInitialized),
        }
    }
}
