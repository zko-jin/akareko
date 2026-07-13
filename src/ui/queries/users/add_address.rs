use freya::query::*;

use crate::{daemon::resource_state::ResourceState, db::user::I2PAddress, ui::AppResources};

#[derive(PartialEq, Eq, Clone, Hash)]
pub struct AddAddress;

impl MutationCapability for AddAddress {
    type Ok = ();
    type Err = ();
    type Keys = I2PAddress;

    async fn run(&self, keys: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        match (AppResources::get_repositories(), AppResources::get_client()) {
            (ResourceState::Loaded(r), ResourceState::Loaded(c)) => {
                let resp = c.get_client().await.who(keys).await.unwrap();
                r.user().upsert_user(resp).await.unwrap();
                Ok(())
            }
            _ => Err(()),
        }
    }

    async fn on_settled(&self, _keys: &Self::Keys, result: &Result<Self::Ok, Self::Err>) {
        if let Ok(_) = result {
            // QueriesStorage::<FetchTorrentStatus>::invalidate_matching(hash.
            // clone()).await;
            // QueriesStorage::<FetchTorrentWatchers>::invalidate_all().await;
        }
    }
}
