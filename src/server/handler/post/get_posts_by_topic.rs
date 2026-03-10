use fastbloom::BloomFilter;

use crate::{
    db::{comments::Post, user::I2PAddress},
    server::{ServerState, handler::AkarekoProtocolCommand, protocol::AkarekoProtocolResponse},
    types::{Timestamp, Topic},
};

pub struct GetPostsByTopic;

impl AkarekoProtocolCommand for GetPostsByTopic {
    type RequestPayload = GetPostsByTopicRequest;
    type ResponsePayload = GetPostsByTopicResponse;
    type ResponseData = Post;

    async fn process(
        req: Self::RequestPayload,
        state: &ServerState,
        _address: &I2PAddress,
    ) -> AkarekoProtocolResponse<Self::ResponsePayload, Self::ResponseData> {
        let Ok(posts) = state
            .repositories
            .get_filtered_posts_by_topic(req.topic, req.timestamp, req.filter)
            .await
        else {
            return AkarekoProtocolResponse::internal_error("Database error".to_string());
        };

        AkarekoProtocolResponse::ok_with_data(GetPostsByTopicResponse {}, posts)
    }
}

#[derive(byteable_derive::Byteable)]
pub struct GetPostsByTopicRequest {
    pub topic: Topic,
    pub timestamp: Option<Timestamp>,
    pub filter: Option<BloomFilter>,
}

#[derive(byteable_derive::Byteable)]
pub struct GetPostsByTopicResponse {}
