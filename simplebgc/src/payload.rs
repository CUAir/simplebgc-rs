use bytes::Bytes;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq)]
pub enum PayloadParseError {
    InvalidFlags { name: String },
    InvalidEnum { name: String },
}

impl Display for PayloadParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PayloadParseError::InvalidFlags { name } => {
                write!(f, "invalid flags value for {}", name)
            }
            PayloadParseError::InvalidEnum { name } => write!(f, "invalid enum value for {}", name),
        }
    }
}

impl Error for PayloadParseError {}

pub trait Payload {
    fn from_bytes(b: Bytes) -> Result<Self, PayloadParseError>
    where
        Self: Sized;

    fn to_bytes(&self) -> Bytes
    where
        Self: Sized;
}
