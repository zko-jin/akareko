use freya::query::QueryCapability;

use crate::{
    daemon::resource_state::ResourceState,
    db::index::{content::Content, tags::IndexTag},
    errors::DatabaseError,
    types::Hash,
    ui::AppResources,
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
        match AppResources::get_repositories() {
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
