use crate::{
    db::{
        index::{
            Index,
            tags::{IndexTag, MangaTag, NoTag},
        },
        user::I2PAddress,
    },
    hash::Hash,
    server::{ServerState, handler::AuroraProtocolCommand, protocol::AuroraProtocolResponse},
};

pub struct GetIndexes;

impl AuroraProtocolCommand for GetIndexes {
    type RequestPayload = GetIndexesRequest;
    type ResponsePayload = GetIndexesResponse;
    type ResponseData = Index<NoTag>;

    async fn process(
        req: Self::RequestPayload,
        state: &ServerState,
        _: &I2PAddress,
    ) -> AuroraProtocolResponse<Self::ResponsePayload, Self::ResponseData> {
        match req.tag.as_str() {
            MangaTag::TAG => {
                let indexes = match state
                    .repositories
                    .index()
                    .await
                    .get_indexes::<MangaTag>(&req.indexes)
                    .await
                {
                    Ok(i) => i,
                    Err(_) => {
                        return AuroraProtocolResponse::internal_error(format!("Database error"));
                    }
                };

                // SAFETY: They are all the same type, just different tags
                AuroraProtocolResponse::ok_with_data(GetIndexesResponse {}, unsafe {
                    std::mem::transmute(indexes)
                })
            }
            _ => AuroraProtocolResponse::invalid_argument(format!("Invalid tag: {}", req.tag)),
        }
    }
}

#[derive(byteable_derive::Byteable)]
pub struct GetIndexesRequest {
    tag: String,
    indexes: Vec<Hash>,
}

impl GetIndexesRequest {
    pub fn new<T: IndexTag>(indexes: Vec<Hash>) -> Self {
        Self {
            tag: T::TAG.to_string(),
            indexes,
        }
    }
}

#[derive(byteable_derive::Byteable)]
pub struct GetIndexesResponse {}
