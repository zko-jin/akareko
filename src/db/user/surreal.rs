use rand::seq::IteratorRandom;
use surrealdb::{Surreal, engine::local::Db, types::RecordId};
use tracing::info;

use crate::{db::user::TrustLevel, errors::DatabaseError, hash::PublicKey};

use super::User;

pub struct UserRepository<'a> {
    db: &'a Surreal<Db>,
}

impl<'a> UserRepository<'a> {
    pub fn new(db: &'a Surreal<Db>) -> UserRepository<'a> {
        UserRepository { db }
    }
}

impl<'a> UserRepository<'a> {
    pub async fn upsert_user(&self, user: User) -> Result<User, DatabaseError> {
        let result: Vec<User> = self.db.upsert(User::TABLE_NAME).content(user).await?;

        match result.into_iter().next() {
            Some(user) => {
                info!("Created user: {}", user.name());
                Ok(user)
            }
            None => Err(DatabaseError::Unknown),
        }
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

        let results: Vec<User> = self
            .db
            .query("SELECT * FROM $ids")
            .bind(("ids", ids))
            .await?
            .take(0)?;

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
