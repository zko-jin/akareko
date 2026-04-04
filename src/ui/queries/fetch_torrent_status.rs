use anawt::{AnawtTorrentStatus, InfoHash};
use freya::{prelude::*, query::*, radio::RadioStation};

use crate::{
    errors::TorrentError,
    ui::{AppChannel, AppState, ResourceState},
};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct FetchTorrentStatus;

impl QueryCapability for FetchTorrentStatus {
    type Ok = Option<AnawtTorrentStatus>;
    type Err = TorrentError;
    type Keys = InfoHash;

    async fn run(&self, keys: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        let radio = try_consume_root_context::<RadioStation<AppState, AppChannel>>();
        let Some(radio) = radio else {
            return Err(TorrentError::NotInitialized);
        };

        match &radio.read().torrent_client {
            ResourceState::Loaded(r) => Ok(r.get_status(keys.clone()).await),
            _ => Err(TorrentError::NotInitialized),
        }
    }
}
