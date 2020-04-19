use crate::commands::constants::*;
use crate::payload::*;
use crate::{OutgoingCommand, Params3Data};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::hash::Hasher;

#[derive(Clone, Debug, PartialEq)]
pub enum MessageParseError {
    BadVersionCode,
    BadHeaderChecksum { expected: u8, actual: u8 },
    BadPayloadChecksum { expected: u16, actual: u16 },
    InsufficientData,
    PayloadParse(PayloadParseError),
}

impl Display for MessageParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageParseError::BadVersionCode => write!(f, "bad version code"),
            MessageParseError::BadHeaderChecksum { expected, actual } => write!(
                f,
                "bad header checksum, expected {:#X}, got {:#X}",
                expected, actual
            ),
            MessageParseError::BadPayloadChecksum { expected, actual } => write!(
                f,
                "bad payload checksum, expected {:#X}, got {:#X}",
                expected, actual
            ),
            MessageParseError::InsufficientData => write!(f, "insufficient data"),
            MessageParseError::PayloadParse(e) => {
                write!(f, "payload could not be parsed: {:#?}", e)
            }
        }
    }
}

impl Error for MessageParseError {}

impl From<PayloadParseError> for MessageParseError {
    fn from(e: PayloadParseError) -> Self {
        MessageParseError::PayloadParse(e)
    }
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

        // assume 1st byte was already checked and removed
        let cmd = buf[1];
        let len = buf[2] as usize;
        let expected_header_checksum = buf[3];
        let header_checksum = cmd.wrapping_add(len as u8);

        // wrapping_add is the same as modulo 256
        if expected_header_checksum != header_checksum {
            return Err(MessageParseError::BadHeaderChecksum {
                expected: expected_header_checksum,
                actual: header_checksum,
            });
        }

        if buf.len() < 5 + len {
            return Err(MessageParseError::InsufficientData);
        }

        let payload = Bytes::copy_from_slice(&buf[4..4 + len]);
        let expected_payload_checksum = buf[4 + len];
        let payload_checksum = checksum_bgc(&payload[..]);

        if expected_payload_checksum != payload_checksum {
            return Err(MessageParseError::BadPayloadChecksum {
                expected: expected_payload_checksum as u16,
                actual: payload_checksum as u16,
            });
        }

        return Self::from_payload_bytes(cmd, payload).map(|m| (m, len + 5));
    }

    fn from_v2_bytes(buf: &[u8]) -> Result<(Self, usize), MessageParseError>
    where
        Self: Sized,
    {
        // use indexing so as not to consume bytes if it's not valid

        // assume 1st byte was already checked and removed
        let cmd = buf[1];
        let len = buf[2] as usize;
        let expected_header_checksum = buf[3];
        let header_checksum = cmd.wrapping_add(len as u8);

        // wrapping_add is the same as modulo 256
        if expected_header_checksum != header_checksum {
            return Err(MessageParseError::BadHeaderChecksum {
                expected: expected_header_checksum,
                actual: header_checksum,
            });
        }

        if buf.len() < 5 + len {
            return Err(MessageParseError::InsufficientData);
        }

        let payload = Bytes::copy_from_slice(&buf[4..4 + len]);
        let expected_payload_checksum = u16::from_le_bytes([buf[4 + len], buf[5 + len]]);

        let payload_checksum = crc16::State::<crc16::ARC>::calculate(&payload[..]);

        if expected_payload_checksum != payload_checksum {
            return Err(MessageParseError::BadPayloadChecksum {
                expected: expected_payload_checksum,
                actual: payload_checksum,
            });
        }

        return Self::from_payload_bytes(cmd, payload).map(|m| (m, len + 6));
    }
}

fn checksum_bgc(buf: &[u8]) -> u8 {
    buf.iter().fold(0u8, |l, r| l.wrapping_add(*r))
}

impl Message for OutgoingCommand {
    fn command_id(&self) -> u8 {
        match self {
            OutgoingCommand::Control { .. } => CMD_CONTROL,
            OutgoingCommand::MotorsOn => CMD_MOTORS_ON,
            OutgoingCommand::MotorsOff { .. } => CMD_MOTORS_OFF,
            OutgoingCommand::ReadParams { .. } => CMD_READ_PARAMS,
            OutgoingCommand::ReadParams3 { .. } => CMD_READ_PARAMS_3,
            OutgoingCommand::ReadParamsExt { .. } => CMD_READ_PARAMS_EXT,
            OutgoingCommand::ReadParamsExt2 { .. } => CMD_READ_PARAMS_EXT2,
            OutgoingCommand::ReadParamsExt3 { .. } => CMD_READ_PARAMS_EXT3,
            OutgoingCommand::WriteParams(_) => CMD_WRITE_PARAMS,
            OutgoingCommand::WriteParams3(_) => CMD_WRITE_PARAMS_3,
            OutgoingCommand::GetAngles => CMD_GET_ANGLES,
            OutgoingCommand::GetAnglesExt => CMD_GET_ANGLES,
            _ => unimplemented!(),
        }
    }

    fn to_payload_bytes(&self) -> Bytes {
        unimplemented!()
    }

    fn from_payload_bytes(id: u8, mut bytes: Bytes) -> Result<Self, MessageParseError>
    where
        Self: Sized,
    {
        use OutgoingCommand::*;

        Ok(match id {
            CMD_READ_PARAMS => ReadParams {
                profile_id: bytes.get_u8(),
            },
            CMD_READ_PARAMS_3 => ReadParams3 {
                profile_id: bytes.get_u8(),
            },
            CMD_READ_PARAMS_EXT => ReadParamsExt {
                profile_id: bytes.get_u8(),
            },
            CMD_READ_PARAMS_EXT2 => ReadParamsExt2 {
                profile_id: bytes.get_u8(),
            },
            CMD_READ_PARAMS_EXT3 => ReadParamsExt3 {
                profile_id: bytes.get_u8(),
            },
            CMD_WRITE_PARAMS => WriteParams(Payload::from_bytes(bytes)?),
            CMD_WRITE_PARAMS_3 => WriteParams3(Payload::from_bytes(bytes)?),
            CMD_GET_ANGLES => GetAngles,
            CMD_GET_ANGLES_EXT => GetAnglesExt,
            CMD_CONTROL => Control(Payload::from_bytes(bytes)?),
            CMD_MOTORS_ON => MotorsOn,
            CMD_MOTORS_OFF => MotorsOff {
                mode: if bytes.remaining() > 0 {
                    Some(read_enum!(bytes, "MODE", u8)?)
                } else {
                    None
                },
            },
            _ => Other { id },
        })
    }
}

mod tests {
    use crate::commands::constants::CMD_READ_PROFILE_NAMES;
    use crate::ProfileFlags::OuterMotorLimitFreeRotation;
    use crate::{Message, OutgoingCommand};
    use std::error::Error;

    #[test]
    fn sanity() -> Result<(), Box<dyn Error>> {
        let packet = [0x3E, 0x52, 0x01, 0x53, 0x01, 0x01];
        let (msg, read) = OutgoingCommand::from_bytes(&packet[..])?;

        assert_eq!(read, 6, "should have read 6 bytes");
        assert_eq!(msg, OutgoingCommand::ReadParams { profile_id: 1 });

        let packet = [0x24, 0x52, 0x01, 0x53, 0x01, 0xC1, 0xC0];
        let (msg, read) = OutgoingCommand::from_bytes(&packet[..])?;

        assert_eq!(read, 7, "should have read 6 bytes");
        assert_eq!(msg, OutgoingCommand::ReadParams { profile_id: 1 });

        Ok(())
    }
}
