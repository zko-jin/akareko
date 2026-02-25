use std::fmt::Debug;

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

use crate::db::{comments::PostRepository, follow_index::IndexFollowRepository};
#[cfg(feature = "surrealdb")]
use crate::errors::DatabaseError;
use crate::{
    config::AuroraConfig,
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
pub mod follow_index;
pub mod index;
pub mod user;

pub type Timestamp = u64;

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

#[derive(Debug, Clone)]
pub struct Repositories {
    #[cfg(feature = "surrealdb")]
    pub db: Surreal<Db>,
    #[cfg(feature = "sqlite")]
    pub db: Pool,
    config: Arc<RwLock<AuroraConfig>>,
}

#[cfg(feature = "sqlite")]
impl Repositories {
    pub async fn initialize(config: Arc<RwLock<AuroraConfig>>) -> Self {}

    pub async fn get_random_contents(
        &self,
        count: u16,
    ) -> Result<Vec<TaggedContent>, DatabaseError> {
        // let mut tagged_contents = Vec::with_capacity(count as usize);

        let conn = self.db.get().await.unwrap();
        let result: i64 = conn
            .interact(|conn| {
                let mut stmt = conn.prepare("SELECT 1")?;
                let mut rows = stmt.query([])?;
                let row = rows.next()?.unwrap();
                row.get(0)
            })
            .await
            .unwrap()
            .unwrap();
        todo!()
        // let novels: Vec<Content<NovelTag>> = conn.

        // .prepare("SELECT * FROM novels ORDER BY RANDOM() LIMIT ?1")
        // .unwrap()
        // .query_map([novel_tag], |row| Ok(TaggedContent::from(row.get(0)?)))
        // .unwrap()
        // .take(0)?;

        // tagged_contents.extend(novels.into_iter().map(TaggedContent::from));

        // Ok(tagged_contents)
    }

    pub async fn user(&self) -> UserRepository {
        UserRepository::new(self.db.get().await.unwrap())
    }

    pub async fn index(&self) -> IndexRepository {
        IndexRepository::new(self.db.get().await.unwrap())
    }
}

const CACHE_TABLE: &'static str = "cache";
fn cache_index_key(index_hash: &Hash, user: &PublicKey) -> String {
    format!("{}_{}", index_hash.as_base64(), user.to_base64())
}

#[cfg(feature = "surrealdb")]
impl Repositories {
    pub async fn initialize(config: Arc<RwLock<AuroraConfig>>) -> Self {
        info!("Initializing SurrealDB");
        let db = Surreal::new::<SurrealKv>("database").await.unwrap();
        db.use_ns("aurora").use_db("main").await.unwrap();
        let repositories = Repositories { db, config };
        info!("Initialized SurrealDB");

        {
            let config = repositories.config.read().await;

            let user_repository = repositories.user().await;
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
                    dbg!(res);
                }
                _ => {}
            }
        }

        repositories
    }

    // pub async fn get_random_contents(
    //     &self,
    //     count: u16,
    // ) -> Result<Vec<TaggedContent>, DatabaseError> {
    //     let mut tagged_contents = Vec::with_capacity(count as usize);

    //     let novel_tag = count;

    //     let novels: Vec<Content<MangaTag>> = self
    //         .db
    //         .query(format!(
    //             "SELECT * FROM {} ORDER BY rand() LIMIT $count",
    //             MangaTag::CONTENT_TABLE
    //         ))
    //         .bind(("count", novel_tag))
    //         .await?
    //         .take(0)?;

    //     tagged_contents.extend(novels.into_iter().map(TaggedContent::from));

    //     Ok(tagged_contents)
    // }

    pub async fn add_cache_index(
        &self,
        index_hash: &Hash,
        user: &PublicKey,
        timestamp: Timestamp,
    ) -> Result<(), DatabaseError> {
        let _: Option<surrealdb_types::Value> = self
            .db
            .upsert(surrealdb_types::RecordId::new(
                CACHE_TABLE,
                cache_index_key(index_hash, user),
            ))
            .content(timestamp)
            .await?;

        Ok(())
    }

    pub async fn check_cache_index(
        &self,
        index_hash: &Hash,
        user: &PublicKey,
    ) -> Result<Timestamp, DatabaseError> {
        const QUERY: &'static str =
            const_format::formatcp!("SELECT * FROM {}:$cache;", CACHE_TABLE);

        let results: Vec<Timestamp> = self
            .db
            .query(QUERY)
            .bind(("cache", cache_index_key(index_hash, user)))
            .await?
            .take(0)?;

        match results.len() {
            0 => Ok(0),
            1 => Ok(results.into_iter().next().unwrap()),
            _ => Err(DatabaseError::Unknown),
        }
    }

    pub async fn user(&self) -> UserRepository<'_> {
        UserRepository::new(&self.db)
    }

    pub async fn index(&self) -> IndexRepository<'_> {
        IndexRepository::new(&self.db)
    }

    pub async fn posts(&self) -> PostRepository<'_> {
        PostRepository::new(&self.db)
    }

    pub async fn index_follow(&self) -> IndexFollowRepository<'_> {
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
