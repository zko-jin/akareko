use crate::errors::{DecodeError, EncodeError};
use crate::helpers::Byteable;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

macro_rules! impl_strings {
    (
        $(
            $bits:literal
        ),*
    ) => {
        paste::paste! {
            $(
                #[derive(Debug, Clone, PartialEq, Eq)]
                pub struct [<String $bits>](String);

                impl surrealdb_types::SurrealValue for [<String $bits>] {
                    fn kind_of() -> surrealdb_types::Kind {
                        surrealdb_types::Kind::String
                    }

                    fn into_value(self) -> surrealdb_types::Value {
                        surrealdb_types::Value::String(self.0)
                    }

                    fn from_value(value: surrealdb_types::Value) -> Result<Self, surrealdb::Error>
                    where
                        Self: Sized,
                    {
                        match value.kind() {
                            surrealdb::types::Kind::String => {
                                [<String $bits>]::new(value.into_string()?).map_err(|e| {
                                    surrealdb::Error::serialization(
                                        "String too long".to_string(),
                                        Some(surrealdb_types::SerializationError::Deserialization),
                                    )
                                })
                            }
                            _ => Err(surrealdb::Error::serialization(
                                "String can only be made from string".to_string(),
                                Some(surrealdb_types::SerializationError::Deserialization),
                            )),
                        }
                    }
                }

                impl [<String $bits>] {
                    /// Returns an error if the string is too long
                    pub fn new(s: String) -> Result<Self, usize> {
                        if s.len() > [<u $bits>]::MAX as usize {
                            return Err(s.len());
                        }
                        Ok([<String $bits>](s))
                    }

                    pub fn inner(&self) -> &str {
                        &self.0
                    }

                    pub fn to_inner(self) -> String {
                        self.0
                    }
                }

                impl AsRef<str> for [<String $bits>] {
                    fn as_ref(&self) -> &str {
                        self.0.as_ref()
                    }
                }

                impl std::fmt::Display for [<String $bits>] {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        self.0.fmt(f)
                    }
                }

                impl Byteable for [<String $bits>] {
                    async fn encode<W: AsyncWrite + Unpin + Send>(
                        &self,
                        writer: &mut W,
                    ) -> Result<(), EncodeError> {
                        if self.0.len() > [<u $bits>]::MAX as usize {
                            return Err(EncodeError::TooManyElements {
                                allowed: [<u $bits>]::MAX as usize,
                                actual: self.0.len(),
                            });
                        }
                        writer.[<write_u $bits>](self.0.len() as [<u $bits>]).await?;
                        writer.write(self.0.as_bytes()).await?;
                        Ok(())
                    }

                    async fn decode<R: AsyncRead + Unpin + Send>(
                        reader: &mut R,
                    ) -> Result<Self, DecodeError> {
                        let len = reader.[<read_u $bits>]().await?;
                        let mut buf = vec![0u8; len as usize];
                        reader.read_exact(&mut buf).await?;
                        Ok([<String $bits>](String::from_utf8(buf)?))
                    }
                }
            )*
        }
    };
}

impl_strings!(8, 16, 32);
