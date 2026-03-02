use std::io;

use rclite::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};
use yosemite::{Session, SessionOptions, style};

use crate::{
    config::AkarekoConfig,
    db::{Repositories, user::I2PAddress},
    errors::{DecodeError, ServerError},
    helpers::{Byteable, b32_from_pub_b64},
    server::protocol::AkarekoProtocolVersion,
};

pub mod client;
mod handler;
pub mod protocol;
pub mod proxy;

pub struct AkarekoServer {}

#[derive(Clone)]
struct ServerState {
    pub config: Arc<RwLock<AkarekoConfig>>,
    pub repositories: Repositories,
}

impl AkarekoServer {
    pub fn new() -> AkarekoServer {
        AkarekoServer {}
    }

    pub async fn run(
        &self,
        config: Arc<RwLock<AkarekoConfig>>,
        repositories: Repositories,
    ) -> Result<(), ServerError> {
        info!("Starting server SAMv3 session");

        let mut sam_session = {
            let config_guard = config.read().await;

            Session::<style::Stream>::new(SessionOptions {
                // nickname: "AuroraServer".to_string(),
                samv3_tcp_port: config_guard.sam_port(),
                destination: yosemite::DestinationKind::Persistent {
                    private_key: config_guard.eepsite_key().clone(),
                },
                ..Default::default()
            })
            .await?
        };

        info!("Server Started");
        // info!(
        //     "Starting server on {}",
        //     b64_to_b32_i2p(sam_session.destination()).unwrap()
        // );

        let state = ServerState {
            config,
            repositories,
        };

        while let Ok(mut stream) = sam_session.accept().await {
            let state = state.clone();
            tokio::spawn(async move {
                let address = b32_from_pub_b64(stream.remote_destination()).unwrap();

                loop {
                    let version = match AkarekoProtocolVersion::decode(&mut stream).await {
                        Ok(v) => v,
                        Err(e) => match e {
                            DecodeError::IoError(e) => {
                                match e.kind() {
                                    io::ErrorKind::UnexpectedEof => {
                                        //
                                    }
                                    _ => {
                                        error!("Failed to decode version: {}", e);
                                    }
                                }
                                break;
                            }
                            _ => {
                                error!("Failed to decode version: {}", e);
                                break;
                            }
                        },
                    };

                    match version {
                        AkarekoProtocolVersion::V1 => {
                            handler::V1::handle(&mut stream, &state, &address).await;
                        }
                    }
                }
            });
        }

        Ok(())
    }
}
