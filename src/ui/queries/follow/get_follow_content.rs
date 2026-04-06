use freya::{prelude::*, query::QueryCapability, radio::RadioStation};

use crate::{
    db::{follow_index::IndexFollow, index::tags::IndexTag},
    errors::DatabaseError,
    types::Hash,
    ui::{AppChannel, AppState, ResourceState},
};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct GetFollowContent<I: IndexTag>(std::marker::PhantomData<I>);

impl<I: IndexTag> GetFollowContent<I> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<I: IndexTag> QueryCapability for GetFollowContent<I> {
    type Ok = Option<IndexFollow<I>>;
    type Err = DatabaseError;
    type Keys = Hash;

    async fn run(&self, keys: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        let radio = try_consume_root_context::<RadioStation<AppState, AppChannel>>();
        let Some(radio) = radio else {
            return Err(DatabaseError::NotInitialized);
        };

        match &radio.read().repositories {
            ResourceState::Loaded(r) => r.index_follow().get_index_follow(keys.clone()).await,
            _ => Err(DatabaseError::NotInitialized),
        }
    }
}
