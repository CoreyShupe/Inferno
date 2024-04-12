use std::string::FromUtf8Error;

#[derive(thiserror::Error, Debug)]
pub enum PacketsError {
    #[error("Unknown packet type: {0}")]
    UnknownPacketType(u8),
    #[error("Unknown value type: {0}")]
    UnknownValueType(u8),
    #[error("Unknown instruction type: {0}")]
    UnknownInstructionType(u8),
}

#[derive(thiserror::Error, Debug)]
pub enum StateError {
    #[error("Unknown state issue.")]
    BadState,
    #[error("Cannot return this key type.")]
    CannotReturnKeyType,
    #[error("Attempted to index a key with a bad type.")]
    BadKeyType,
}

#[derive(thiserror::Error, Debug)]
pub enum InfernoError {
    #[error(transparent)]
    Packets(#[from] PacketsError),
    #[error(transparent)]
    State(#[from] StateError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    FromUtf8(#[from] FromUtf8Error),
    #[error("{0}")]
    DecodedMessage(String),
}

pub type Result<T> = std::result::Result<T, InfernoError>;
