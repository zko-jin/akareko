use rclite::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};
use yosemite::{Session, SessionOptions, Stream, style};

use crate::{
    config::AuroraConfig,
    db::{
        Repositories,
        index::{Index, IndexRepository, tags::IndexTag},
        user::{I2PAddress, TrustLevel, User},
    },
    errors::ClientError,
    hash::PublicKey,
    server::{
        handler::{
            self, AuroraProtocolCommand,
            index::GetAllIndexesRequest,
            users::{get_users::GetUsersRequest, who::WhoRequest},
        },
        proxy::LoggingStream, // proxy::I2PConnector,
    },
};

#[derive(Clone)]
pub struct AuroraClient {
    repositories: Repositories,
    host_address: I2PAddress,
    session: Arc<Mutex<Session<style::Stream>>>,
}

impl AuroraClient {
    pub async fn new(repositories: Repositories, config: AuroraConfig) -> Self {
        info!("Initializing AuroraClient...");

        let session = Arc::new(Mutex::new(
            Session::<style::Stream>::new(SessionOptions {
                // nickname: "AuroraClient".to_string(),
                samv3_tcp_port: config.sam_port(),
                destination: yosemite::DestinationKind::Persistent {
                    private_key: config.eepsite_key().clone(),
                },
                ..Default::default()
            })
            .await
            .unwrap(),
        ));

        info!("Initialized AuroraClient");

        Self {
            repositories,
            session,
            host_address: config.eepsite_address().clone(),
        }
    }

    async fn get_stream(&mut self, url: &I2PAddress) -> Result<LoggingStream<Stream>, ClientError> {
        let session = self.session.clone();
        let stream = session.lock().await.connect(url.inner()).await?;

        let stream = LoggingStream(stream);

        Ok(stream)
    }

    // ╔===========================================================================╗
    // ║                                   Index                                   ║
    // ╚===========================================================================╝

    pub async fn get_all_indexes<T: IndexTag>(
        &mut self,
        url: &I2PAddress,
        db: IndexRepository<'_>,
    ) -> Result<(), ClientError> {
        let mut stream = self.get_stream(url).await?;

        let mut res = handler::index::GetAllIndexes::request(
            GetAllIndexesRequest::new::<T>(0, None),
            &mut stream,
        )
        .await?;

        if !res.status().is_ok() {
            return Err(ClientError::UnexpectedResponseCode {
                status: res.status().clone(),
            });
        }

        while let Ok(Some(index)) = res.data().next(&mut stream).await {
            let index: Index<T> = index.transmute();

            if !index.verify() {
                error!("Invalid index signature");
                continue;
            }

            match db.add_index::<T>(index).await {
                Ok(_) => {}
                Err(e) => {
                    error!("Failed to add index: {}", e);
                }
            }
        }

        Ok(())
    }

    // ╔===========================================================================╗
    // ║                                 Exchange                                  ║
    // ╚===========================================================================╝

    // pub async fn routine_exchange(&mut self, url: &I2PAddress) -> Result<(), ClientError> {
    //     let mut stream = self.get_stream(url).await?;

    //     let who = self.who_internal(&mut stream).await?;

    //     self.repositories.user().await.upsert_user(who).await?;

    //     let response = handler::index::ExchangeContent::request(
    //         ExchangeContentRequest { count: 10 },
    //         &mut stream,
    //     )
    //     .await?;

    //     let contents = response.payload_if_ok()?.contents;

    //     let mut existing_indexes: HashSet<Hash> = HashSet::new();
    //     let mut missing_indexes: Vec<Hash> = Vec::new();

    //     for content in contents.iter() {
    //         match content {
    //             TaggedContent::Manga(content) => {
    //                 match self
    //                     .repositories
    //                     .index()
    //                     .await
    //                     .get_index::<MangaTag>(content.index_hash())
    //                     .await
    //                 {
    //                     Ok(i) => match i {
    //                         Some(_) => {
    //                             existing_indexes.insert(content.index_hash().clone());
    //                         }
    //                         None => {
    //                             missing_indexes.push(content.index_hash().clone());
    //                         }
    //                     },
    //                     Err(e) => {
    //                         error!("Failed to get index: {}", e);
    //                     }
    //                 }
    //             }
    //         }
    //     }

    //     let mut response = handler::index::GetIndexes::request(
    //         GetIndexesRequest::new(missing_indexes),
    //         &mut stream,
    //     )
    //     .await?;

    //     if !response.status().is_ok() {
    //         return Err(ClientError::UnexpectedResponseCode {
    //             status: response.status().clone(),
    //         });
    //     }

    //     while let Ok(Some(index)) = response.data().next(&mut stream).await {
    //         let index: Index<T> = index.make_tagged();

    //         if !index.verify() {
    //             error!("Invalid index signature");
    //             continue;
    //         }

    //         match index {
    //             self.repositories.index().await.add_index(index).await {
    //                 Ok(i) => {
    //                     existing_indexes.insert(i.hash().clone());
    //                 }
    //                 Err(e) => {
    //                     error!("Failed to add index: {}", e);
    //                 }
    //             }
    //         }
    //     }

    //     for content in contents.into_iter() {
    //         if !existing_indexes.contains(content.index_hash()) {
    //             continue;
    //         }

    //         if !content.verify() {
    //             error!("Invalid content signature");
    //             continue;
    //         }
    //         match content {
    //             TaggedContent::Manga(content) => {
    //                 match self.repositories.index().await.add_content(content).await {
    //                     Ok(_) => {}
    //                     Err(e) => {
    //                         error!("Failed to add content: {}", e);
    //                     }
    //                 }
    //             }
    //         }
    //     }

    //     Ok(())
    // }

    // ╔===========================================================================╗
    // ║                                   User                                    ║
    // ╚===========================================================================╝

    /// Who function without creating a new stream
    async fn who_internal(&self, stream: &mut LoggingStream<Stream>) -> Result<User, ClientError> {
        let res = handler::users::Who::request(WhoRequest {}, stream).await?;

        if !res.status().is_ok() {
            return Err(ClientError::UnexpectedResponseCode {
                status: res.status().clone(),
            });
        }

        let Some(payload) = res.payload() else {
            return Err(ClientError::MissingPayload);
        };

        if !payload.verify(&self.host_address) {
            return Err(ClientError::InvalidSignature);
        }

        let mut user = payload.user.as_user();
        if !user.verify() {
            return Err(ClientError::InvalidSignature);
        }

        user.set_trust(TrustLevel::Untrusted);

        Ok(user)
    }

    pub async fn who(&mut self, url: &I2PAddress) -> Result<User, ClientError> {
        let mut stream = self.get_stream(url).await?;
        self.who_internal(&mut stream).await
    }

    pub async fn request_users(
        &mut self,
        url: &I2PAddress,
        pub_keys: Vec<PublicKey>,
    ) -> Result<Vec<User>, ClientError> {
        let mut stream = self.get_stream(url).await?;

        let res =
            handler::users::GetUsers::request(GetUsersRequest { pub_keys }, &mut stream).await?;

        if !res.status().is_ok() {
            return Err(ClientError::UnexpectedResponseCode {
                status: res.status().clone(),
            });
        }

        let Some(payload) = res.payload() else {
            return Err(ClientError::MissingPayload);
        };

        let users: Vec<User> = payload.users.into_iter().map(|u| u.as_user()).collect();

        // TODO
        // self.repositories
        //     .get_user_repository()
        //     .save_users(users.clone())
        //     .await?;

        Ok(users)
    }
}

impl std::fmt::Debug for AuroraClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuroraClient").finish()
    }
}
