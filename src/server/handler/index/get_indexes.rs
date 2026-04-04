use crate::{
    db::{
        index::{Index, tags::IndexTag},
        user::I2PAddress,
    },
    server::{ServerState, handler::AkarekoProtocolCommand, protocol::AkarekoProtocolResponse},
    types::Hash,
};

pub struct GetIndexes<I: IndexTag>(std::marker::PhantomData<I>);

impl<I: IndexTag> AkarekoProtocolCommand for GetIndexes<I> {
    type RequestPayload = GetIndexesRequest;
    type ResponsePayload = GetIndexesResponse;
    type ResponseData = Index<I>;

    async fn process(
        req: Self::RequestPayload,
        state: &ServerState,
        _: &I2PAddress,
    ) -> AkarekoProtocolResponse<Self::ResponsePayload, Self::ResponseData> {
        let indexes = match state
            .repositories
            .index()
            .get_indexes::<I>(&req.indexes)
            .await
        {
            Ok(i) => i,
            Err(_) => {
                return AkarekoProtocolResponse::internal_error(format!("Database error"));
            }
        };

        AkarekoProtocolResponse::ok_with_data(GetIndexesResponse {}, indexes)
    }
}

#[derive(byteable_derive::Byteable)]
pub struct GetIndexesRequest {
    indexes: Vec<Hash>,
}

impl GetIndexesRequest {
    pub fn new(indexes: Vec<Hash>) -> Self {
        Self { indexes }
    }
}

#[derive(byteable_derive::Byteable)]
pub struct GetIndexesResponse {}
