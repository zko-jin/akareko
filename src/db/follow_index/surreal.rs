use surrealdb::{Surreal, engine::local::Db};
use tracing::info;

use crate::{
    db::{
        follow_index::IndexFollow,
        index::{Index, tags::IndexTag},
    },
    errors::DatabaseError,
    hash::Hash,
};

pub struct IndexFollowRepository<'a> {
    db: &'a Surreal<Db>,
}

impl<'a> IndexFollowRepository<'a> {
    pub fn new(db: &'a Surreal<Db>) -> IndexFollowRepository<'a> {
        IndexFollowRepository { db }
    }
}

impl<'a> IndexFollowRepository<'a> {
    pub async fn add_index_follow<T: IndexTag>(
        &self,
        follow: IndexFollow<T>,
    ) -> Result<IndexFollow<T>, DatabaseError> {
        let result: Option<IndexFollow<T>> = self
            .db
            .create(IndexFollow::<T>::table_name())
            .content(follow)
            .await?;

        match result {
            Some(follow) => {
                info!("Added follow: {}", follow.index);
                Ok(follow)
            }
            None => Err(DatabaseError::Unknown),
        }
    }

    pub async fn get_index_follow<T: IndexTag>(
        &self,
        index: Hash,
    ) -> Result<Option<IndexFollow<T>>, DatabaseError> {
        let result: Option<IndexFollow<T>> = self
            .db
            .select((IndexFollow::<T>::table_name(), index.as_base64()))
            .await?;

        Ok(result)
    }

    pub async fn remove_index_follow<T: IndexTag>(&self, index: Hash) -> Result<(), DatabaseError> {
        let _: Option<surrealdb_types::Value> = self
            .db
            .delete((IndexFollow::<T>::table_name(), index.as_base64()))
            .await?;

        Ok(())
    }

    pub async fn get_followed_indexes<T: IndexTag>(
        &self,
        take: usize,
        skip: usize,
    ) -> Result<Vec<(IndexFollow<T>, Index<T>)>, DatabaseError> {
        let query = format!(
            "
                SELECT *
                FROM {0}
                LIMIT $take
                START $skip;
            ",
            IndexFollow::<T>::table_name()
        );

        let result: Vec<(IndexFollow<T>, Index<T>)> = self
            .db
            .query(query)
            .bind(("take", take))
            .bind(("skip", skip))
            .await?
            .take(0)?;

        Ok(result)
    }
}
