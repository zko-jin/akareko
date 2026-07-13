use std::string::FromUtf8Error;

use anawt::errors::LtrsError;
use skerry::skerry_global;

use crate::server::protocol::AkarekoStatus;

error_set::error_set! {
    Base64Error := {
        InvalidBase64(base64::DecodeError),
        InvalidLength {
            expected: usize,
            actual: usize
        }
    }

    TomlError := {
        TomlDeError(toml::de::Error),
        TomlSerError(toml::ser::Error)
    }

    IoError := {
        IoError(std::io::Error)
    }

    ApiError := {
        MangadexApiError(mangadex_api::error::Error)
    }

    TomlSaveError := TomlError || IoError

    I2PParseError := Base64Error

    TorrentError := {
        LtrsError(LtrsError),
        Unknown,
        NotInitialized
    }

    YosemiteError := {
        YosemiteError(yosemite::Error)
    }

    SurrealError := {
        SurrealError(surrealdb::Error)
    }

    // DieselError := {
    //     DieselError(diesel::result::Error)
    // }

    DatabaseError := {Unknown, NotInitialized} || SurrealError /*||
DieselError */
    ServerError := { RelayNotEnabled } || YosemiteError || IoError

    InvalidSignature := {
        InvalidSignature
    }

    ClientError := { MissingPayload, UnexpectedResponseCode { status:
AkarekoStatus } } || EncodeError             || DecodeError || YosemiteError
|| InvalidSignature || DatabaseError

    EncodeError := {
        InvalidData,
        TooManyElements {
            allowed: usize,
            actual: usize
        }
    } || IoError || Base64Error

    DecodeError := {
        InvalidEnumVariant {
            variant_value: String,
            enum_name: &'static str
        },
        InvalidData,
        FromUtf8Error(FromUtf8Error)
    } || IoError
}

impl serde::ser::Error for EncodeError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        EncodeError::InvalidData
    }
}

impl serde::de::Error for DecodeError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        DecodeError::InvalidData
    }
}

#[skerry_global]
pub enum AkarekoErrors {
    #[from]
    InvalidBase64(base64::DecodeError),
    Base64WrongLength {
        expected: usize,
        actual: usize,
    },
    #[from]
    TomlDe(toml::de::Error),
    #[from]
    TomlSer(toml::ser::Error),
    #[from]
    FromUtf8(FromUtf8Error),
    // #[from]
    // StdIo(std::io::Error),
    #[from]
    TokioIo(tokio::io::Error),
    // ==================== Validation ====================
    InvalidSignature,
    // ==================== Networking ====================
    #[from]
    Yosemite(yosemite::Error),
    #[from]
    MangadexApi(mangadex_api::error::Error),
    // ==================== Database ====================
    #[from]
    Surreal(surrealdb::Error),
    NotFound,
    DatabaseNotInitialized,
}
