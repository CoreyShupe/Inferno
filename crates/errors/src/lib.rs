use std::string::FromUtf8Error;

#[derive(thiserror::Error, Debug)]
pub enum PacketsError {
    #[error("Unknown packet type: {0}")]
    UnknownPacketType(u8),
    #[error("Unknown value type: {0}")]
    UnknownValueType(u8),
}

#[derive(thiserror::Error, Debug)]
pub enum InfernoError {
    #[error(transparent)]
    Packets(PacketsError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    FromUtf8(#[from] FromUtf8Error),
    #[error("{0}")]
    DecodedMessage(String),
}

pub type Result<T> = std::result::Result<T, InfernoError>;
