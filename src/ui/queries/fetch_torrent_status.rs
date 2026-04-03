use anawt::{AnawtTorrentStatus, InfoHash};
use freya::{
    prelude::{try_consume_context, use_provide_context, use_try_consume},
    query::*,
    radio::RadioStation,
};

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
        let radio = try_consume_context::<RadioStation<AppState, AppChannel>>();
        dbg!(radio.is_some());
        let Some(radio) = radio else {
            return Err(TorrentError::NotInitialized);
        };

        match &radio.read().torrent_client {
            ResourceState::Loaded(r) => Ok(r.get_status(keys.clone()).await),
            _ => Err(TorrentError::NotInitialized),
        }
    }
}
