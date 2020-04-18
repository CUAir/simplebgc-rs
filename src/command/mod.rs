mod control;
mod read_params_3;
mod motors_off;

pub use self::control::*;
pub use self::read_params_3::*;
pub use self::motors_off::*;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::convert::{TryFrom, TryInto};

pub struct Version {
    major: u8,
    minor: u8,
    beta: u8,
}

#[derive(BitFlags, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum StateFlags1 {
    DebugMode = 1 << 0,
    IsFrameInverted = 1 << 1,
    InitStep1Done = 1 << 2,
    InitStep2Done = 1 << 3,
    StartupAutoRoutineDone = 1 << 4,
}

#[derive(BitFlags, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum BoardFeatures {
    ThreeAxis = 1 << 0,
    BatMonitoring = 1 << 1,
    Encoders = 1 << 2,
    BodeTest = 1 << 3,
    Scripting = 1 << 4,
    CurrentSensor = 1 << 5,
}

#[derive(BitFlags, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum ConnectionFlag {
    USB = 1 << 0,
}

pub enum IncomingCommand {
    BoardInfo {
        board_version: Version,
        firmware_version: Version,
        state: StateFlags1,
        connection_flag: ConnectionFlag,
        frw_extra_id: u32,
        reserved: [u8; 7],
    },
    GetAngles {
        /// Imu angles in 14-bit resolution per full turn
        /// Units: 0,02197265625 degree
        imu_angle: i32,

        /// Target angles in 14-bit resolution per full turn
        /// Units: 0,02197265625 degree
        target_angle: i32,

        /// Target speed that gimbal should keep, over Euler axes
        /// Units: 0,1220740379 degree/sec
        target_speed: i32,
    },
    ReadParams3(Params3Data),
}



pub enum OutgoingCommand {
    Control {
        mode: ControlMode,
        axes: (ControlAxisParams, ControlAxisParams, ControlAxisParams),
    },
    MotorsOn,
    MotorsOff {
        mode: Option<MotorsOffMode>
    },
    WriteParams3(Param3Data)
}

pub trait Message {
    fn command_id(&self) -> u8;

    fn to_payload_bytes(&self) -> Bytes;

    fn from_payload_bytes<T: Buf>(bytes: &mut T) -> Option<Self>
    where
        Self: Sized;

    fn to_v1_bytes(&self) -> Bytes {
        let cmd = self.command_id();
        let payload = self.payload();
        let mut buf = BytesMut::with_capacity(payload.len() + 8);

        buf.put_u8(0x3E);
        buf.put_u8(cmd);
        buf.put_u8(payload.len() as u8);

        let header_checksum = (cmd + payload.len() as u8) % 256;
        let payload_checksum = payload.iter().sum() % 256;

        buf.put_u8(header_checksum);
        buf.put(payload);
        buf.put_u8(payload_checksum);

        buf.freeze()
    }

    fn to_v2_bytes(&self) -> Bytes {
        use crc::crc16::checksum_x25;

        let cmd = self.command_id();
        let payload = self.payload();
        let mut buf = BytesMut::with_capacity(payload.len() + 8);

        buf.put_u8(0x24);
        buf.put_u8(cmd);
        buf.put_u8(payload.len() as u8);

        let header_checksum = (cmd + payload.len() as u8) % 256;
        let payload_checksum = checksum_x25(&payload[..]);

        buf.put_u8(header_checksum);
        buf.put(payload);
        buf.put_u16(payload_checksum);

        buf.freeze()
    }

    fn from_bytes<T: Buf>(buf: &mut T) -> Option<Self>
    where
        Self: Sized,
    {
        match buf[0] {
            0x3E => Message::from_v1_bytes(buf),
            0x24 => Message::from_v2_bytes(buf),
            _ => None,
        }
    }

    fn from_v1_bytes<T: Buf>(buf: &mut T) -> Option<Self>
    where
        Self: Sized,
    {
        // assume 1st byte was already checked
        buf.advance(1);

        let cmd = buf.get_u8();
        let len = buf.get_u8();
        let header_checksum = buf.get_u8();

        if header_checksum != (cmd + len) % 256 {
            return None;
        }

        let mut payload = Bytes::copy_from_slice(&buf[..len]);
        buf.advance(len as usize);

        let payload_checksum = buf.get_u8();

        if payload_checksum != payload.iter().sum() % 256 {
            return None;
        }

        return Message::from_payload_bytes(&mut payload);
    }

    fn from_v2_bytes<T: Buf>(buf: &mut T) -> Option<Self>
    where
        Self: Sized,
    {
        // assume 1st byte was already checked
        buf.advance(1);

        let cmd = buf.get_u8();
        let len = buf.get_u8();
        let header_checksum = buf.get_u8();

        if header_checksum != (cmd + len) % 256 {
            return None;
        }

        let mut payload = Bytes::copy_from_slice(&buf[..len]);
        buf.advance(len as usize);

        let payload_checksum = buf.get_u8();

        if payload_checksum != payload.iter().sum() % 256 {
            return None;
        }

        return Message::from_payload_bytes(&mut payload);
    }
}
