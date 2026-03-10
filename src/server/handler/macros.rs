#[macro_export]
macro_rules! handler {
    (
        $version:ident,
        {
            $(
                $command:ident ($cmd_discriminant:literal $(, $middleware:ident)?) => $handler:path
            ),* $(,)?
        }
    ) => {
        paste::paste! {
            pub struct $version;

            #[repr(u8)]
            #[derive(Debug, Clone)]
            pub enum [<Commands $version>] {
                $(
                    $command,
                )*
            }
            impl CommandEnum for [<Commands $version>] {}
            impl Byteable for [<Commands $version>] {
                async fn encode<W: AsyncWrite + Unpin + Send>(
                    &self,
                    writer: &mut W,
                ) -> Result<(), EncodeError> {
                    match self {
                        $(
                            [<Commands $version>]::$command => $cmd_discriminant.to_string().encode(writer).await,
                        )*
                    }
                }

                async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
                    Ok(match String::decode(reader).await?.as_str() {
                        $(
                            $cmd_discriminant => [<Commands $version>]::$command,
                        )*
                        s => return Err(DecodeError::InvalidEnumVariant {
                            variant_value: s.to_string(),
                            enum_name: stringify!([<Commands $version>]),
                        }),
                    })
                }
            }

            $(
                impl AkarekoProtocolCommandMetadata for $handler {
                    type CommandType = [<Commands $version>];

                    const COMMAND: [<Commands $version>] =
                        [<Commands $version>]::$command;
                    const VERSION: AkarekoProtocolVersion = AkarekoProtocolVersion::$version;
                }
            )*

            impl $version {
                pub async fn handle<S: AsyncRead + AsyncWrite + Unpin + Send>(stream: &mut S, state: &ServerState, address: &I2PAddress) {
                    let command = [<Commands $version>]::decode(stream)
                        .await
                        .unwrap();

                    match command {
                        $(
                            [<Commands $version>]::$command => {
                                $(
                                    $middleware.apply(state, address).await.unwrap();
                                )*
                                <$handler as AkarekoProtocolCommandHandler>::handle(stream, state, address).await;
                            }
                        )*
                    }
                }
            }
        }
    };
}
