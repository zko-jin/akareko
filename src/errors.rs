#![allow(unused_variables)]

use std::string::FromUtf8Error;

use crate::server::protocol::AuroraStatus;

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

    TomlSaveError := TomlError || IoError

    I2PParseError := Base64Error

    YosemiteError := {
        YosemiteError(yosemite::Error)
    }

    SurrealError := {
        SurrealError(surrealdb::Error)
    }

    DatabaseError := {Unknown} || SurrealError

    ServerError := YosemiteError

    InvalidSignature := {
        InvalidSignature
    }

    ClientError := { MissingPayload, UnexpectedResponseCode { status: AuroraStatus } } || EncodeError
            || DecodeError || YosemiteError || InvalidSignature || DatabaseError

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
        FromUtf8Error(FromUtf8Error)
    } || IoError
}
