use crate::{
    db::user::{I2PAddress, User},
    server::{ServerState, handler::AkarekoProtocolCommand, protocol::AkarekoProtocolResponse},
    types::PublicKey,
};

pub struct GetUsers;

impl AkarekoProtocolCommand for GetUsers {
    type RequestPayload = GetUsersRequest;
    type ResponsePayload = GetUsersResponse;
    type ResponseData = ();

    async fn process(
        req: Self::RequestPayload,
        state: &ServerState,
        _: &I2PAddress,
    ) -> AkarekoProtocolResponse<Self::ResponsePayload, Self::ResponseData> {
        let users = match state.repositories.user().get_users(req.pub_keys).await {
            Ok(users) => users,
            Err(_) => {
                return AkarekoProtocolResponse::internal_error("Failed to get users".to_string());
            }
        };

        let users = users.into_iter().map(|u| u.into()).collect();

        AkarekoProtocolResponse::ok(Self::ResponsePayload { users })
    }
}

#[derive(Debug, byteable_derive::Byteable)]
pub struct GetUsersRequest {
    pub pub_keys: Vec<PublicKey>,
}

#[derive(Debug, byteable_derive::Byteable)]
pub struct GetUsersResponse {
    pub users: Vec<User>,
}
