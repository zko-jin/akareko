use std::marker::PhantomData;

use freya::query::*;

use crate::{
    daemon::resource_state::ResourceState,
    db::{follow_index::IndexFollow, index::tags::IndexTag},
    errors::DatabaseError,
    types::{Hash, Timestamp},
    ui::{AppResources, queries::GetFollowContent},
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
        match AppResources::get_repositories() {
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
