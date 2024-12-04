use std::io::Error;

#[derive(Debug)]
pub enum EncodeError {
    MalformedPacket,
    InvalidTopic,
    EncodingFailure,
    UnexpectedEndOfInput,
    Other(String),
    IoError(Error),
}

impl From<Error> for EncodeError {
    fn from(error: Error) -> Self {
        Self::IoError(error)
    }
}
