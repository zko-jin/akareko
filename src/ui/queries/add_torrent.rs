use anawt::InfoHash;
use freya::{prelude::try_consume_context, query::*, radio::RadioStation};

use crate::{
    db::{
        Magnet,
        index::{content::Content, tags::IndexTag},
    },
    errors::{DatabaseError, TorrentError},
    types::Signature,
    ui::{
        AppChannel, AppState, ResourceState,
        queries::{FetchContents, FetchTorrentStatus},
    },
};

#[derive(PartialEq, Eq, Clone, Hash)]
pub struct AddTorrent;

impl MutationCapability for AddTorrent {
    type Ok = InfoHash;
    type Err = TorrentError;
    type Keys = (Magnet, String /* path */);

    async fn run(&self, keys: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        println!("TRYING HARD");

        let radio = try_consume_context::<RadioStation<AppState, AppChannel>>();
        let Some(radio) = radio else {
            return Err(TorrentError::NotInitialized);
        };

        match &radio.read().torrent_client {
            ResourceState::Loaded(c) => {
                let x = c
                    .add_magnet(&keys.0.0, &keys.1)
                    .await
                    .map_err(|_| TorrentError::Unknown);
                println!("ADDED ");
                x
            }
            _ => Err(TorrentError::NotInitialized),
        }
    }

    async fn on_settled(&self, _keys: &Self::Keys, result: &Result<Self::Ok, Self::Err>) {
        if let Ok(hash) = result {
            QueriesStorage::<FetchTorrentStatus>::invalidate_matching(hash.clone()).await;
        }
    }
}
