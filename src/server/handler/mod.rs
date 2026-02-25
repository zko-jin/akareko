use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{
    db::user::I2PAddress,
    errors::{ClientError, DecodeError, EncodeError},
    helpers::Byteable,
    server::{
        ServerState,
        protocol::{AuroraProtocolRequest, AuroraProtocolResponse, AuroraProtocolVersion},
    },
};

pub mod index;
mod macros;
pub mod post {
    mod get_posts_by_topic;
    pub use get_posts_by_topic::GetPostsByTopic;
}
pub mod users;

/// Marker implemented by the handler macro
pub trait CommandEnum: Byteable {}

/// Should be implemented by each command, can be skipped by directly implementing [`AuroraProtocolCommandHandler`]
pub(super) trait AuroraProtocolCommand: Sized + AuroraProtocolCommandMetadata {
    type RequestPayload: Byteable;
    type ResponsePayload: Byteable;
    type ResponseData: Byteable;

    // Used by the client
    async fn request<S: AsyncRead + AsyncWrite + Unpin + Send>(
        payload: Self::RequestPayload,
        stream: &mut S,
    ) -> Result<AuroraProtocolResponse<Self::ResponsePayload, Self::ResponseData>, ClientError>
    {
        let req = AuroraProtocolRequest::<Self> { payload };
        req.encode(stream).await?;
        let res =
            AuroraProtocolResponse::<Self::ResponsePayload, Self::ResponseData>::decode(stream)
                .await?;
        Ok(res)
    }

    async fn process(
        req: Self::RequestPayload,
        state: &ServerState,
        address: &I2PAddress,
    ) -> AuroraProtocolResponse<Self::ResponsePayload, Self::ResponseData>;
}

trait AuroraProtocolCommandHandler {
    async fn handle<S: AsyncRead + AsyncWrite + Unpin + Send>(
        stream: &mut S,
        state: &ServerState,
        address: &I2PAddress,
    );
}

impl<T: AuroraProtocolCommand> AuroraProtocolCommandHandler for T {
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
pub trait AuroraProtocolCommandMetadata {
    type CommandType: CommandEnum;

    const COMMAND: Self::CommandType;
    const VERSION: AuroraProtocolVersion;

    async fn encode_request<W: AsyncWrite + Unpin + Send>(
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        Self::VERSION.encode(writer).await?;
        Self::COMMAND.encode(writer).await
    }
}

pub trait AuroraMiddleware {}

crate::handler!(V1,
{
    Who("who") => users::Who,

    // ==================== User ====================
    GetUsers("user/get_users") => users::GetUsers,

    // ==================== Index ====================
    GetAllIndexes("index/get_all_indexes") => index::GetAllIndexes,
    GetIndexes("index/get_indexes") => index::GetIndexes,

    // ==================== Post ====================
    GetPostsByTopic("post/get_posts_by_topic") => post::GetPostsByTopic
});
