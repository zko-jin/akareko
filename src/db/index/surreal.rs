use fastbloom::BloomFilter;
use surrealdb::{Surreal, engine::local::Db, types::RecordId};
use surrealdb_types::Value;

use crate::{
    db::{
        BLOOM_FILTER_FALSE_POSITIVE_RATE, Content, Timestamp,
        comments::Topic,
        event::{Event, insert_event},
        index::{Index, IndexTag},
    },
    errors::DatabaseError,
    hash::{Hash, Signature},
    helpers::now_timestamp,
};

// ==================== End Imports ====================

pub struct IndexRepository<'a> {
    db: &'a Surreal<Db>,
}

impl<'a> IndexRepository<'a> {
    pub fn new(db: &'a Surreal<Db>) -> IndexRepository<'a> {
        IndexRepository { db }
    }
}

impl<'a> IndexRepository<'a> {
    pub async fn add_index<T: IndexTag>(&self, index: Index<T>) -> Result<Index<T>, DatabaseError> {
        let transaction = self.db.clone().begin().await?;

        let timestamp = now_timestamp();

        let event = Event {
            timestamp,
            event_type: T::EVENT_TYPE,
            topic: Topic::from_index(&index),
        };

        insert_event(vec![event], &transaction).await?;

        let created: Vec<Index<T>> = transaction.upsert(T::TAG).content(index).await?;

        let r = match created.len() {
            1 => created.into_iter().next().unwrap(),
            _ => return Err(DatabaseError::Unknown),
        };

        transaction.commit().await?;

        Ok(r)
    }

    pub async fn add_content<T: IndexTag + 'static>(
        &self,
        content: Content<T>,
    ) -> Result<(), DatabaseError> {
        let transaction = self.db.clone().begin().await?;

        let timestamp = now_timestamp();

        let event = Event {
            timestamp,
            event_type: T::CONTENT_EVENT_TYPE,
            topic: Topic::from_content(&content),
        };

        insert_event(vec![event], &transaction).await?;

        let _: Vec<Value> = transaction
            .upsert(T::CONTENT_TABLE)
            .content(content)
            .await?;

        transaction.commit().await?;

        Ok(())
    }

    pub async fn get_all_indexes<T: IndexTag>(
        &self,
        timestamp: Timestamp,
        filter: Option<BloomFilter>,
    ) -> Result<Vec<Index<T>>, DatabaseError> {
        let query = format!(
            "SELECT * FROM {} {};",
            T::TAG,
            if timestamp != 0 {
                "WHERE timestamp >= $timestamp"
            } else {
                ""
            }
        );

        let results: Vec<Index<T>> = self
            .db
            .query(query)
            .bind(("timestamp", timestamp))
            .await?
            .take(0)?;

        let filtered_indexes = match filter {
            Some(filter) => results
                .into_iter()
                .filter(|i| !filter.contains(i))
                .collect(),
            None => results,
        };

        Ok(filtered_indexes)
    }

    pub async fn get_indexes<T: IndexTag>(
        &self,
        hashes: &[Hash],
    ) -> Result<Vec<Index<T>>, DatabaseError> {
        let ids: Vec<RecordId> = hashes
            .iter()
            .map(|h| RecordId::new(T::TAG, h.as_base64()))
            .collect();

        let results: Vec<Index<T>> = self
            .db
            .query("SELECT * FROM $ids")
            .bind(("ids", ids))
            .await?
            .take(0)?;

        Ok(results)
    }

    pub async fn get_contents<T: IndexTag>(
        &self,
        signatures: &[Signature],
    ) -> Result<Vec<Content<T>>, DatabaseError> {
        let ids: Vec<RecordId> = signatures
            .iter()
            .map(|s| RecordId::new(T::CONTENT_TABLE, s.as_base64()))
            .collect();

        let results: Vec<Content<T>> = self
            .db
            .query("SELECT * FROM $ids")
            .bind(("ids", ids))
            .await?
            .take(0)?;

        Ok(results)
    }

    pub async fn get_index<T: IndexTag>(
        &self,
        hash: &Hash,
    ) -> Result<Option<Index<T>>, DatabaseError> {
        let result: Option<Index<T>> = self.db.select((T::TAG, hash.as_base64())).await?;
        Ok(result)
    }

    pub async fn get_filtered_index_contents<T: IndexTag>(
        &self,
        index_hash: Hash,
        timestamp: Timestamp,
        filter: Option<BloomFilter>,
    ) -> Result<Vec<Content<T>>, DatabaseError> {
        let query: String = format!(
            "SELECT * FROM {} WHERE index_hash = $index_hash {};",
            T::CONTENT_TABLE,
            if timestamp != 0 {
                "WHERE timestamp >= $timestamp"
            } else {
                ""
            }
        );

        let results: Vec<Content<T>> = self
            .db
            .query(query)
            .bind(("index_hash", index_hash))
            .await?
            .take(0)?;

        let contents = match filter {
            Some(filter) => results
                .into_iter()
                .filter(|c| !filter.contains(c))
                .collect(),
            None => results,
        };

        Ok(contents)
    }

    pub async fn make_filter<T: IndexTag>(
        &self,
        index_hash: &Hash,
        timestamp: u64,
    ) -> Result<BloomFilter, DatabaseError> {
        let query: String = format!(
            "
                SELECT * FROM {0} WHERE index_hash = $index_hash {1};
            ",
            T::CONTENT_TABLE,
            if timestamp != 0 {
                " AND timestamp >= $timestamp"
            } else {
                ""
            }
        );

        let result: Vec<Content<T>> = self
            .db
            .query(query)
            .bind(("index_hash", index_hash.as_base64()))
            .bind(("timestamp", timestamp))
            .await?
            .take(0)?;

        let mut filter = BloomFilter::with_false_pos(BLOOM_FILTER_FALSE_POSITIVE_RATE)
            .expected_items(result.len());
        filter.insert(&result);

        Ok(filter)
    }
}
