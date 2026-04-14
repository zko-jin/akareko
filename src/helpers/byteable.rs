use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use uuid::Uuid;

use crate::errors::{DecodeError, EncodeError};

pub trait Byteable {
    fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> impl Future<Output = Result<(), EncodeError>>;
    fn decode<R: AsyncRead + Unpin + Send>(
        reader: &mut R,
    ) -> impl Future<Output = Result<Self, DecodeError>>
    where
        Self: Sized;
}

// Replace byteable later with these 2
pub trait Encodeable {
    fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> impl Future<Output = Result<(), EncodeError>>;
}

pub trait Decodeable {
    fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError>
    where
        Self: Sized;
}

impl Byteable for () {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        _writer: &mut W,
    ) -> Result<(), EncodeError> {
        Ok(())
    }

    async fn decode<R: AsyncRead + Unpin + Send>(_reader: &mut R) -> Result<Self, DecodeError> {
        Ok(())
    }
}

impl Byteable for Uuid {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        let b = self.as_bytes();
        b.encode(writer).await?;
        Ok(())
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        let b = uuid::Bytes::decode(reader).await?;
        Ok(Uuid::from_bytes(b))
    }
}

impl<T: Byteable, U: Byteable> Byteable for (T, U) {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        self.0.encode(writer).await?;
        self.1.encode(writer).await
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        Ok((T::decode(reader).await?, U::decode(reader).await?))
    }
}

impl<T: Byteable> Byteable for Vec<T> {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        if self.len() > u16::MAX as usize {
            return Err(EncodeError::TooManyElements {
                allowed: u16::MAX as usize,
                actual: self.len(),
            });
        }
        writer.write_u16(self.len() as u16).await?;

        for i in self {
            i.encode(writer).await?;
        }

        Ok(())
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError>
    where
        Self: Sized,
    {
        let len = reader.read_u16().await?;

        let mut vec = Vec::with_capacity(len as usize);
        for _ in 0..len {
            vec.push(T::decode(reader).await?);
        }

        Ok(vec)
    }
}

impl Byteable for u8 {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        writer.write_u8(*self).await?;
        Ok(())
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        Ok(reader.read_u8().await?)
    }
}

impl Byteable for u16 {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        writer.write_u16(*self).await?;
        Ok(())
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        Ok(reader.read_u16().await?)
    }
}

impl Byteable for u32 {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        writer.write_u32(*self).await?;
        Ok(())
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        Ok(reader.read_u32().await?)
    }
}

impl Byteable for u64 {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        writer.write_u64(*self).await?;
        Ok(())
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        Ok(reader.read_u64().await?)
    }
}

impl<T: Byteable> Byteable for Option<T> {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        match self {
            Some(value) => {
                writer.write_u8(1).await?;
                value.encode(writer).await
            }
            None => {
                writer.write_u8(0).await?;
                Ok(())
            }
        }
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        let value = reader.read_u8().await?;
        if value == 0 {
            Ok(None)
        } else {
            Ok(Some(T::decode(reader).await?))
        }
    }
}

impl<T: Byteable, E: Byteable> Byteable for Result<T, E> {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        match self {
            Ok(value) => {
                writer.write_u8(0).await?;
                value.encode(writer).await
            }
            Err(err) => {
                writer.write_u8(1).await?;
                err.encode(writer).await
            }
        }
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        let value = reader.read_u8().await?;
        if value == 0 {
            Ok(Ok(T::decode(reader).await?))
        } else {
            Ok(Err(E::decode(reader).await?))
        }
    }
}

impl Encodeable for str {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        if self.len() > u16::MAX as usize {
            return Err(EncodeError::TooManyElements {
                allowed: u16::MAX as usize,
                actual: self.len(),
            });
        }
        writer.write_u16(self.len() as u16).await?;
        writer.write(self.as_bytes()).await?;
        Ok(())
    }
}

impl Byteable for String {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        //self.as_str().encode(writer).await
        if self.len() > u16::MAX as usize {
            return Err(EncodeError::TooManyElements {
                allowed: u16::MAX as usize,
                actual: self.len(),
            });
        }
        writer.write_u16(self.len() as u16).await?;
        writer.write(self.as_bytes()).await?;
        Ok(())
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        let len = reader.read_u16().await?;
        let mut buf = vec![0u8; len as usize];
        reader.read_exact(&mut buf).await?;
        Ok(String::from_utf8(buf)?)
    }
}

impl Byteable for bool {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        writer.write_u8(*self as u8).await?;
        Ok(())
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        Ok(reader.read_u8().await? != 0)
    }
}

impl Byteable for f32 {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        writer.write_f32(*self).await?;
        Ok(())
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        Ok(reader.read_f32().await?)
    }
}

impl<const N: usize> Byteable for [u8; N] {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        writer.write(self).await?;
        Ok(())
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        let mut buf = [0u8; N];
        reader.read_exact(&mut buf).await?;
        Ok(buf)
    }
}

impl Byteable for i32 {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        writer.write_i32(*self).await?;
        Ok(())
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        Ok(reader.read_i32().await?)
    }
}

impl Byteable for i64 {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        writer.write_i64(*self).await?;
        Ok(())
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        Ok(reader.read_i64().await?)
    }
}
