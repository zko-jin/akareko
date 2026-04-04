use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{
    errors::{ClientError, DecodeError, EncodeError},
    helpers::Byteable,
    server::handler::{AkarekoProtocolCommand, AkarekoProtocolCommandMetadata},
};

#[repr(u8)]
#[derive(Debug, Clone, byteable_derive::Byteable)]
pub enum AkarekoProtocolVersion {
    V1 = 1,
}

#[derive(Debug)]
pub(super) struct AkarekoProtocolRequest<C: AkarekoProtocolCommand> {
    pub payload: C::RequestPayload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AkarekoStatus {
    Ok,
    NotFound(String),
    InvalidArgument(String),
    InternalError(String),
}

impl AkarekoStatus {
    const OK_CODE: u16 = 200;
    const INTERNAL_ERROR_CODE: u16 = 500;
    const INVALID_ARGUMENT_CODE: u16 = 400;
    const NOT_FOUND_CODE: u16 = 404;

    pub fn is_ok(&self) -> bool {
        matches!(self, AkarekoStatus::Ok)
    }

    pub fn code(&self) -> u16 {
        match self {
            AkarekoStatus::Ok => Self::OK_CODE,
            AkarekoStatus::InvalidArgument(_) => Self::INVALID_ARGUMENT_CODE,
            AkarekoStatus::NotFound(_) => Self::NOT_FOUND_CODE,
            AkarekoStatus::InternalError(_) => Self::INTERNAL_ERROR_CODE,
        }
    }
}

impl Byteable for AkarekoStatus {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        writer.write_u16(self.code()).await?;

        match self {
            AkarekoStatus::Ok => (),
            AkarekoStatus::InvalidArgument(message) => {
                message.encode(writer).await?;
            }
            AkarekoStatus::NotFound(message) => {
                message.encode(writer).await?;
            }
            AkarekoStatus::InternalError(message) => {
                message.encode(writer).await?;
            }
        }

        Ok(())
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        let code = reader.read_u16().await?;

        let status = match code {
            Self::OK_CODE => AkarekoStatus::Ok,
            Self::INVALID_ARGUMENT_CODE => {
                let message = String::decode(reader).await?;
                AkarekoStatus::InvalidArgument(message)
            }
            Self::NOT_FOUND_CODE => {
                let message = String::decode(reader).await?;
                AkarekoStatus::NotFound(message)
            }
            Self::INTERNAL_ERROR_CODE => {
                let message = String::decode(reader).await?;
                AkarekoStatus::InternalError(message)
            }
            _ => {
                return Err(DecodeError::InvalidEnumVariant {
                    enum_name: "AkarekoStatus",
                    variant_value: code.to_string(),
                });
            }
        };

        Ok(status)
    }
}

enum Either<A, B> {
    A(A),
    B(B),
}

// TODO: Later try to change the vec to a stream
pub(super) struct StreamDecode<D: Byteable> {
    d: Either<Vec<D>, u64>,
}

impl<D: Byteable> StreamDecode<D> {
    pub fn new(data: Vec<D>) -> Self {
        Self { d: Either::A(data) }
    }

    pub fn new_receiver(len: u64) -> Self {
        Self { d: Either::B(len) }
    }

    pub async fn next<R: AsyncRead + Unpin + Send>(
        &mut self,
        reader: &mut R,
    ) -> Result<Option<D>, DecodeError> {
        match &mut self.d {
            Either::A(_) => Ok(None),
            Either::B(len) => {
                if *len == 0 {
                    Ok(None)
                } else {
                    *len -= 1;
                    Ok(Some(D::decode(reader).await?))
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        match &self.d {
            Either::A(vec) => vec.len(),
            Either::B(len) => *len as usize,
        }
    }
}

impl<D: Byteable> Byteable for StreamDecode<D> {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        match &self.d {
            Either::A(iter) => {
                (iter.len() as u64).encode(writer).await?;
                for i in iter {
                    i.encode(writer).await?;
                }
            }
            Either::B(_) => {
                return Err(EncodeError::InvalidData);
            }
        }

        Ok(())
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        Ok(StreamDecode {
            d: Either::B(u64::decode(reader).await?),
        })
    }
}

pub(super) struct AkarekoProtocolResponse<P: Byteable, D: Byteable = ()> {
    status: AkarekoStatus,
    payload: Option<P>, // None if status is an error
    data: StreamDecode<D>,
}

impl<P: Byteable> AkarekoProtocolResponse<P, ()> {
    pub fn ok(payload: P) -> Self {
        Self {
            status: AkarekoStatus::Ok,
            payload: Some(payload),
            data: StreamDecode::new(vec![]),
        }
    }
}

impl<P: Byteable, D: Byteable> AkarekoProtocolResponse<P, D> {
    pub fn ok_with_data(payload: P, data: Vec<D>) -> Self {
        Self {
            status: AkarekoStatus::Ok,
            payload: Some(payload),
            data: StreamDecode::new(data),
        }
    }

    pub fn data(&mut self) -> &mut StreamDecode<D> {
        &mut self.data
    }

    pub fn not_found(message: String) -> Self {
        Self {
            status: AkarekoStatus::NotFound(message),
            payload: None,
            data: StreamDecode::new(vec![]),
        }
    }

    pub fn invalid_argument(message: String) -> Self {
        Self {
            status: AkarekoStatus::InvalidArgument(message),
            payload: None,
            data: StreamDecode::new(vec![]),
        }
    }

    pub fn internal_error(message: String) -> Self {
        Self {
            status: AkarekoStatus::InternalError(message),
            payload: None,
            data: StreamDecode::new(vec![]),
        }
    }

    pub fn status(&self) -> &AkarekoStatus {
        &self.status
    }

    pub fn payload(self) -> Option<P> {
        self.payload
    }

    pub fn payload_if_ok(self) -> Result<P, ClientError> {
        if !self.status().is_ok() {
            return Err(ClientError::UnexpectedResponseCode {
                status: self.status,
            });
        }

        let Some(contents) = self.payload() else {
            return Err(ClientError::MissingPayload);
        };

        return Ok(contents);
    }
}

impl<C: AkarekoProtocolCommand + AkarekoProtocolCommandMetadata> AkarekoProtocolRequest<C> {
    pub async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        C::encode_request(writer).await?;
        self.payload.encode(writer).await
    }
}

impl<P: Byteable, D: Byteable> Byteable for AkarekoProtocolResponse<P, D> {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        self.status.encode(writer).await?;
        if let Some(payload) = &self.payload {
            payload.encode(writer).await?;
            self.data.encode(writer).await?;
        }

        Ok(())
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        let status = AkarekoStatus::decode(reader).await?;

        if !status.is_ok() {
            return Ok(AkarekoProtocolResponse {
                status,
                payload: None,
                data: StreamDecode::new_receiver(0),
            });
        }

        let response = P::decode(reader).await?;
        let data = StreamDecode::decode(reader).await?;
        Ok(AkarekoProtocolResponse {
            status,
            payload: Some(response),
            data,
        })
    }
}
