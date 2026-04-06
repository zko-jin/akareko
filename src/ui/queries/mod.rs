use freya::{
    prelude::*,
    query::{MutationCapability, QueriesStorage},
    radio::RadioStation,
};

use crate::{
    db::index::{Index, content::Content, tags::IndexTag},
    errors::DatabaseError,
    ui::{AppChannel, AppState, ResourceState},
};

mod follow {
    pub mod follow_content;
    pub mod get_follow_content;
}
pub use follow::follow_content::FollowContent;
pub use follow::get_follow_content::GetFollowContent;

mod fetch_indexes;
pub use fetch_indexes::FetchIndexes;
mod fetch_contents;
pub use fetch_contents::FetchContents;
mod update_content_progress;
pub use update_content_progress::UpdateContentProgress;
mod fetch_torrent_status;
pub use fetch_torrent_status::FetchTorrentStatus;
mod add_torrent;
pub use add_torrent::AddTorrent;

#[derive(Clone)]
pub struct AddIndex<I: IndexTag> {
    _phantom: std::marker::PhantomData<I>,
}

impl<I: IndexTag> AddIndex<I> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<I: IndexTag> std::hash::Hash for AddIndex<I> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&0, state);
    }
}

impl<I: IndexTag> PartialEq for AddIndex<I> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<I: IndexTag> Eq for AddIndex<I> {}

impl<I: IndexTag + 'static> MutationCapability for AddIndex<I> {
    type Ok = ();
    type Err = DatabaseError;
    type Keys = Index<I>;

    async fn run(&self, keys: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        let radio = try_consume_root_context::<RadioStation<AppState, AppChannel>>();
        let Some(radio) = radio else {
            return Err(DatabaseError::NotInitialized);
        };

        match &radio.read().repositories {
            ResourceState::Loaded(r) => {
                r.index().add_index(keys.clone()).await?;
                Ok(())
            }
            _ => Err(DatabaseError::NotInitialized),
        }
    }

    async fn on_settled(&self, _keys: &Self::Keys, _result: &Result<Self::Ok, Self::Err>) {
        QueriesStorage::<FetchIndexes<I>>::invalidate_all().await;
    }
}

#[derive(Clone)]
pub struct AddIndexContent<I: IndexTag> {
    _phantom: std::marker::PhantomData<I>,
}

impl<I: IndexTag> AddIndexContent<I> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<I: IndexTag> std::hash::Hash for AddIndexContent<I> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&0, state);
    }
}

impl<I: IndexTag> PartialEq for AddIndexContent<I> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<I: IndexTag> Eq for AddIndexContent<I> {}

impl<I: IndexTag + 'static> MutationCapability for AddIndexContent<I> {
    type Ok = ();
    type Err = DatabaseError;
    type Keys = Content<I>;

    async fn run(&self, keys: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        let radio = try_consume_root_context::<RadioStation<AppState, AppChannel>>();
        let Some(radio) = radio else {
            return Err(DatabaseError::NotInitialized);
        };

        match &radio.read().repositories {
            ResourceState::Loaded(r) => r.index().add_content(keys.clone()).await,
            _ => Err(DatabaseError::NotInitialized),
        }
    }

    async fn on_settled(&self, _keys: &Self::Keys, _result: &Result<Self::Ok, Self::Err>) {
        QueriesStorage::<FetchIndexes<I>>::invalidate_all().await;
    }
}
