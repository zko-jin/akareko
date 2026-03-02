use diesel::{
    associations::HasTable,
    deserialize::{FromSql, FromSqlRow},
    insert_into, no_arg_sql_function,
    prelude::*,
    upsert::excluded,
};
use diesel_async::{
    RunQueryDsl,
    pooled_connection::{AsyncDieselConnectionManager, bb8::PooledConnection},
};
use rand::seq::{IteratorRandom, SliceRandom};
use tracing::info;

use crate::{
    db::{Connection, DbPool, user::TrustLevel},
    errors::DatabaseError,
    hash::PublicKey,
};

use super::User;

pub struct UserRepository(DbPool);

impl UserRepository {
    pub fn new(pool: DbPool) -> UserRepository {
        UserRepository(pool)
    }
}

#[cfg(feature = "sqlite")]
pub mod sqlite {
    use diesel::{
        deserialize::FromSql,
        serialize::{self, IsNull, Output, ToSql},
        sql_types::{Binary, Integer, Text},
        sqlite::{Sqlite, SqliteValue},
    };

    use crate::{
        db::user::{I2PAddress, TrustLevel},
        hash::{PublicKey, Signature},
    };

    impl FromSql<Integer, Sqlite> for TrustLevel {
        fn from_sql(bytes: SqliteValue) -> diesel::deserialize::Result<Self> {
            let value = <i32 as FromSql<Integer, Sqlite>>::from_sql(bytes)?;
            match value.try_into() {
                Ok(trust_level) => Ok(trust_level),
                Err(e) => Err(format!("Invalid TrustLevel value: {}", e).into()),
            }
        }
    }

    impl ToSql<Integer, Sqlite> for TrustLevel {
        fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
            out.set_value(*self as i32);
            Ok(IsNull::No)
        }
    }

    impl FromSql<Text, Sqlite> for I2PAddress {
        fn from_sql(bytes: SqliteValue) -> diesel::deserialize::Result<Self> {
            let value = <String as FromSql<Text, Sqlite>>::from_sql(bytes)?;
            Ok(I2PAddress::new(value))
        }
    }

    impl ToSql<Text, Sqlite> for I2PAddress {
        fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
            out.set_value(self.inner().as_str());
            Ok(IsNull::No)
        }
    }
}

#[declare_sql_function]
extern "SQL" {
    fn random() -> Text;
}

impl UserRepository {
    pub async fn upsert_user(&self, user: User) -> Result<(), DatabaseError> {
        use crate::db::schema::users;

        let mut conn = self.0.get().await.unwrap();

        diesel::insert_into(users::table)
            .values(&user)
            .on_conflict(users::pub_key)
            .filter_target(excluded(users::timestamp.gt(users::timestamp)))
            .do_update()
            .set((
                users::name.eq(excluded(users::name)),
                users::timestamp.eq(excluded(users::timestamp)),
                users::signature.eq(excluded(users::signature)),
                users::address.eq(excluded(users::address)),
                users::trust.eq(excluded(users::trust)),
            ))
            .execute(&mut conn)
            .await;

        Ok(())
    }

    pub async fn get_users(&self, pub_keys: Vec<PublicKey>) -> Result<Vec<User>, DatabaseError> {
        use crate::db::schema::users::dsl::*;

        let mut conn = self.0.get().await.unwrap();

        let results: Vec<User> = users
            .filter(pub_key.eq_any(pub_keys))
            .select(User::as_select())
            .load(&mut conn)
            .await?;

        Ok(results)
    }

    pub async fn get_random_users(
        &self,
        min_trust: TrustLevel,
        take: u32,
    ) -> Result<Vec<User>, DatabaseError> {
        use crate::db::schema::users::dsl::*;

        let mut conn = self.0.get().await.unwrap();

        let results: Vec<User> = users
            .filter(trust.ge(min_trust as i32))
            .order(random())
            .limit(take as i64) // This is a bit weird, for some reason diesel takes an i64
            .load(&mut conn)
            .await?;

        Ok(results)
    }

    pub async fn get_all_users(&mut self) -> Result<Vec<User>, DatabaseError> {
        use crate::db::schema::users::dsl::*;
        let mut conn = self.0.get().await.unwrap();
        let result: Vec<User> = users.select(User::as_select()).load(&mut conn).await?;

        Ok(result)
    }

    pub async fn get_user(&self, key: &PublicKey) -> Result<Option<User>, DatabaseError> {
        use crate::db::schema::users::dsl::*;
        let mut conn = self.0.get().await.unwrap();
        let result: Vec<User> = users
            .filter(pub_key.eq(key))
            .select(User::as_select())
            .load(&mut conn)
            .await?;

        match result.into_iter().next() {
            Some(user) => Ok(Some(user)),
            None => Ok(None),
        }
    }
}
