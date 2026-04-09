use std::time::Duration;

use freya::{
    prelude::*,
    query::{
        Mutation, MutationCapability, QueriesStorage, Query, QueryCapability, use_mutation,
        use_query,
    },
    radio::RadioStation,
};

use crate::{
    db::index::{content::Content, tags::IndexTag},
    errors::DatabaseError,
    types::Hash,
    ui::{AppChannel, AppState, ResourceState},
};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct FetchContents<I: IndexTag> {
    _phantom: std::marker::PhantomData<I>,
}

impl<I: IndexTag + 'static> QueryCapability for FetchContents<I> {
    type Ok = Vec<Content<I>>;
    type Err = DatabaseError;
    type Keys = Hash;

    async fn run(&self, keys: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        let radio = try_consume_root_context::<RadioStation<AppState, AppChannel>>();
        let Some(radio) = radio else {
            return Err(DatabaseError::NotInitialized);
        };

        match &radio.read().repositories.clone() {
            ResourceState::Loaded(r) => {
                r.index()
                    .get_filtered_index_contents(keys.clone(), None, None)
                    .await
            }
            _ => Err(DatabaseError::NotInitialized),
        }
    }
}

impl<I: IndexTag> FetchContents<I> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}
