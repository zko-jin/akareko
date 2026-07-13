use bytes::Bytes;
#[cfg(feature = "surrealdb")]
use skerry::skerry;
use std::{env, fmt::Debug};

use serde::{Deserialize, Serialize};
use surrealdb::{
    Surreal,
    engine::local::{Db, SurrealKv},
    opt::{Config, capabilities::Capabilities},
};
use surrealdb_types::SurrealValue;
use tracing::{info, warn};

use crate::db::{
    comments::Post,
    follow_index::IndexFollow,
    index::tags::{IndexTag, MangaTag},
};
use crate::errors::DatabaseError;
use crate::types::Timestamp;
use crate::{
    config::AkarekoConfig,
    db::{
        index::IndexRepository,
        user::{User, UserRepository},
    },
};
#[cfg(feature = "surrealdb")]
use crate::{db::follow_index::IndexFollowRepository, errors::RepositoriesRetrieveCoverError};
use crate::{db::index::content::Content, types::PublicKey};

// ==================== End Imports ====================

pub mod comments;
pub mod event;
pub mod follow_index;
pub mod group;
pub mod index;
pub mod schedule;
#[cfg(feature = "diesel")]
pub mod schema;
pub mod user;

pub const BLOOM_FILTER_FALSE_POSITIVE_RATE: f64 = 0.0001;

#[derive(Deserialize)]
pub struct PaginateResponse<T> {
    pub values: T,
    pub total: usize,
}

pub trait ToBytes {
    fn to_bytes(&self) -> Vec<u8>;
}

impl ToBytes for () {
    fn to_bytes(&self) -> Vec<u8> {
        Vec::new()
    }
}

impl ToBytes for String {
    fn to_bytes(&self) -> Vec<u8> {
        self.clone().into_bytes()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, SurrealValue)]
#[serde(transparent)]
pub struct Magnet(pub String);

#[derive(Clone)]
pub struct Repositories {
    #[cfg(feature = "surrealdb")]
    pub db: Surreal<Db>,
}

impl std::fmt::Debug for Repositories {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Repositories").finish()
    }
}

#[derive(Debug, Clone, SurrealValue)]
pub struct FullSyncTarget {
    #[surreal(rename = "id")]
    pub pub_key: PublicKey,
    pub last_sync: Timestamp,
}

impl FullSyncTarget {
    const TABLE_NAME: &'static str = "full_sync_targets";

    pub fn new(pub_key: PublicKey, last_sync: Timestamp) -> Self {
        Self { pub_key, last_sync }
    }

    pub fn from_user(user: &User) -> Self {
        Self {
            pub_key: user.pub_key().clone(),
            last_sync: Timestamp::new(0),
        }
    }
}

#[cfg(feature = "surrealdb")]
#[skerry]
impl Repositories {
    const COVER_BUCKET: &'static str = "covers";

    /// Use Repositories::initialize() instead, this function is only so we can
    /// run tests without setting a user and in memory
    pub async fn setup(db: Surreal<Db>) -> Self {
        db.use_ns("akareko").use_db("main").await.unwrap();

        let mut init_query = String::new();

        for table in [
            MangaTag::TAG,
            MangaTag::CONTENT_TABLE,
            &IndexFollow::<MangaTag>::table_name(),
            User::TABLE_NAME,
            Post::TABLE_NAME,
            FullSyncTarget::TABLE_NAME,
            "events",
        ] {
            init_query.push_str(&format!("DEFINE TABLE IF NOT EXISTS {};\n", table));
        }

        let cwd = env::current_dir().expect("Failed to get current working directory");
        let cwd_str = cwd
            .to_str()
            .expect("Path contains invalid UTF-8")
            .to_string()
            + "/database/surreal/buckets/covers";
        unsafe {
            env::set_var("SURREAL_BUCKET_FOLDER_ALLOWLIST", &cwd_str);
        }

        init_query.push_str(&format!(
            "DEFINE INDEX IF NOT EXISTS eventStamps ON TABLE events FIELDS timestamp, event_type;
            DEFINE BUCKET IF NOT EXISTS {} BACKEND \"file:{}\";",
            Self::COVER_BUCKET,
            cwd_str
        ));

        db.query(init_query).await.unwrap();

        Self { db }
    }

    pub async fn in_memory() -> Self {
        let db: Surreal<Db> = Surreal::new::<surrealdb::engine::local::Mem>(())
            .await
            .unwrap();
        Self::setup(db).await
    }

    pub async fn initialize(config: &AkarekoConfig) -> Self {
        let capabilities = Capabilities::default().with_all_experimental_features_allowed();
        let surreal_config = Config::default().capabilities(capabilities);

        let db: Surreal<Db> = Surreal::new::<SurrealKv>(("./database/surreal", surreal_config))
            .await
            .unwrap();

        info!("Initializing SurrealDB");
        let repositories = Self::setup(db).await;
        info!("Initialized SurrealDB");

        {
            let user_repository = repositories.user();
            match user_repository.get_user(&config.public_key()).await {
                Err(_) => {
                    use crate::db::user::TrustLevel;

                    let mut user = User::new_signed(
                        "Anon".to_string(),
                        Timestamp::now(),
                        &config.private_key(),
                        config.eepsite_address().clone(),
                    );
                    user.set_trust(TrustLevel::Ignore);
                    user_repository.upsert_user(user).await.unwrap();
                }
                _ => {}
            }
        }

        repositories
    }

    pub async fn upsert_full_sync_address(
        &self,
        target: FullSyncTarget,
    ) -> Result<(), DatabaseError> {
        use surrealdb_types::Value;

        let _: Vec<Value> = self
            .db
            .upsert(FullSyncTarget::TABLE_NAME)
            .content(target)
            .await?;

        Ok(())
    }

    pub async fn remove_full_sync_address(&self, pub_key: PublicKey) -> Result<(), e![Surreal]> {
        use surrealdb_types::{RecordId, Value};
        let _: Option<Value> = self
            .db
            .delete(RecordId::new(
                FullSyncTarget::TABLE_NAME,
                pub_key.to_base64(),
            ))
            .await?;
        Ok(())
    }

    pub async fn full_sync_addresses(&self) -> Result<Vec<FullSyncTarget>, e![Surreal]> {
        let addresses: Vec<FullSyncTarget> = self.db.select(FullSyncTarget::TABLE_NAME).await?;
        Ok(addresses)
    }

    pub async fn save_cover(&self, key: &str, bytes: &[u8]) -> Result<(), e![Surreal]> {
        // TODO: Find a way to not have to clone bytes, not sure if surrealdb allows it
        let x = self
            .db
            .query(format!("f\"{}:/{}\".put($data);", Self::COVER_BUCKET, key))
            .bind(("data", Bytes::copy_from_slice(bytes)))
            .await?;

        Ok(())
    }

    pub async fn retrieve_cover(&self, key: &str) -> Result<Bytes, e![Surreal, NotFound]> {
        let mut response = self
            .db
            .query(format!("f\"{}:/{}\".get();", Self::COVER_BUCKET, key))
            .await?;

        if let Some(surrealdb_types::Value::Bytes(bytes)) =
            response.take::<Option<surrealdb_types::Value>>(0)?
        {
            Ok(bytes.into_inner())
        } else {
            Err(RepositoriesRetrieveCoverError::NotFound)
        }
    }

    pub fn user(&self) -> UserRepository<'_> {
        UserRepository::new(&self.db)
    }

    pub fn index(&self) -> IndexRepository<'_> {
        IndexRepository::new(&self.db)
    }

    pub fn index_follow(&self) -> IndexFollowRepository<'_> {
        IndexFollowRepository::new(&self.db)
    }
}

#[cfg(feature = "surrealdb")]
mod surreal {
    use serde::{Deserialize, Serialize};
    use std::marker::PhantomData;
    use surrealdb_types::SurrealValue;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SurrealPhantom<T>(PhantomData<T>);

    impl<T> Default for SurrealPhantom<T> {
        fn default() -> Self {
            Self(Default::default())
        }
    }

    impl<T> SurrealValue for SurrealPhantom<T> {
        fn kind_of() -> surrealdb_types::Kind {
            surrealdb_types::Kind::None
        }

        fn into_value(self) -> surrealdb_types::Value {
            surrealdb_types::Value::None
        }

        fn from_value(_: surrealdb_types::Value) -> Result<Self, surrealdb::Error>
        where
            Self: Sized,
        {
            return Ok(SurrealPhantom(PhantomData));
        }
    }
}
#[cfg(feature = "surrealdb")]
pub use surreal::*;
