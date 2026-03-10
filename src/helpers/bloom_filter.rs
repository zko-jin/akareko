use fastbloom::BloomFilter;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    errors::{DecodeError, EncodeError},
    helpers::{
        Byteable,
        serde_byteable::{decode_serde, encode_serde},
    },
};

impl Byteable for BloomFilter {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        encode_serde(&self, writer).await
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError>
    where
        Self: Sized,
    {
        decode_serde(reader).await
    }
}
