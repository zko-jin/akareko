use crate::{
    db::{
        index::{
            Index,
            tags::{IndexTag, MangaTag, NoTag},
        },
        user::I2PAddress,
    },
    hash::Hash,
    server::{ServerState, handler::AkarekoProtocolCommand, protocol::AkarekoProtocolResponse},
};

pub struct GetIndexes;

impl AkarekoProtocolCommand for GetIndexes {
    type RequestPayload = GetIndexesRequest;
    type ResponsePayload = GetIndexesResponse;
    type ResponseData = Index<NoTag>;

    async fn process(
        req: Self::RequestPayload,
        state: &ServerState,
        _: &I2PAddress,
    ) -> AkarekoProtocolResponse<Self::ResponsePayload, Self::ResponseData> {
        match req.tag.as_str() {
            MangaTag::TAG => {
                let indexes = match state
                    .repositories
                    .index()
                    .get_indexes::<MangaTag>(&req.indexes)
                    .await
                {
                    Ok(i) => i,
                    Err(_) => {
                        return AkarekoProtocolResponse::internal_error(format!("Database error"));
                    }
                };

                // SAFETY: They are all the same type, just different tags
                AkarekoProtocolResponse::ok_with_data(GetIndexesResponse {}, unsafe {
                    std::mem::transmute(indexes)
                })
            }
            _ => AkarekoProtocolResponse::invalid_argument(format!("Invalid tag: {}", req.tag)),
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
