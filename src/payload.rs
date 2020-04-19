use bytes::{Buf, Bytes};

#[derive(Clone, Debug, PartialEq)]
pub enum PayloadParseError {
    InvalidFlags { name: String },
    InvalidEnum { name: String },
}

pub trait Payload {
    fn from_bytes(b: Bytes) -> Result<Self, PayloadParseError>
    where
        Self: Sized;

    fn to_bytes(&self) -> Bytes
    where
        Self: Sized;
}
