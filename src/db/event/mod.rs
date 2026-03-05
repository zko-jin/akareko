use fastbloom::BloomFilter;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use strum_macros::EnumCount;
use surrealdb::{Surreal, engine::local::Db, method::Transaction};
use surrealdb_types::{SurrealValue, Value};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    db::{
        BLOOM_FILTER_FALSE_POSITIVE_RATE, Timestamp,
        comments::Topic,
        index::tags::{IndexTag, MangaTag},
    },
    errors::{DatabaseError, DecodeError, EncodeError},
    helpers::{Byteable, Encodeable},
};

#[derive(SurrealValue, Debug, Clone)]
pub struct Event {
    pub timestamp: Timestamp,
    pub event_type: EventType,
    pub topic: Topic,
}

pub async fn insert_event(events: Vec<Event>, db: &Transaction<Db>) -> Result<(), DatabaseError> {
    let _: Vec<Value> = db.insert("events").content(events).await?;
    Ok(())
}

pub async fn get_paginated_events(
    page: usize,
    per_page: usize,
    db: &Surreal<Db>,
) -> Result<(Vec<Event>, usize), DatabaseError> {
    const QUERY: &str = "
        LET $rows = (
            SELECT *
            FROM events
            ORDER BY timestamp DESC
            LIMIT $take
            START $skip
        );

        {{
            total: count(
                SELECT *
                FROM events
            ),
            data: $rows
        }}
        ";

    #[derive(SurrealValue)]
    struct Response {
        total: usize,
        data: Vec<Event>,
    }

    let events: Vec<Response> = db
        .query(QUERY)
        .bind(("take", per_page))
        .bind(("skip", (page - 1) * per_page))
        .await?
        .take(1)?;

    if let Some(response) = events.into_iter().next() {
        return Ok((response.data, (response.total / per_page) + 1));
    }

    Err(DatabaseError::Unknown)
}

pub async fn filter_events(
    timestamp: Timestamp,
    filter: Option<BloomFilter>,
    db: &Surreal<Db>,
) -> Result<Vec<(EventType, Vec<Topic>)>, DatabaseError> {
    const QUERY: &'static str = "
                SELECT event_type, array::group(topic) AS topics FROM events WHERE timestamp >= $timestamp GROUP BY event_type;
            ";

    #[derive(SurrealValue)]
    struct Grouped {
        event_type: EventType,
        topics: Vec<Topic>,
    }

    let events: Vec<Grouped> = db
        .query(QUERY)
        .bind(("timestamp", timestamp))
        .await?
        .take(0)?;

    let mut response = vec![];

    for event in events {
        if let Some(filter) = &filter {
            response.push((
                event.event_type,
                event
                    .topics
                    .into_iter()
                    .filter(|e| {
                        println!("Checking {:?} {}", &e, !filter.contains(e));
                        !filter.contains(e)
                    })
                    .collect(),
            ));
        } else {
            response.push((event.event_type, event.topics));
        }
    }

    Ok(response)
}

pub async fn make_event_filter(
    timestamp: Timestamp,
    db: &Surreal<Db>,
) -> Result<BloomFilter, DatabaseError> {
    const QUERY: &'static str = "
        SELECT topic FROM events WHERE timestamp >= $timestamp ;
    ";

    #[derive(SurrealValue, Hash, Debug)]
    struct TopicWrapper {
        topic: Topic,
    }

    let topics: Vec<TopicWrapper> = db
        .query(QUERY)
        .bind(("timestamp", timestamp))
        .await?
        .take(0)?;

    let mut filter =
        BloomFilter::with_false_pos(BLOOM_FILTER_FALSE_POSITIVE_RATE).expected_items(topics.len());
    filter.insert_all(&topics);
    Ok(filter)
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    IntoPrimitive,
    TryFromPrimitive,
    SurrealValue,
    EnumCount,
)]
#[repr(u8)]
pub enum EventType {
    Invalid = 0,
    User = 1,
    Manga = 2,
    MangaContent = 3,
    Post = 4,
}

impl EventType {
    pub fn as_str(&self) -> &str {
        match self {
            EventType::Invalid => "",
            EventType::User => "user",
            EventType::Manga => MangaTag::TAG,
            EventType::MangaContent => MangaTag::CONTENT_TABLE,
            EventType::Post => "post",
        }
    }
}

impl Byteable for EventType {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        self.as_str().encode(writer).await
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        let value = String::decode(reader).await?;
        match value.as_str() {
            "user" => Ok(EventType::User),
            MangaTag::TAG => Ok(EventType::Manga),
            MangaTag::CONTENT_TABLE => Ok(EventType::MangaContent),
            "post" => Ok(EventType::Post),
            _ => Err(DecodeError::InvalidEnumVariant {
                enum_name: "EventType",
                variant_value: value,
            }),
        }
    }
}

impl std::hash::Hash for Event {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.topic.hash(state);
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        db::{Repositories, index::Index},
        hash::PrivateKey,
    };

    use super::*;

    #[tokio::test]
    async fn test_filter_events() {
        let repo = Repositories::in_memory().await;
        let index = Index::<MangaTag>::new_signed("test".to_string(), 0, &PrivateKey::new());

        repo.index().add_index(index.clone()).await.unwrap();

        let filter = make_event_filter(0, &repo.db).await.unwrap();

        let topic = Topic::from_index(&index);

        assert!(filter.contains(&topic));
    }
}
