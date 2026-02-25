use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{
    errors::{ClientError, DecodeError, EncodeError},
    helpers::Byteable,
    server::handler::AuroraProtocolCommand,
};

#[repr(u8)]
#[derive(Debug, Clone, byteable_derive::Byteable)]
pub enum AuroraProtocolVersion {
    V1 = 1,
}

#[derive(Debug)]
pub(super) struct AuroraProtocolRequest<C: AuroraProtocolCommand> {
    pub payload: C::RequestPayload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuroraStatus {
    Ok,
    NotFound(String),
    InvalidArgument(String),
    InternalError(String),
}

impl AuroraStatus {
    const OK_CODE: u16 = 200;
    const INTERNAL_ERROR_CODE: u16 = 500;
    const INVALID_ARGUMENT_CODE: u16 = 400;
    const NOT_FOUND_CODE: u16 = 404;

    pub fn is_ok(&self) -> bool {
        matches!(self, AuroraStatus::Ok)
    }

    pub fn code(&self) -> u16 {
        match self {
            AuroraStatus::Ok => Self::OK_CODE,
            AuroraStatus::InvalidArgument(_) => Self::INVALID_ARGUMENT_CODE,
            AuroraStatus::NotFound(_) => Self::NOT_FOUND_CODE,
            AuroraStatus::InternalError(_) => Self::INTERNAL_ERROR_CODE,
        }
    }
}

impl Byteable for AuroraStatus {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        writer.write_u16(self.code()).await?;

        match self {
            AuroraStatus::Ok => (),
            AuroraStatus::InvalidArgument(message) => {
                message.encode(writer).await?;
            }
            AuroraStatus::NotFound(message) => {
                message.encode(writer).await?;
            }
            AuroraStatus::InternalError(message) => {
                message.encode(writer).await?;
            }
        }

        Ok(())
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        let code = reader.read_u16().await?;

        let status = match code {
            Self::OK_CODE => AuroraStatus::Ok,
            Self::INVALID_ARGUMENT_CODE => {
                let message = String::decode(reader).await?;
                AuroraStatus::InvalidArgument(message)
            }
            Self::NOT_FOUND_CODE => {
                let message = String::decode(reader).await?;
                AuroraStatus::NotFound(message)
            }
            Self::INTERNAL_ERROR_CODE => {
                let message = String::decode(reader).await?;
                AuroraStatus::InternalError(message)
            }
            _ => {
                return Err(DecodeError::InvalidEnumVariant {
                    enum_name: "AuroraStatus",
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

    fn new_receiver(len: u64) -> Self {
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

pub(super) struct AuroraProtocolResponse<P: Byteable, D: Byteable = ()> {
    status: AuroraStatus,
    payload: Option<P>, // None if status is an error
    data: StreamDecode<D>,
}

impl<P: Byteable> AuroraProtocolResponse<P, ()> {
    pub fn ok(payload: P) -> Self {
        Self {
            status: AuroraStatus::Ok,
            payload: Some(payload),
            data: StreamDecode::new_receiver(0),
        }
    }
}

impl<P: Byteable, D: Byteable> AuroraProtocolResponse<P, D> {
    pub fn ok_with_data(payload: P, data: Vec<D>) -> Self {
        Self {
            status: AuroraStatus::Ok,
            payload: Some(payload),
            data: StreamDecode::new(data),
        }
    }

    pub fn data(&mut self) -> &mut StreamDecode<D> {
        &mut self.data
    }

    pub fn not_found(message: String) -> Self {
        Self {
            status: AuroraStatus::NotFound(message),
            payload: None,
            data: StreamDecode::new(vec![]),
        }
    }

    pub fn invalid_argument(message: String) -> Self {
        Self {
            status: AuroraStatus::InvalidArgument(message),
            payload: None,
            data: StreamDecode::new(vec![]),
        }
    }

    pub fn internal_error(message: String) -> Self {
        Self {
            status: AuroraStatus::InternalError(message),
            payload: None,
            data: StreamDecode::new(vec![]),
        }
    }

    pub fn status(&self) -> &AuroraStatus {
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

impl<C: AuroraProtocolCommand> AuroraProtocolRequest<C> {
    pub async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        C::encode_request(writer).await?;
        self.payload.encode(writer).await
    }
}

impl<P: Byteable, D: Byteable> Byteable for AuroraProtocolResponse<P, D> {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        self.status.encode(writer).await?;
        if let Some(payload) = &self.payload {
            payload.encode(writer).await?;
        }

        Ok(())
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        let status = AuroraStatus::decode(reader).await?;

        if !status.is_ok() {
            return Ok(AuroraProtocolResponse {
                status,
                payload: None,
                data: StreamDecode::new_receiver(0),
            });
        }

        let response = P::decode(reader).await?;
        let data = StreamDecode::decode(reader).await?;
        Ok(AuroraProtocolResponse {
            status,
            payload: Some(response),
            data,
        })
    }
}
