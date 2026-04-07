use fastbloom::BloomFilter;
use surrealdb::{Surreal, engine::local::Db, types::RecordId};
use surrealdb_types::Value;

use crate::{
    db::{
        BLOOM_FILTER_FALSE_POSITIVE_RATE, Content,
        event::{Event, insert_event, remove_event},
        index::{Index, IndexTag},
    },
    errors::DatabaseError,
    types::{Hash, Signature, Timestamp, Topic},
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

        let timestamp = Timestamp::now();

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

    pub async fn add_content<T: IndexTag>(&self, content: Content<T>) -> Result<(), DatabaseError> {
        let transaction = self.db.clone().begin().await?;

        let timestamp = Timestamp::now();

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

    pub async fn update_content_progress<T: IndexTag>(
        &self,
        signature: Signature,
        progress: u32,
    ) -> Result<Option<Content<T>>, DatabaseError> {
        let query = format!("UPDATE $id SET progress = $progress");

        let content: Option<Content<T>> = self
            .db
            .query(query)
            .bind(("id", RecordId::new(T::CONTENT_TABLE, signature.as_base64())))
            .bind(("progress", progress))
            .await?
            .take(0)?;

        Ok(content)
    }

    pub async fn update_content_count<I: IndexTag>(
        &self,
        signature: Signature,
        count: u32,
    ) -> Result<Option<Content<I>>, DatabaseError> {
        let query = format!("UPDATE $id SET count = $count");

        let content: Option<Content<I>> = self
            .db
            .query(query)
            .bind(("id", RecordId::new(I::CONTENT_TABLE, signature.as_base64())))
            .bind(("count", count))
            .await?
            .take(0)?;

        Ok(content)
    }

    pub async fn remove_content<T: IndexTag>(
        &self,
        signature: Signature,
    ) -> Result<(), DatabaseError> {
        let transaction = self.db.clone().begin().await?;

        let topic = Topic::from_signature(signature.clone());

        remove_event(topic, &transaction).await?;

        let _: Option<Value> = transaction
            .delete(RecordId::new(T::CONTENT_TABLE, signature.as_base64()))
            .await?;

        transaction.commit().await?;

        Ok(())
    }

    pub async fn get_all_indexes<T: IndexTag>(
        &self,
        timestamp: Option<Timestamp>,
        filter: Option<BloomFilter>,
    ) -> Result<Vec<Index<T>>, DatabaseError> {
        let query_str = format!(
            "SELECT * FROM {} {};",
            T::TAG,
            if timestamp.is_some() {
                "WHERE timestamp >= $timestamp"
            } else {
                ""
            }
        );

        let mut query = self.db.query(query_str);

        if let Some(timestamp) = timestamp {
            query = query.bind(("timestamp", timestamp));
        }

        let results: Vec<Index<T>> = query.await?.take(0)?;

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
        timestamp: Option<Timestamp>,
        filter: Option<BloomFilter>,
    ) -> Result<Vec<Content<T>>, DatabaseError> {
        let query_str: String = format!(
            "SELECT * FROM {} WHERE index_hash = $index_hash {};",
            T::CONTENT_TABLE,
            if timestamp.is_some() {
                "WHERE timestamp >= $timestamp"
            } else {
                ""
            }
        );

        let mut query = self.db.query(query_str).bind(("index_hash", index_hash));

        dbg!("Querying");

        if let Some(timestamp) = timestamp {
            query = query.bind(("timestamp", timestamp));
        }

        let results: Vec<Content<T>> = query.await?.take(0)?;
        dbg!("Results");

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
        timestamp: Option<Timestamp>,
    ) -> Result<BloomFilter, DatabaseError> {
        let query_str: String = format!(
            "
                SELECT * FROM {0} WHERE index_hash = $index_hash {1};
            ",
            T::CONTENT_TABLE,
            if timestamp.is_some() {
                " AND timestamp >= $timestamp"
            } else {
                ""
            }
        );

        let mut query = self
            .db
            .query(query_str)
            .bind(("index_hash", index_hash.as_base64()));

        if let Some(timestamp) = timestamp {
            query = query.bind(("timestamp", timestamp));
        }

        let result: Vec<Content<T>> = query.await?.take(0)?;

        let mut filter = BloomFilter::with_false_pos(BLOOM_FILTER_FALSE_POSITIVE_RATE)
            .expected_items(result.len());
        filter.insert(&result);

        Ok(filter)
    }
}
