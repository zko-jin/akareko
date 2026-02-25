use surrealdb_types::SurrealValue;

use crate::{
    db::{SurrealPhantom, index::tags::IndexTag},
    hash::Hash,
};

#[cfg(feature = "surrealdb")]
mod surreal;
#[cfg(feature = "surrealdb")]
pub use surreal::IndexFollowRepository;

#[derive(Debug, Clone, SurrealValue)]
pub struct IndexFollow<T: IndexTag> {
    index: Hash,
    notify: bool,
    _phantom: SurrealPhantom<T>,
}

impl<T: IndexTag> IndexFollow<T> {
    pub fn table_name() -> String {
        format!("{}_follow", T::TAG)
    }

    pub fn new(index: Hash, notify: bool) -> Self {
        Self {
            index,
            notify,
            _phantom: SurrealPhantom::default(),
        }
    }
}
