use fastbloom::BloomFilter;

use crate::{
    db::{
        event::{EventType, filter_events},
        index::tags::MangaTag,
        user::I2PAddress,
    },
    helpers::Byteable,
    server::{
        ServerState,
        handler::{
            AkarekoProtocolCommandHandler, AkarekoProtocolCommandMetadata,
            AkarekoProtocolCommandRequest,
        },
        protocol::AkarekoProtocolResponse,
    },
    types::{Hash, PublicKey, Signature, Timestamp},
};

pub struct SyncEvents;

impl AkarekoProtocolCommandRequest<SyncEventsRequest, AkarekoProtocolResponse<SyncEventsResponse>>
    for SyncEvents
{
    async fn request<S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send>(
        payload: SyncEventsRequest,
        stream: &mut S,
    ) -> Result<AkarekoProtocolResponse<SyncEventsResponse>, crate::errors::ClientError> {
        SyncEvents::encode_request(stream).await?;
        payload.encode(stream).await?;
        let res = AkarekoProtocolResponse::<SyncEventsResponse>::decode(stream).await?;
        Ok(res)
    }
}
impl AkarekoProtocolCommandHandler for SyncEvents {
    async fn handle<S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send>(
        stream: &mut S,
        state: &ServerState,
        _: &I2PAddress,
    ) {
        let req = SyncEventsRequest::decode(stream).await.unwrap();

        let events = match filter_events(req.timestamp, req.filter, &state.repositories.db).await {
            Ok(events) => events,
            Err(_) => {
                AkarekoProtocolResponse::<(), ()>::internal_error("Database Error".into())
                    .encode(stream)
                    .await
                    .unwrap();
                return;
            }
        };

        let decode_streams = events
            .iter()
            .filter_map(|e| {
                if e.0 == EventType::Invalid {
                    None
                } else {
                    Some((e.0, e.1.len() as u64))
                }
            })
            .collect();

        AkarekoProtocolResponse::<SyncEventsResponse>::ok(SyncEventsResponse {
            decode_streams,
            timestamp: Timestamp::now(),
        })
        .encode(stream)
        .await
        .unwrap();

        // SAFETY: Our DB should be verified anyway and the client will check it
        // later too. The only problem would be losing trust from a bad DB state.
        // Perhaps we should add an option where we verify stuff before sending,
        // check the performance impact if so.
        for (event_type, topics) in events {
            match event_type {
                EventType::Invalid => unreachable!(),
                EventType::User => {
                    let keys = topics
                        .into_iter()
                        .map(|v| unsafe {
                            // Don't know if the clone is necessary...
                            let bytes = v.to_inner()[..32].try_into().unwrap();
                            PublicKey::from_bytes_unchecked(bytes)
                        })
                        .collect();

                    let users = state.repositories.user().get_users(keys).await.unwrap();
                    for user in users {
                        user.encode(stream).await.unwrap();
                    }
                }
                EventType::Manga => {
                    let hashes = topics
                        .into_iter()
                        .map(|v| Hash::new(v.to_inner()))
                        .collect::<Vec<_>>();

                    let indexes = state
                        .repositories
                        .index()
                        .get_indexes::<MangaTag>(&hashes)
                        .await
                        .unwrap();

                    for index in indexes {
                        index.encode(stream).await.unwrap();
                    }
                }
                EventType::MangaContent => {
                    let signatures = topics
                        .into_iter()
                        .map(|v| unsafe { Signature::from_bytes_unchecked(v.to_inner()) })
                        .collect::<Vec<_>>();

                    let contents = state
                        .repositories
                        .index()
                        .get_contents::<MangaTag>(&signatures)
                        .await
                        .unwrap();

                    for content in contents {
                        content.encode(stream).await.unwrap();
                    }
                }
                EventType::Post => {
                    let signatures = topics
                        .into_iter()
                        .map(|v| unsafe { Signature::from_bytes_unchecked(v.to_inner()) })
                        .collect::<Vec<_>>();

                    let posts = state.repositories.get_posts(&signatures).await.unwrap();

                    for post in posts {
                        post.encode(stream).await.unwrap();
                    }
                }
            }
        }
    }
}

#[derive(Debug, byteable_derive::Byteable)]
pub struct SyncEventsRequest {
    pub timestamp: Timestamp,
    pub filter: Option<BloomFilter>,
}

#[derive(Debug, byteable_derive::Byteable)]
pub struct SyncEventsResponse {
    // The opposing server checked timestamp
    pub timestamp: Timestamp,
    pub decode_streams: Vec<(EventType, u64)>,
}
