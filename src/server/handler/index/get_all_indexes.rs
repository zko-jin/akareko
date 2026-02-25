use fastbloom::BloomFilter;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{
    db::{
        Timestamp,
        index::{
            Index,
            tags::{IndexTag, MangaTag, NoTag},
        },
        user::I2PAddress,
    },
    errors::{DecodeError, EncodeError},
    helpers::Byteable,
    server::{ServerState, handler::AuroraProtocolCommand, protocol::AuroraProtocolResponse},
};

pub struct GetAllIndexes;

impl AuroraProtocolCommand for GetAllIndexes {
    type RequestPayload = GetAllIndexesRequest;
    type ResponsePayload = GetAllIndexesResponse;
    type ResponseData = Index<NoTag>;

    async fn process(
        req: Self::RequestPayload,
        state: &ServerState,
        _: &I2PAddress,
    ) -> AuroraProtocolResponse<Self::ResponsePayload, Self::ResponseData> {
        match req.tag.as_str() {
            MangaTag::TAG => {
                let indexes = match state
                    .repositories
                    .index()
                    .await
                    .get_all_indexes::<MangaTag>(req.timestamp, req.filter)
                    .await
                {
                    Ok(indexes) => indexes,
                    Err(_) => {
                        return AuroraProtocolResponse::internal_error(format!("Database error"));
                    }
                };

                // SAFETY: They are all the same type, just different tags
                AuroraProtocolResponse::ok_with_data(GetAllIndexesResponse {}, unsafe {
                    std::mem::transmute(indexes)
                })
            }
            _ => AuroraProtocolResponse::invalid_argument(format!("Invalid tag: {}", req.tag)),
        }
    }
}

impl Byteable for BloomFilter {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        let data = self.as_slice();
        // Safety: We're just reinterpreting the u64 array as bytes
        let bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * size_of::<u64>())
        };

        writer.write_u32(self.num_hashes()).await?;
        writer.write_u64(bytes.len() as u64).await?;
        writer.write(bytes).await?;

        Ok(())
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError>
    where
        Self: Sized,
    {
        let num_hashes = reader.read_u32().await?;
        let len = reader.read_u64().await? as usize;
        let mut vec: Vec<u64> = Vec::with_capacity(len / size_of::<u64>());
        unsafe {
            vec.set_len(len / size_of::<u64>());

            let buf: &mut [u8] = std::slice::from_raw_parts_mut(
                vec.as_mut_ptr() as *mut u8,
                vec.len() * size_of::<u64>(),
            );
            reader.read_exact(buf).await?;
        };

        Ok(BloomFilter::from_vec(vec).hashes(num_hashes))
    }
}

#[derive(byteable_derive::Byteable)]
pub struct GetAllIndexesRequest {
    tag: String,
    // Get indexes created_updated after this timestamp
    timestamp: Timestamp,
    filter: Option<BloomFilter>,
}

impl GetAllIndexesRequest {
    pub fn new<T: IndexTag>(timestamp: Timestamp, filter: Option<BloomFilter>) -> Self {
        Self {
            tag: T::TAG.to_string(),
            timestamp,
            filter,
        }
    }
}

#[derive(byteable_derive::Byteable)]
pub struct GetAllIndexesResponse {}
