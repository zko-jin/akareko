use std::marker::PhantomData;

use fastbloom::BloomFilter;

use crate::{
    db::{
        index::{content::Content, tags::IndexTag},
        user::I2PAddress,
    },
    server::{ServerState, handler::AkarekoProtocolCommand, protocol::AkarekoProtocolResponse},
    types::{Hash, Timestamp},
};

pub struct GetContents<I: IndexTag>(PhantomData<I>);

impl<I: IndexTag> AkarekoProtocolCommand for GetContents<I> {
    type RequestPayload = GetContentsRequest;
    type ResponsePayload = GetContentsResponse;
    type ResponseData = Content<I>;

    async fn process(
        req: Self::RequestPayload,
        state: &ServerState,
        _: &I2PAddress,
    ) -> AkarekoProtocolResponse<Self::ResponsePayload, Self::ResponseData> {
        let contents = match state
            .repositories
            .index()
            .get_filtered_index_contents::<I>(req.index, req.after, req.filter)
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
    after: Option<Timestamp>,
    filter: Option<BloomFilter>,
}

impl GetContentsRequest {
    pub fn new(index: Hash, after: Option<Timestamp>, filter: Option<BloomFilter>) -> Self {
        Self {
            index,
            after,
            filter,
        }
    }
}

#[derive(byteable_derive::Byteable)]
pub struct GetContentsResponse {}
