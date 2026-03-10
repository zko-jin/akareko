use crate::{
    db::{
        ToBytes,
        user::{I2PAddress, User},
    },
    server::{ServerState, handler::AkarekoProtocolCommand, protocol::AkarekoProtocolResponse},
    types::{PrivateKey, Signature, Timestamp},
};

#[derive(Debug)]
pub struct Who;

impl AkarekoProtocolCommand for Who {
    type RequestPayload = WhoRequest;
    type ResponsePayload = WhoResponse;
    type ResponseData = ();

    async fn process(
        _: Self::RequestPayload,
        state: &ServerState,
        address: &I2PAddress,
    ) -> AkarekoProtocolResponse<Self::ResponsePayload, Self::ResponseData> {
        let response: Option<WhoResponse> = {
            let config = state.config.read().await;
            let user_pub_key = config.public_key();
            let priv_key = config.private_key();
            match state
                .repositories
                .user()
                .get_user(user_pub_key)
                .await
                .unwrap()
            {
                Some(user) => Some(WhoResponse::new_signed(user, &address, priv_key)),
                None => None,
            }
        };

        if let Some(response) = response {
            AkarekoProtocolResponse::ok(response)
        } else {
            AkarekoProtocolResponse::not_found("User not found".to_string())
        }
    }
}

#[derive(Debug, byteable_derive::Byteable)]
pub struct WhoRequest {}

#[derive(Debug, byteable_derive::Byteable)]
pub struct WhoResponse {
    pub user: User,
    pub timestamp: Timestamp,
    pub signature: Signature, // Timestamp + Address of requesting user
}

impl WhoResponse {
    pub fn verification_bytes(&self, request_address: &I2PAddress) -> Vec<u8> {
        let mut bytes = self.timestamp.to_bytes();
        bytes.extend(request_address.to_string().as_bytes());
        bytes
    }

    pub fn new_signed(user: User, request_address: &I2PAddress, priv_key: &PrivateKey) -> Self {
        let mut response = Self {
            user: user.into(),
            timestamp: Timestamp::now(),
            signature: Signature::empty(),
        };

        let to_sign = response.verification_bytes(request_address);
        response.signature = priv_key.sign(&to_sign);

        response
    }

    pub fn verify(&self, request_address: &I2PAddress) -> bool {
        let bytes = self.verification_bytes(request_address);
        self.user.pub_key().verify(&bytes, &self.signature)
    }
}
