use surrealdb_types::SurrealValue;

use crate::{
    db::{SurrealPhantom, Timestamp, index::tags::IndexTag},
    types::Hash,
};

#[cfg(feature = "surrealdb")]
mod surreal;
#[cfg(feature = "surrealdb")]
pub use surreal::IndexFollowRepository;
#[cfg(feature = "sqlite")]
mod sqlite;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "surrealdb", derive(SurrealValue))]
pub struct IndexFollow<T: IndexTag> {
    #[cfg_attr(feature = "surrealdb", surreal(rename = "id"))]
    index: Hash,
    last_check: Timestamp,
    notify: bool,
    _phantom: SurrealPhantom<T>,
}

impl<T: IndexTag> IndexFollow<T> {
    pub fn table_name() -> String {
        format!("{}_follows", T::TAG)
    }

    pub fn new(index: Hash, notify: bool, last_check: Timestamp) -> Self {
        Self {
            index,
            last_check,
            notify,
            _phantom: SurrealPhantom::default(),
        }
    }
}
