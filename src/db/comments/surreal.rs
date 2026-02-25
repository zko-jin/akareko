use std::collections::HashSet;

use const_format::formatcp;
use fastbloom::BloomFilter;
use surrealdb::{Surreal, engine::local::Db};
use surrealdb_types::SurrealValue;
use tracing::info;

use crate::{
    db::{
        PaginateResponse,
        comments::{Post, Topic},
        user::User,
    },
    errors::DatabaseError,
};

pub struct PostRepository<'a> {
    db: &'a Surreal<Db>,
}

impl<'a> PostRepository<'a> {
    pub fn new(db: &'a Surreal<Db>) -> PostRepository<'a> {
        PostRepository { db }
    }
}

impl<'a> PostRepository<'a> {
    pub async fn add_comment(&self, post: Post) -> Result<Post, DatabaseError> {
        let result: Option<Post> = self
            .db
            .create((Post::TABLE_NAME, post.signature.as_base64()))
            .content(post)
            .await?;

        match result {
            Some(post) => {
                info!("Created post: {}", post.signature.as_base64());
                Ok(post)
            }
            None => Err(DatabaseError::Unknown),
        }
    }

    pub async fn get_posts_by_topic(
        &self,
        topic: Topic,
        take: usize,
        skip: usize,
    ) -> Result<PaginateResponse<(Vec<Post>, HashSet<User>)>, DatabaseError> {
        const QUERY: &'static str = formatcp!(
            "
            LET $rows = (
                SELECT *
                FROM {0}
                WHERE topic = $topic
                ORDER BY timestamp ASC
                LIMIT $take
                START $skip
            );

            LET $sources = $rows.map(|$r| $r.source);

            RETURN $sources;

            {{
                total: count(
                    SELECT *
                    FROM {0}
                    WHERE topic = $topic
                ),
                data: $rows,
                users: (
                    SELECT *
                    FROM $sources
                )
            }}
            ",
            Post::TABLE_NAME
        );

        #[derive(SurrealValue)]
        struct Response {
            total: usize,
            data: Vec<Post>,
            // TODO: Change this to HashSet when surrealdb-types supports it
            users: Vec<User>,
        }

        let result: Option<Response> = self
            .db
            .query(QUERY)
            .bind(("topic", topic))
            .bind(("take", take))
            .bind(("skip", skip))
            .await?
            .take(3)?;

        match result {
            Some(r) => Ok(PaginateResponse {
                values: (r.data, HashSet::from_iter(r.users)),
                total: r.total,
            }),
            None => Err(DatabaseError::Unknown),
        }
    }

    pub async fn make_filter(
        &self,
        topic: Topic,
        timestamp: u64,
    ) -> Result<BloomFilter, DatabaseError> {
        let query: String = format!(
            "
                SELECT * FROM {0} WHERE topic = $topic {1};
            ",
            Post::TABLE_NAME,
            if timestamp != 0 {
                " AND timestamp >= $timestamp"
            } else {
                ""
            }
        );

        let result: Vec<Post> = self
            .db
            .query(query)
            .bind(("topic", topic))
            .bind(("timestamp", timestamp))
            .await?
            .take(0)?;

        let mut filter = BloomFilter::with_false_pos(0.0001).expected_items(result.len());
        filter.insert(&result);

        Ok(filter)
    }

    pub async fn get_all_posts_by_topic(
        &self,
        topic: Topic,
        timestamp: u64,
        filter: Option<BloomFilter>,
    ) -> Result<Vec<Post>, DatabaseError> {
        let query: String = format!(
            "
                SELECT * FROM {0} WHERE topic = $topic {1};
            ",
            Post::TABLE_NAME,
            if timestamp != 0 {
                " AND timestamp >= $timestamp"
            } else {
                ""
            }
        );

        let result: Vec<Post> = self
            .db
            .query(query)
            .bind(("topic", topic))
            .bind(("timestamp", timestamp))
            .await?
            .take(0)?;

        let filtered_posts = match filter {
            Some(filter) => result.into_iter().filter(|p| !filter.contains(p)).collect(),
            None => result,
        };

        Ok(filtered_posts)
    }
}
