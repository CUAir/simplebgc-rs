use crate::commands::constants::*;
use crate::payload::*;
use crate::{OutgoingCommand, Params3Data};
use bytes::{Buf, BufMut, Bytes, BytesMut};

pub enum MessageParseError {
    BadVersionCode,
    BadHeaderChecksum,
    BadPayloadChecksum,
    PayloadParse(PayloadParseError),
}

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
        use crc::crc16::checksum_x25;

        let cmd = self.command_id();
        let payload = self.to_payload_bytes();
        let mut buf = BytesMut::with_capacity(payload.len() + 8);

        buf.put_u8(0x24);
        buf.put_u8(cmd);
        buf.put_u8(payload.len() as u8);

        let header_checksum = cmd.wrapping_add(payload.len() as u8);
        let payload_checksum = checksum_x25(&payload[..]);

        buf.put_u8(header_checksum);
        buf.put(payload);
        buf.put_u16(payload_checksum);

        buf.freeze()
    }

    fn from_bytes(buf: &mut dyn Buf) -> Result<Self, MessageParseError>
    where
        Self: Sized,
    {
        match buf.get_u8() {
            0x3E => Message::from_v1_bytes(buf),
            0x24 => Message::from_v2_bytes(buf),
            _ => Err(MessageParseError::BadVersionCode),
        }
    }

    fn from_v1_bytes(buf: &mut dyn Buf) -> Result<Self, MessageParseError>
    where
        Self: Sized,
    {
        // assume 1st byte was already checked

        let cmd = buf.get_u8();
        let len = buf.get_u8() as usize;
        let header_checksum = buf.get_u8();

        // wrapping_add is the same as modulo 256
        if header_checksum != cmd.wrapping_add(len as u8) {
            return Err(MessageParseError::BadHeaderChecksum);
        }

        let mut payload = BytesMut::with_capacity(len);
        buf.copy_to_slice(&mut payload[..]);
        unsafe {
            payload.set_len(len);
        }

        let payload_checksum = buf.get_u8();

        if payload_checksum != payload.bytes().iter().fold(0u8, |l, r| l.wrapping_add(*r)) {
            return Err(MessageParseError::BadPayloadChecksum);
        }

        return Self::from_payload_bytes(cmd, payload.freeze());
    }

    fn from_v2_bytes(buf: &mut dyn Buf) -> Result<Self, MessageParseError>
    where
        Self: Sized,
    {
        // assume 1st byte was already checked

        let cmd = buf.get_u8();
        let len = buf.get_u8() as usize;
        let header_checksum = buf.get_u8();

        // overflowing_add is the same as modulo 256
        if header_checksum != cmd.wrapping_add(len as u8) {
            return Err(MessageParseError::BadHeaderChecksum);
        }

        let mut payload = BytesMut::with_capacity(len);
        buf.copy_to_slice(&mut payload[..]);
        unsafe {
            payload.set_len(len);
        }

        let payload_checksum = buf.get_u8();

        if payload_checksum != payload.bytes().iter().fold(0u8, |l, r| l.wrapping_add(*r)) {
            return Err(MessageParseError::BadPayloadChecksum);
        }

        return Self::from_payload_bytes(cmd, payload.freeze());
    }
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
            _ => Other {
                id,
                data: bytes.to_bytes(),
            },
        })
    }
}
