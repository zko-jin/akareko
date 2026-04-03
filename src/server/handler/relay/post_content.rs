use std::marker::PhantomData;

use fastbloom::BloomFilter;

use crate::{
    db::{
        index::{
            content::Content,
            tags::{IndexTag, MangaTag},
        },
        user::I2PAddress,
    },
    server::{ServerState, handler::AkarekoProtocolCommand, protocol::AkarekoProtocolResponse},
};

pub struct SendContent<I: IndexTag>(PhantomData<I>);

impl<I: IndexTag + 'static> AkarekoProtocolCommand for SendContent<I> {
    type RequestPayload = PostContentRequest<I>;
    type ResponsePayload = PostContentResponse;
    type ResponseData = ();

    async fn process(
        req: Self::RequestPayload,
        state: &ServerState,
        _: &I2PAddress,
    ) -> AkarekoProtocolResponse<Self::ResponsePayload, Self::ResponseData> {
        if !req.content.verify() {
            return AkarekoProtocolResponse::invalid_argument("Signature is not valid".to_string());
        }

        match state.repositories.index().add_content(req.content).await {
            Ok(_) => {}
            Err(_) => return AkarekoProtocolResponse::internal_error("Database error".to_string()),
        };

        AkarekoProtocolResponse::ok(PostContentResponse {})
    }
}

#[derive(byteable_derive::Byteable)]
pub struct PostContentRequest<I: IndexTag> {
    pub content: Content<I>,
}

#[derive(byteable_derive::Byteable)]
pub struct PostContentResponse {}
