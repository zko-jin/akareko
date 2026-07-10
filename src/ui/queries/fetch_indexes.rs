use freya::query::QueryCapability;

use crate::{
    daemon::resource_state::ResourceState,
    db::index::{Index, tags::IndexTag},
    errors::DatabaseError,
    ui::AppResources,
};
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct FetchIndexes<I: IndexTag> {
    _phantom: std::marker::PhantomData<I>,
}

impl<I: IndexTag> FetchIndexes<I> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<I: IndexTag> QueryCapability for FetchIndexes<I> {
    type Ok = Vec<Index<I>>;
    type Err = DatabaseError;
    type Keys = ();

    async fn run(&self, _keys: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        match &AppResources::get_repositories() {
            ResourceState::Loaded(r) => r.index().get_all_indexes(None, None).await,
            _ => Err(DatabaseError::NotInitialized),
        }
    }
}
