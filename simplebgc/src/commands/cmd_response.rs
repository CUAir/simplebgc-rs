use crate::{Payload, PayloadParseError};
use bytes::{Buf, BufMut, Bytes, BytesMut};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Confirm {
    pub cmd_id: u8,
    pub data: DataType,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DataType {
    None,
    DataU8(u8),
    DataU16(u16),
}

impl Payload for Confirm {
    fn from_bytes(mut b: Bytes) -> Result<Self, PayloadParseError>
    where
        Self: Sized,
    {
        Ok(Confirm {
            cmd_id: read_enum!(b, "CMD_ID", u8)?,
            data: if b.remaining() == 0 {
                DataType::None
            } else if b.remaining() == 1 {
                DataType::DataU8(read_enum!(b, "DATA", u8)?)
            } else if b.remaining() == 2 {
                DataType::DataU16(read_enum!(b, "DATA", u16)?)
            } else {
                panic!("Unexpected amount of remaining bytes. Expected 0, 1, or 2, got {}", b.remaining())
            },
        })
    }

    fn to_bytes(&self) -> Bytes
    where
        Self: Sized,
    {
        let b = match self.data {
            DataType::None => {
                let mut b = BytesMut::with_capacity(1);
                b.put_u8(self.cmd_id);
                b
            }
            DataType::DataU8(data_raw) => {
                let mut b = BytesMut::with_capacity(2);
                b.put_u8(self.cmd_id);
                b.put_u8(data_raw);
                b
            },
            DataType::DataU16(data_raw) => {
                let mut b = BytesMut::with_capacity(3);
                b.put_u8(self.cmd_id);
                b.put_u16(data_raw);
                b
            }
        };

        b.freeze()
    }
}

// #[derive(BgcPayload, Copy, Clone, Debug, PartialEq)]
// pub struct Error {

// }