use freya::{prelude::*, query::*, radio::RadioStation};

use crate::{
    db::index::{content::Content, tags::IndexTag},
    errors::DatabaseError,
    types::Signature,
    ui::{AppChannel, AppState, ResourceState, queries::FetchContents},
};

#[derive(PartialEq, Eq, Clone, Hash)]
pub struct UpdateContentCount<I: IndexTag> {
    _phantom: std::marker::PhantomData<I>,
}

impl<I: IndexTag> UpdateContentCount<I> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<I: IndexTag> MutationCapability for UpdateContentCount<I> {
    type Ok = Option<Content<I>>;
    type Err = DatabaseError;
    type Keys = (Signature, u32);

    async fn run(&self, keys: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        let radio = try_consume_root_context::<RadioStation<AppState, AppChannel>>();
        let Some(radio) = radio else {
            return Err(DatabaseError::NotInitialized);
        };

        match &radio.read().repositories {
            ResourceState::Loaded(r) => {
                r.index()
                    .update_content_count::<I>(keys.0.clone(), keys.1)
                    .await
            }
            _ => Err(DatabaseError::NotInitialized),
        }
    }

    async fn on_settled(&self, _keys: &Self::Keys, result: &Result<Self::Ok, Self::Err>) {
        if let Ok(Some(content)) = result {
            QueriesStorage::<FetchContents<I>>::invalidate_matching(content.index_hash().clone())
                .await;
        }
    }
}
