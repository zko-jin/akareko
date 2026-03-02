use std::fmt::Debug;
#[cfg(feature = "surrealdb")]
use std::path::Path;

#[cfg(feature = "diesel")]
use diesel::SqliteConnection;
#[cfg(feature = "diesel")]
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, bb8::Pool};
#[cfg(feature = "diesel")]
use diesel_async::sync_connection_wrapper::SyncConnectionWrapper;
use rclite::Arc;

use serde::{Deserialize, Serialize};
use surrealdb::{
    Surreal,
    engine::local::{Db, SurrealKv},
};
use surrealdb_types::SurrealValue;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::RwLock,
};
use tracing::info;

use crate::db::user::I2PAddress;
use crate::db::{
    comments::Post,
    follow_index::IndexFollow,
    index::tags::{IndexTag, MangaTag},
};
#[cfg(feature = "surrealdb")]
use crate::db::{comments::PostRepository, follow_index::IndexFollowRepository};
// use crate::db::{comments::PostRepository, follow_index::IndexFollowRepository};
use crate::errors::DatabaseError;
use crate::{
    config::AkarekoConfig,
    db::{
        index::IndexRepository,
        user::{User, UserRepository},
    },
    errors::{DecodeError, EncodeError},
    helpers::{Byteable, now_timestamp},
};
use crate::{
    db::index::content::Content,
    hash::{Hash, PublicKey},
};

// ==================== End Imports ====================

pub mod comments;
pub mod event;
pub mod follow_index;
pub mod index;
pub mod schedule;
#[cfg(feature = "diesel")]
pub mod schema;
pub mod user;

pub type Timestamp = u64;
pub const BLOOM_FILTER_FALSE_POSITIVE_RATE: f64 = 0.0001;

pub struct PaginateSearch<T> {
    search: T,
    take: usize,
    skip: usize,
}

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

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
#[serde(transparent)]
pub struct Magnet(pub String);

impl Byteable for Magnet {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        self.0.encode(writer).await
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        Ok(Magnet(String::decode(reader).await?))
    }
}

#[cfg(feature = "sqlite")]
type Connection = SyncConnectionWrapper<SqliteConnection>;

#[cfg(feature = "sqlite")]
type DbPool = Pool<Connection>;

#[derive(Clone)]
pub struct Repositories {
    #[cfg(feature = "surrealdb")]
    pub db: Surreal<Db>,
    #[cfg(feature = "sqlite")]
    pub db: DbPool,
}

impl std::fmt::Debug for Repositories {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Repositories").finish()
    }
}

#[cfg(feature = "sqlite")]
impl Repositories {
    pub async fn initialize(config: Arc<RwLock<AkarekoConfig>>) -> Self {
        use diesel_async::{AsyncConnection, pooled_connection::AsyncDieselConnectionManager};

        let manager = AsyncDieselConnectionManager::new("./database/sqlite.db");
        let db = DbPool::builder().build(manager).await.unwrap();

        Self { db }
    }

    pub fn user(&self) -> UserRepository {
        UserRepository::new(self.db.clone())
    }

    pub fn index(&self) -> IndexRepository {
        IndexRepository::new(self.db.clone())
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
            last_sync: 0,
        }
    }
}

#[cfg(feature = "surrealdb")]
impl Repositories {
    /// Use Repositories::initialize() instead, this function is only so we can run tests
    /// without setting a user and in memory
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

        init_query.push_str(
            "DEFINE INDEX IF NOT EXISTS eventStamps ON TABLE events FIELDS timestamp, event_type;",
        );

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
        let db: Surreal<Db> = Surreal::new::<SurrealKv>("./database/surreal")
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
                        now_timestamp(),
                        &config.private_key(),
                        config.eepsite_address().clone(),
                    );
                    user.set_trust(TrustLevel::Ignore);
                    let res = user_repository.upsert_user(user).await.unwrap();
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

    pub async fn remove_full_sync_address(&self, pub_key: PublicKey) -> Result<(), DatabaseError> {
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

    pub async fn full_sync_addresses(&self) -> Result<Vec<FullSyncTarget>, DatabaseError> {
        let addresses: Vec<FullSyncTarget> = self.db.select(FullSyncTarget::TABLE_NAME).await?;
        Ok(addresses)
    }

    pub fn user(&self) -> UserRepository<'_> {
        UserRepository::new(&self.db)
    }

    pub fn index(&self) -> IndexRepository<'_> {
        IndexRepository::new(&self.db)
    }

    pub fn posts(&self) -> PostRepository<'_> {
        PostRepository::new(&self.db)
    }

    pub fn index_follow(&self) -> IndexFollowRepository<'_> {
        IndexFollowRepository::new(&self.db)
    }
}

#[cfg(feature = "surrealdb")]
mod surreal {
    use std::marker::PhantomData;
    use surrealdb_types::SurrealValue;

    #[derive(Debug, Clone)]
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
