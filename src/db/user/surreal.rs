use surrealdb::{Surreal, engine::local::Db, types::RecordId};
use surrealdb_types::{SurrealValue, Value};

use crate::{
    db::{
        event::{Event, EventType, insert_event},
        user::TrustLevel,
    },
    errors::DatabaseError,
    types::{PublicKey, Timestamp, Topic},
};

use super::User;

pub struct UserRepository<'a> {
    db: &'a Surreal<Db>,
}

impl SurrealValue for TrustLevel {
    fn kind_of() -> surrealdb_types::Kind {
        surrealdb_types::Kind::Number
    }

    fn into_value(self) -> surrealdb_types::Value {
        (self as u8).into_value()
    }

    fn from_value(value: surrealdb_types::Value) -> Result<Self, surrealdb::Error>
    where
        Self: Sized,
    {
        let value = u8::from_value(value)?;
        value
            .try_into()
            .map_err(|e: num_enum::TryFromPrimitiveError<TrustLevel>| {
                surrealdb::Error::internal(e.to_string())
            })
    }
}

impl<'a> UserRepository<'a> {
    pub fn new(db: &'a Surreal<Db>) -> UserRepository<'a> {
        UserRepository { db }
    }
}

impl<'a> UserRepository<'a> {
    pub async fn upsert_user(&self, user: User) -> Result<(), DatabaseError> {
        let transaction = self.db.clone().begin().await?;

        let timestamp = Timestamp::now();

        let event = Event {
            timestamp,
            event_type: EventType::User,
            topic: Topic::from_user(&user),
        };

        insert_event(vec![event], &transaction).await?;

        let _: Vec<Value> = transaction.upsert(User::TABLE_NAME).content(user).await?;

        transaction.commit().await?;

        Ok(())
    }

    pub async fn upsert_users(&self, users: Vec<User>) -> Result<(), DatabaseError> {
        let transaction = self.db.clone().begin().await?;

        let timestamp = Timestamp::now();

        let events = users
            .iter()
            .map(|u| Event {
                timestamp,
                event_type: EventType::User,
                topic: Topic::from_user(u),
            })
            .collect();

        insert_event(events, &transaction).await?;

        let _: Vec<Value> = transaction.upsert(User::TABLE_NAME).content(users).await?;

        transaction.commit().await?;

        Ok(())
    }

    pub async fn get_users_b64(
        &self,
        pub_keys_base64: Vec<String>,
    ) -> Result<Vec<User>, DatabaseError> {
        let ids: Vec<RecordId> = pub_keys_base64
            .into_iter()
            .map(|p| RecordId::new(User::TABLE_NAME, p))
            .collect();

        let results: Vec<User> = self
            .db
            .query("SELECT * FROM $ids")
            .bind(("ids", ids))
            .await?
            .take(0)?;

        Ok(results)
    }

    pub async fn get_users(&self, pub_keys: Vec<PublicKey>) -> Result<Vec<User>, DatabaseError> {
        let ids: Vec<RecordId> = pub_keys
            .iter()
            .map(|p| RecordId::new(User::TABLE_NAME, p.to_base64()))
            .collect();

        let mut results: Vec<User> = vec![];
        for id in ids.iter() {
            let result = self.db.select(id).await?;
            if let Some(user) = result {
                results.push(user);
            }
        }

        // let results: Vec<User> = self
        //     .db
        //     .query("SELECT * FROM $ids")
        //     .bind(("ids", ids))
        //     .await?
        //     .take(0)?;

        Ok(results)
    }

    pub async fn get_random_users(
        &self,
        min_trust: TrustLevel,
        take: usize,
    ) -> Result<Vec<User>, DatabaseError> {
        const QUERY: &'static str =
            "SELECT * FROM users WHERE trust >= $min_trust ORDER BY RANDOM() LIMIT $take";

        let results: Vec<User> = self
            .db
            .query(QUERY)
            .bind(("min_trust", min_trust))
            .bind(("take", take))
            .await?
            .take(0)?;

        Ok(results)
    }

    pub async fn get_all_users(&self) -> Vec<User> {
        let results: Vec<User> = self.db.select("users").await.unwrap();
        results
    }

    pub async fn get_user(&self, pub_key: &PublicKey) -> Result<Option<User>, DatabaseError> {
        let results: Option<User> = self.db.select(("users", pub_key.to_base64())).await?;

        Ok(results)
    }
}
