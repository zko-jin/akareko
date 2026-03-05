use std::marker::PhantomData;

use fastbloom::BloomFilter;

use crate::{
    db::{
        Timestamp,
        index::{
            content::Content,
            tags::{IndexTag, MangaTag},
        },
        user::I2PAddress,
    },
    hash::Hash,
    server::{ServerState, handler::AkarekoProtocolCommand, protocol::AkarekoProtocolResponse},
};

pub struct GetContents<I: IndexTag>(PhantomData<I>);

impl AkarekoProtocolCommand for GetContents<MangaTag> {
    type RequestPayload = GetContentsRequest;
    type ResponsePayload = GetContentsResponse;
    type ResponseData = Content<MangaTag>;

    async fn process(
        req: Self::RequestPayload,
        state: &ServerState,
        _: &I2PAddress,
    ) -> AkarekoProtocolResponse<Self::ResponsePayload, Self::ResponseData> {
        let contents = match state
            .repositories
            .index()
            .get_filtered_index_contents::<MangaTag>(req.index, req.after, req.filter)
            .await
        {
            Ok(c) => c,
            Err(_) => {
                return AkarekoProtocolResponse::internal_error(format!("Database error"));
            }
        };

        AkarekoProtocolResponse::ok_with_data(GetContentsResponse {}, contents)
    }
}

#[derive(byteable_derive::Byteable)]
pub struct GetContentsRequest {
    index: Hash,
    /// Get indexes created_updated after this timestamp
    after: Timestamp,
    filter: Option<BloomFilter>,
}

impl GetContentsRequest {
    pub fn new(index: Hash, after: Timestamp, filter: Option<BloomFilter>) -> Self {
        Self {
            index,
            after,
            filter,
        }
    }
}

#[derive(byteable_derive::Byteable)]
pub struct GetContentsResponse {}
