use surrealdb_types::SurrealValue;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

use crate::types::Hash;

#[derive(Debug, Default, Clone, SurrealValue, byteable_derive::Byteable)]
pub enum IndexStatus {
    Completed,
    Hiatus,
    Cancelled,
    Releasing,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, SurrealValue, byteable_derive::Byteable)]
pub struct IndexMetadata {
    hash: Hash, // Primary Key
    pub status: IndexStatus,
}
