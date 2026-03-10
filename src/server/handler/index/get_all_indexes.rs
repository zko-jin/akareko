use fastbloom::BloomFilter;

use crate::{
    db::{
        index::{
            Index,
            tags::{IndexTag, MangaTag, NoTag},
        },
        user::I2PAddress,
    },
    errors::{DecodeError, EncodeError},
    server::{ServerState, handler::AkarekoProtocolCommand, protocol::AkarekoProtocolResponse},
    types::Timestamp,
};

pub struct GetAllIndexes<I: IndexTag>(std::marker::PhantomData<I>);

impl<I: IndexTag> AkarekoProtocolCommand for GetAllIndexes<I> {
    type RequestPayload = GetAllIndexesRequest;
    type ResponsePayload = GetAllIndexesResponse;
    type ResponseData = Index<I>;

    async fn process(
        req: Self::RequestPayload,
        state: &ServerState,
        _: &I2PAddress,
    ) -> AkarekoProtocolResponse<Self::ResponsePayload, Self::ResponseData> {
        let indexes = match state
            .repositories
            .index()
            .get_all_indexes::<I>(req.timestamp, req.filter)
            .await
        {
            Ok(indexes) => indexes,
            Err(_) => {
                return AkarekoProtocolResponse::internal_error(format!("Database error"));
            }
        };

        AkarekoProtocolResponse::ok_with_data(GetAllIndexesResponse {}, indexes)
    }
}

#[derive(byteable_derive::Byteable)]
pub struct GetAllIndexesRequest {
    tag: String,
    /// Get indexes created_updated after this timestamp
    timestamp: Option<Timestamp>,
    filter: Option<BloomFilter>,
}

impl GetAllIndexesRequest {
    pub fn new<T: IndexTag>(timestamp: Option<Timestamp>, filter: Option<BloomFilter>) -> Self {
        Self {
            tag: T::TAG.to_string(),
            timestamp,
            filter,
        }
    }
}

#[derive(byteable_derive::Byteable)]
pub struct GetAllIndexesResponse {}
