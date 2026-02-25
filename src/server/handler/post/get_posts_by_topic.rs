use fastbloom::BloomFilter;

use crate::{
    db::{
        comments::{Post, Topic},
        user::I2PAddress,
    },
    server::{ServerState, handler::AuroraProtocolCommand, protocol::AuroraProtocolResponse},
};

pub struct GetPostsByTopic;

impl AuroraProtocolCommand for GetPostsByTopic {
    type RequestPayload = GetPostsByTopicRequest;
    type ResponsePayload = GetPostsByTopicResponse;
    type ResponseData = Post;

    async fn process(
        req: Self::RequestPayload,
        state: &ServerState,
        _address: &I2PAddress,
    ) -> AuroraProtocolResponse<Self::ResponsePayload, Self::ResponseData> {
        let Ok(posts) = state
            .repositories
            .posts()
            .await
            .get_all_posts_by_topic(req.topic, req.timestamp, req.filter)
            .await
        else {
            return AuroraProtocolResponse::internal_error("Database error".to_string());
        };

        AuroraProtocolResponse::ok_with_data(GetPostsByTopicResponse {}, posts)
    }
}

#[derive(byteable_derive::Byteable)]
pub struct GetPostsByTopicRequest {
    pub topic: Topic,
    pub timestamp: u64,
    pub filter: Option<BloomFilter>,
}

#[derive(byteable_derive::Byteable)]
pub struct GetPostsByTopicResponse {}
