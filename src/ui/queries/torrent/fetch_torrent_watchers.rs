use anawt::AnawtTorrentStatus;
use freya::{prelude::*, query::QueryCapability, radio::RadioStation};
use tokio::sync::watch;

use crate::{
    errors::TorrentError,
    types::Hash,
    ui::{AppChannel, AppState, ResourceState},
};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct FetchTorrentWatchers;

impl QueryCapability for FetchTorrentWatchers {
    type Ok = Vec<watch::Receiver<AnawtTorrentStatus>>;
    type Err = TorrentError;
    type Keys = ();

    async fn run(&self, keys: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        let radio = try_consume_root_context::<RadioStation<AppState, AppChannel>>();
        let Some(radio) = radio else {
            return Err(TorrentError::NotInitialized);
        };

        match &radio.read().torrent_client {
            ResourceState::Loaded(c) => Ok(c.subscribe_all().await),
            _ => Err(TorrentError::NotInitialized),
        }
    }
}
