use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    db::{index::tags::MangaTag, user::I2PAddress},
    errors::{ClientError, DecodeError, EncodeError, ServerError},
    helpers::Byteable,
    server::{
        ServerState,
        protocol::{AkarekoProtocolRequest, AkarekoProtocolResponse, AkarekoProtocolVersion},
    },
};

pub mod index;
mod macros;
pub mod events {
    mod sync_events;
    pub use sync_events::{SyncEvents, SyncEventsRequest};
}
pub mod post {
    mod get_posts_by_topic;
    pub use get_posts_by_topic::{
        GetPostsByTopic, GetPostsByTopicRequest, GetPostsByTopicResponse,
    };
}
pub mod relay {
    mod post_content;
    pub use post_content::{PostContentRequest, PostContentResponse, SendContent};
}
pub mod users;

/// Marker implemented by the handler macro
pub trait CommandEnum: Byteable {}

/// Should be implemented by each command, can be skipped by directly
/// implementing [`AkarekoProtocolCommandHandler`]
pub(super) trait AkarekoProtocolCommand: Sized {
    type RequestPayload: Byteable;
    type ResponsePayload: Byteable;
    type ResponseData: Byteable;

    async fn process(
        req: Self::RequestPayload,
        state: &ServerState,
        address: &I2PAddress,
    ) -> AkarekoProtocolResponse<Self::ResponsePayload, Self::ResponseData>;
}

pub trait AkarekoProtocolCommandRequest<P, R> {
    // Used by the client
    async fn request<S: AsyncRead + AsyncWrite + Unpin + Send>(
        payload: P,
        stream: &mut S,
    ) -> Result<R, ClientError>;
}

impl<T: AkarekoProtocolCommand + AkarekoProtocolCommandMetadata>
    AkarekoProtocolCommandRequest<
        T::RequestPayload,
        AkarekoProtocolResponse<T::ResponsePayload, T::ResponseData>,
    > for T
{
    async fn request<S: AsyncRead + AsyncWrite + Unpin + Send>(
        payload: T::RequestPayload,
        stream: &mut S,
    ) -> Result<AkarekoProtocolResponse<T::ResponsePayload, T::ResponseData>, ClientError> {
        let req = AkarekoProtocolRequest::<Self> { payload };
        req.encode(stream).await?;
        let res =
            AkarekoProtocolResponse::<T::ResponsePayload, T::ResponseData>::decode(stream).await?;
        Ok(res)
    }
}

trait AkarekoProtocolCommandHandler {
    async fn handle<S: AsyncRead + AsyncWrite + Unpin + Send>(
        stream: &mut S,
        state: &ServerState,
        address: &I2PAddress,
    );
}

impl<T: AkarekoProtocolCommand> AkarekoProtocolCommandHandler for T {
    async fn handle<S: AsyncRead + AsyncWrite + Unpin + Send>(
        stream: &mut S,
        state: &ServerState,
        address: &I2PAddress,
    ) {
        let req = T::RequestPayload::decode(stream).await.unwrap();
        let res = T::process(req, state, address).await;
        res.encode(stream).await.unwrap();
    }
}

/// Auto implemented by the handler macro, used to encode requests
pub trait AkarekoProtocolCommandMetadata {
    type CommandType: CommandEnum;

    const COMMAND: Self::CommandType;
    const VERSION: AkarekoProtocolVersion;

    async fn encode_request<W: AsyncWrite + Unpin + Send>(
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        Self::VERSION.encode(writer).await?;
        Self::COMMAND.encode(writer).await
    }
}

pub trait AkarekoMiddleware {
    fn apply(
        state: &ServerState,
        address: &I2PAddress,
    ) -> impl Future<Output = Result<(), ServerError>>;
}

struct RelayMiddleware;
impl AkarekoMiddleware for RelayMiddleware {
    async fn apply(state: &ServerState, _address: &I2PAddress) -> Result<(), ServerError> {
        if !state.config.read().await.is_relay() {
            return Err(ServerError::RelayNotEnabled);
        }

        Ok(())
    }
}

crate::handler!(V1,
{
    Who("who") => users::Who,

    // ==================== User ====================
    GetUsers("user/get_users") => users::GetUsers,

    // ==================== Index ====================
    GetAllIndexes("manga/get_all_indexes") => index::GetAllIndexes<MangaTag>,
    GetIndexes("manga/get_indexes") => index::GetIndexes<MangaTag>,
    GetContents("manga/get_contents") => index::GetContents<MangaTag>,

    // ==================== Post ====================
    GetPostsByTopic("post/get_posts_by_topic") => post::GetPostsByTopic,

    // ==================== Events ====================
    SyncEvents("event/sync_events") => events::SyncEvents

});
