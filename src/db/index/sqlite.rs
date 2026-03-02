use std::marker::PhantomData;

use diesel::{
    ExpressionMethods, Insertable, QueryDsl, Queryable, QueryableByName, Selectable,
    SelectableHelper,
};
use diesel_async::RunQueryDsl;
use fastbloom::BloomFilter;
use futures::TryStreamExt;
use tracing::info;

use crate::db::index::Index;
use crate::db::index::tags::MangaTag;
use crate::db::{Content, DbPool, IndexTag, Timestamp};
use crate::errors::DatabaseError;
use crate::hash::{Hash, PublicKey, Signature};

pub struct IndexRepository<T: IndexTag>(DbPool, PhantomData<T>);

impl<T: IndexTag> IndexRepository<T> {
    pub fn new(conn: DbPool) -> IndexRepository<T> {
        IndexRepository(conn, PhantomData)
    }
}

/// Used because diesel hates PhantomData for some reason, there's no #[diesel(skip)], only
/// #[diesel(skip_insertion)]
#[derive(Insertable, Queryable, QueryableByName, Selectable)]
#[diesel(table_name = crate::db::schema::mangas)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct UntaggedIndex {
    hash: Hash, // Primary Key
    title: String,
    release_date: i32,
    source: PublicKey,
    received_at: Timestamp,
    signature: Signature,
}

impl<T: IndexTag> From<Index<T>> for UntaggedIndex {
    fn from(index: Index<T>) -> Self {
        // SAFETY: Same type, just missing PhantomData
        unsafe { std::mem::transmute(index) }
    }
}

impl<T: IndexTag> From<UntaggedIndex> for Index<T> {
    fn from(index: UntaggedIndex) -> Self {
        // SAFETY: Same type, just missing PhantomData
        unsafe { std::mem::transmute(index) }
    }
}

impl IndexRepository<MangaTag> {
    pub async fn add_index(&self, index: Index<MangaTag>) -> Result<(), DatabaseError> {
        use crate::db::schema::mangas::dsl::*;

        let index: UntaggedIndex = index.into();

        let mut conn = self.0.get().await.unwrap();
        // TODO: Use on_conflict() later
        diesel::insert_into(mangas)
            .values(&index)
            .execute(&mut conn)
            .await?;

        Ok(())
    }

    pub async fn add_content(
        &self,
        content: Content<MangaTag>,
    ) -> Result<Content<MangaTag>, DatabaseError> {
        let created: Result<Option<Content<T>>, surrealdb::Error> = self
            .db
            .upsert((T::CONTENT_TABLE, content.signature().as_base64()))
            .content(content)
            .await;

        match created {
            Ok(n) => match n {
                Some(n) => Ok(n),
                None => Err(DatabaseError::Unknown),
            },
            Err(e) => {
                info!("Error: {}", e);
                Err(DatabaseError::Unknown)
            }
        }
    }

    pub async fn get_all_indexes<T: IndexTag>(
        &self,
        timestamp: Timestamp,
        filter: Option<BloomFilter>,
    ) -> Result<Vec<Index<T>>, DatabaseError> {
        use crate::db::schema::mangas::dsl::*;

        let mut conn = self.0.get().await.unwrap();

        let mut query = mangas.into_boxed();
        if timestamp == 0 {
            query = query.filter(received_at.ge(timestamp));
        }

        let result = query
            .select(UntaggedIndex::as_select())
            .load_stream::<UntaggedIndex>(&mut conn)
            .await?
            .try_fold(Vec::new(), |mut acc, item| {
                if let Some(filter) = &filter {
                    if !filter.contains(&item.hash) {
                        acc.push(item.into());
                        return futures::future::ready(Ok(acc));
                    }
                }
                futures::future::ready(Ok(acc))
            })
            .await?;

        Ok(result)
    }

    pub async fn get_indexes<T: IndexTag>(
        &self,
        hashes: &[Hash],
    ) -> Result<Vec<Index<T>>, DatabaseError> {
        use crate::db::schema::mangas::dsl::*;

        let mut conn = self.0.get().await.unwrap();

        let result = mangas
            .filter(hash.eq_any(hashes))
            .select(UntaggedIndex::as_select())
            .load(&mut conn)
            .await?;

        let result = unsafe { std::mem::transmute(result) };
        Ok(result)
    }

    pub async fn get_index<T: IndexTag>(
        &self,
        index_hash: &Hash,
    ) -> Result<Option<Index<T>>, DatabaseError> {
        use crate::db::schema::mangas::dsl::*;

        let mut conn = self.0.get().await.unwrap();

        let result = match mangas
            .filter(hash.eq(index_hash))
            .select(UntaggedIndex::as_select())
            .first(&mut conn)
            .await
        {
            Ok(i) => Some(i.into()),
            Err(e) => {
                if e == diesel::result::Error::NotFound {
                    None
                } else {
                    return Err(e.into());
                }
            }
        };

        Ok(result)
    }

    pub async fn get_contents<T: IndexTag>(
        &self,
        index_hash: Hash,
    ) -> Result<Vec<Content<T>>, DatabaseError> {
        let query: String = format!(
            "SELECT * FROM {} WHERE index_hash = $index_hash",
            T::CONTENT_TABLE
        );

        let chapters: Vec<Content<T>> = self
            .db
            .query(query)
            .bind(("index_hash", index_hash))
            .await?
            .take(0)?;

        Ok(chapters)
    }
}

#[cfg(test)]
mod tests {
    use crate::{db::index::tags::NoTag, hash::PrivateKey};

    use super::*;

    #[test]
    fn untagged_index_transmute() {
        let title = "test";
        let release_date = 0;
        let key = PrivateKey::new().public_key();
        let signature = Signature::empty();

        let index: Index<NoTag> = Index::new(
            title.to_string(),
            release_date,
            key.clone(),
            signature.clone(),
        );

        let hash = index.hash().clone();
        let received_at = index.received_at;

        let untagged_index = UntaggedIndex::from(index);

        assert_eq!(untagged_index.hash, hash);
        assert_eq!(untagged_index.title, title);
        assert_eq!(untagged_index.release_date, release_date);
        assert_eq!(untagged_index.source, key);
        assert_eq!(untagged_index.received_at, received_at);
        assert_eq!(untagged_index.signature, signature);
    }
}
