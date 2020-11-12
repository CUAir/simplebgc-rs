use crate::commands::constants::*;
use crate::payload::*;
use crate::{IncomingCommand, OutgoingCommand};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use thiserror::Error;

#[derive(Error, Clone, Debug, PartialEq)]
pub enum MessageParseError {
    #[error("bad version code")]
    BadVersionCode,
    #[error("bad command id: {id}")]
    BadCommandId { id: u8 },
    #[error("bad header checksum, expected {expected:#X}, got {actual:#X}")]
    BadHeaderChecksum { expected: u8, actual: u8 },
    #[error("bad payload checksum, expected {expected:#X}, got {actual:#X}")]
    BadPayloadChecksum { expected: u16, actual: u16 },
    #[error("there was not enough data in the buffer to read the whole message")]
    InsufficientData,
    #[error(transparent)]
    PayloadParse(#[from] PayloadParseError),
}

pub trait Message {
    fn command_id(&self) -> u8;

    /// Returns a commands ID and a `Bytes` object representing
    /// the bytes of this payload.
    fn to_payload_bytes(&self) -> Bytes;

    fn from_payload_bytes(id: u8, bytes: Bytes) -> Result<Self, MessageParseError>
    where
        Self: Sized;

    fn to_v1_bytes(&self) -> Bytes {
        let cmd = self.command_id();
        let payload = self.to_payload_bytes();
        let mut buf = BytesMut::with_capacity(payload.len() + 8);

        buf.put_u8(0x3E);
        buf.put_u8(cmd);
        buf.put_u8(payload.len() as u8);

        let header_checksum = cmd.wrapping_add(payload.len() as u8);
        let payload_checksum = payload.bytes().iter().fold(0u8, |l, r| l.wrapping_add(*r));

        buf.put_u8(header_checksum);
        buf.put(payload);
        buf.put_u8(payload_checksum);

        buf.freeze()
    }

    fn to_v2_bytes(&self) -> Bytes {
        let cmd = self.command_id();
        let payload = self.to_payload_bytes();
        let mut buf = BytesMut::with_capacity(payload.len() + 8);

        buf.put_u8(0x24);
        buf.put_u8(cmd);
        buf.put_u8(payload.len() as u8);

        let header_checksum = cmd.wrapping_add(payload.len() as u8);
        let payload_checksum = crc16::State::<crc16::ARC>::calculate(&payload[..]);

        buf.put_u8(header_checksum);
        buf.put(payload);
        buf.put_u16_le(payload_checksum);

        buf.freeze()
    }

    /// On success, returns the number of bytes read from the buffer
    fn from_bytes(buf: &[u8]) -> Result<(Self, usize), MessageParseError>
    where
        Self: Sized,
    {
        // use indexing so as not to consume bytes if it's not valid
        match buf[0] {
            0x3E => Message::from_v1_bytes(buf),
            0x24 => Message::from_v2_bytes(buf),
            _ => Err(MessageParseError::BadVersionCode),
        }
    }

    fn from_v1_bytes(buf: &[u8]) -> Result<(Self, usize), MessageParseError>
    where
        Self: Sized,
    {
        // use indexing so as not to consume bytes if it's not valid

        // assume version byte was already checked
        let cmd = buf[1];

        if cmd == 0 {
            return Err(MessageParseError::BadCommandId { id: cmd });
        }

        let payload_len = buf[2] as usize;
        let expected_header_checksum = buf[3];
        let header_checksum = cmd.wrapping_add(payload_len as u8);

        // wrapping_add is the same as modulo 256
        if expected_header_checksum != header_checksum {
            return Err(MessageParseError::BadHeaderChecksum {
                expected: expected_header_checksum,
                actual: header_checksum,
            });
        }

        if buf.len() < 5 + payload_len {
            return Err(MessageParseError::InsufficientData);
        }

        let payload = Bytes::copy_from_slice(&buf[4..4 + payload_len]);
        let expected_payload_checksum = buf[4 + payload_len];
        let payload_checksum = checksum_bgc_v1(&payload[..]);

        if expected_payload_checksum != payload_checksum {
            return Err(MessageParseError::BadPayloadChecksum {
                expected: expected_payload_checksum as u16,
                actual: payload_checksum as u16,
            });
        }

        return Self::from_payload_bytes(cmd, payload).map(|m| (m, payload_len + 5));
    }

    fn from_v2_bytes(buf: &[u8]) -> Result<(Self, usize), MessageParseError>
    where
        Self: Sized,
    {
        // use indexing so as not to consume bytes if it's not valid

        // assume version byte was already checked
        let cmd = buf[1];

        if cmd == 0 {
            return Err(MessageParseError::BadCommandId { id: cmd });
        }

        let payload_len = buf[2] as usize;
        let expected_header_checksum = buf[3];
        let header_checksum = cmd.wrapping_add(payload_len as u8);

        // wrapping_add is the same as modulo 256
        if expected_header_checksum != header_checksum {
            return Err(MessageParseError::BadHeaderChecksum {
                expected: expected_header_checksum,
                actual: header_checksum,
            });
        }

        if buf.len() < 6 + payload_len {
            return Err(MessageParseError::InsufficientData);
        }

        let payload = Bytes::copy_from_slice(&buf[4..4 + payload_len]);

        let expected_checksum = u16::from_le_bytes([buf[4 + payload_len], buf[5 + payload_len]]);
        let checksum = checksum_bgc_v2(&buf[1..4 + payload_len]);

        if expected_checksum != checksum {
            return Err(MessageParseError::BadPayloadChecksum {
                expected: expected_checksum,
                actual: checksum,
            });
        }

        return Self::from_payload_bytes(cmd, payload).map(|m| (m, payload_len + 6));
    }
}

fn checksum_bgc_v1(buf: &[u8]) -> u8 {
    buf.iter().fold(0u8, |l, r| l.wrapping_add(*r))
}

fn checksum_bgc_v2(buf: &[u8]) -> u16 {
    const POLYNOM: u16 = 0x8005;
    let mut crc = 0;

    for &byte in buf.iter() {
        let mut shift_register = 1;
        while shift_register > 0 {
            let data_bit = byte & shift_register != 0;
            let crc_bit = (crc >> 15) != 0;
            crc <<= 1;

            if data_bit != crc_bit {
                crc ^= POLYNOM;
            }

            shift_register <<= 1;
        }
    }

    crc
}

impl Message for OutgoingCommand {
    fn command_id(&self) -> u8 {
        use OutgoingCommand::*;
        match self {
            BoardInfo => CMD_BOARD_INFO,
            BoardInfo3 => CMD_BOARD_INFO_3,
            Control { .. } => CMD_CONTROL,
            MotorsOn => CMD_MOTORS_ON,
            MotorsOff { .. } => CMD_MOTORS_OFF,
            ReadParams { .. } => CMD_READ_PARAMS,
            ReadParams3 { .. } => CMD_READ_PARAMS_3,
            ReadParamsExt { .. } => CMD_READ_PARAMS_EXT,
            ReadParamsExt2 { .. } => CMD_READ_PARAMS_EXT2,
            ReadParamsExt3 { .. } => CMD_READ_PARAMS_EXT3,
            WriteParams(_) => CMD_WRITE_PARAMS,
            WriteParams3(_) => CMD_WRITE_PARAMS_3,
            GetAngles => CMD_GET_ANGLES,
            GetAnglesExt => CMD_GET_ANGLES,
            _ => unimplemented!(),
        }
    }

    fn to_payload_bytes(&self) -> Bytes {
        use OutgoingCommand::*;
        match self {
            BoardInfo => Bytes::default(),
            BoardInfo3 => Bytes::default(),
            Control(data) => Payload::to_bytes(data),
            MotorsOn => Bytes::default(),
            MotorsOff(data) => Payload::to_bytes(data),
            ReadParams(data) => Payload::to_bytes(data),
            ReadParams3(data) => Payload::to_bytes(data),
            ReadParamsExt(data) => Payload::to_bytes(data),
            ReadParamsExt2(data) => Payload::to_bytes(data),
            ReadParamsExt3(data) => Payload::to_bytes(data),
            WriteParams(data) => Payload::to_bytes(data),
            WriteParams3(data) => Payload::to_bytes(data),
            GetAngles => Bytes::default(),
            GetAnglesExt => Bytes::default(),
            Other { id: _ } => Bytes::default(),
        }
    }

    fn from_payload_bytes(id: u8, bytes: Bytes) -> Result<Self, MessageParseError>
    where
        Self: Sized,
    {
        use OutgoingCommand::*;

        Ok(match id {
            CMD_READ_PARAMS => ReadParams(Payload::from_bytes(bytes)?),
            CMD_READ_PARAMS_3 => ReadParams3(Payload::from_bytes(bytes)?),
            CMD_READ_PARAMS_EXT => ReadParamsExt(Payload::from_bytes(bytes)?),
            CMD_READ_PARAMS_EXT2 => ReadParamsExt2(Payload::from_bytes(bytes)?),
            CMD_READ_PARAMS_EXT3 => ReadParamsExt3(Payload::from_bytes(bytes)?),
            CMD_WRITE_PARAMS => WriteParams(Payload::from_bytes(bytes)?),
            CMD_WRITE_PARAMS_3 => WriteParams3(Payload::from_bytes(bytes)?),
            CMD_GET_ANGLES => GetAngles,
            CMD_GET_ANGLES_EXT => GetAnglesExt,
            CMD_CONTROL => Control(Payload::from_bytes(bytes)?),
            CMD_MOTORS_ON => MotorsOn,
            CMD_MOTORS_OFF => MotorsOff(Payload::from_bytes(bytes)?),
            _ => return Err(MessageParseError::BadCommandId { id }),
        })
    }
}

impl Message for IncomingCommand {
    fn command_id(&self) -> u8 {
        match self {
            IncomingCommand::BoardInfo(_) => CMD_BOARD_INFO,
            IncomingCommand::GetAngles(_) => CMD_GET_ANGLES,
            IncomingCommand::ReadParams(_) => CMD_READ_PARAMS,
            IncomingCommand::ReadParams3(_) => CMD_READ_PARAMS_3,
        }
    }

    fn to_payload_bytes(&self) -> Bytes {
        use IncomingCommand::*;
        match self {
            BoardInfo(info) => Payload::to_bytes(info),
            GetAngles(angles) => Payload::to_bytes(angles),
            ReadParams(params) => Payload::to_bytes(params),
            ReadParams3(params) => Payload::to_bytes(params),
        }
    }

    fn from_payload_bytes(id: u8, bytes: Bytes) -> Result<Self, MessageParseError>
    where
        Self: Sized,
    {
        use IncomingCommand::*;

        Ok(match id {
            CMD_BOARD_INFO => BoardInfo(Payload::from_bytes(bytes)?),
            CMD_GET_ANGLES => BoardInfo(Payload::from_bytes(bytes)?),
            CMD_READ_PARAMS => BoardInfo(Payload::from_bytes(bytes)?),
            CMD_READ_PARAMS_3 => BoardInfo(Payload::from_bytes(bytes)?),
            _ => return Err(MessageParseError::BadCommandId { id }),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{Message, OutgoingCommand, ParamsQuery};
    use std::error::Error;

    #[test]
    fn sanity() -> Result<(), Box<dyn Error>> {
        let packet = [0x3E, 0x52, 0x01, 0x53, 0x01, 0x01];
        let (msg, read) = OutgoingCommand::from_bytes(&packet[..])?;

        assert_eq!(read, 6, "should have read 6 bytes");
        assert_eq!(
            msg,
            OutgoingCommand::ReadParams(ParamsQuery { profile_id: 1 })
        );

        Ok(())
    }
}
