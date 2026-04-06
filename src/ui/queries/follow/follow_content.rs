use std::marker::PhantomData;

use freya::{prelude::*, query::*, radio::RadioStation};

use crate::{
    db::{Magnet, follow_index::IndexFollow, index::tags::IndexTag},
    errors::{DatabaseError, TorrentError},
    types::{Hash, Timestamp},
    ui::{
        AppChannel, AppState, ResourceState,
        queries::{FetchTorrentStatus, GetFollowContent},
    },
};

#[derive(PartialEq, Eq, Clone, Hash)]
pub struct FollowContent<I: IndexTag>(PhantomData<I>);

impl<I: IndexTag> FollowContent<I> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<I: IndexTag> MutationCapability for FollowContent<I> {
    type Ok = ();
    type Err = DatabaseError;
    type Keys = (Hash, bool);

    async fn run(&self, keys: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        let radio = try_consume_root_context::<RadioStation<AppState, AppChannel>>();
        let Some(radio) = radio else {
            return Err(DatabaseError::NotInitialized);
        };

        match &radio.read().repositories {
            ResourceState::Loaded(r) => {
                if keys.1 {
                    r.index_follow()
                        .add_index_follow::<I>(IndexFollow::new(
                            keys.0.clone(),
                            true,
                            Timestamp::new(0),
                        ))
                        .await
                        .map(|_| ())
                } else {
                    r.index_follow()
                        .remove_index_follow::<I>(keys.0.clone())
                        .await
                        .map(|_| ())
                }
            }

            _ => Err(DatabaseError::NotInitialized),
        }
    }

    async fn on_settled(&self, keys: &Self::Keys, result: &Result<Self::Ok, Self::Err>) {
        if result.is_ok() {
            QueriesStorage::<GetFollowContent<I>>::invalidate_matching(keys.0.clone()).await;
        }
    }
}
