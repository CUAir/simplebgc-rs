use bytes::{Buf, Bytes};
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
    /// Parses this payload from bytes according to the SimpleBGC spec.
    fn from_bytes(b: Bytes) -> Result<Self, PayloadParseError>
    where
        Self: Sized;

    /// Converts this payload to bytes according to the SimpleBGC spec.
    fn to_bytes(&self) -> Bytes
    where
        Self: Sized;
}

impl Payload for u8 {
    fn from_bytes(mut b: Bytes) -> Result<Self, PayloadParseError>
    where
        Self: Sized,
    {
        Ok(b.get_u8())
    }

    fn to_bytes(&self) -> Bytes
    where
        Self: Sized,
    {
        Bytes::copy_from_slice(&[*self])
    }
}

impl Payload for i8 {
    fn from_bytes(mut b: Bytes) -> Result<Self, PayloadParseError>
    where
        Self: Sized,
    {
        Ok(b.get_i8())
    }

    fn to_bytes(&self) -> Bytes
    where
        Self: Sized,
    {
        Bytes::copy_from_slice(&[*self as u8])
    }
}
