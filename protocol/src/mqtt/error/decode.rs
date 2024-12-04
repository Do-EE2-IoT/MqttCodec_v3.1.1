use std::io::Error;
#[derive(Debug)]
pub enum DecodeError {
    MalformedPacket,
    InvalidTopic,
    UnexpectedEndOfInput,
    InvalidMessageFormat,
    PayloadTooLarge,
    ChecksumError,
    ProtocolError,
    Other(String),
    IoError(Error),
}

impl From<Error> for DecodeError {
    fn from(error: Error) -> Self {
        DecodeError::IoError(error)
    }
}
