#[macro_use]
pub(crate) mod macros;
pub(crate) mod constants;

mod control;
mod motors_off;
mod read_params_3;

pub use self::control::*;
pub use self::motors_off::*;
pub use self::read_params_3::*;

use crate::{Payload, PayloadParseError};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use enumflags2::BitFlags;

#[derive(Copy, Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct RollPitchYaw<T: Payload> {
    roll: T,
    pitch: T,
    yaw: T,
}

impl <T: Payload + Copy> Copy for RollPitchYaw<T> {}

#[derive(BgcPayload, Copy, Clone, Debug, PartialEq)]
pub struct AngleInfo {
    /// Imu angles in 14-bit resolution per full turn
    /// Units: 0,02197265625 degree
    #[bgc_raw("IMU_ANGLE")]
    imu_angle: i32,

    /// Target angles in 14-bit resolution per full turn
    /// Units: 0,02197265625 degree
    #[bgc_raw("TARGET_ANGLE")]
    target_angle: i32,
}

roll_pitch_yaw!(AngleInfo, 4);

#[derive(Clone, Debug, PartialEq)]
pub enum IncomingCommand {
    BoardInfo {
        board_version: Version,
        firmware_version: Version,
        state: StateFlags1,
        connection_flag: ConnectionFlag,
        frw_extra_id: u32,
        reserved: [u8; 7],
    },
    GetAngles(AngleInfo),
    ReadParams(Params3Data),
    ReadParams3(Params3Data),
}

#[derive(Clone, Debug, PartialEq)]
pub enum OutgoingCommand {
    BoardInfo,
    BoardInfo3,
    Control(ControlData),
    MotorsOn,
    MotorsOff { mode: Option<MotorsOffMode> },
    ReadParams { profile_id: u8 },
    ReadParams3 { profile_id: u8 },
    ReadParamsExt { profile_id: u8 },
    ReadParamsExt2 { profile_id: u8 },
    ReadParamsExt3 { profile_id: u8 },
    WriteParams(Params3Data),
    WriteParams3(Params3Data),
    GetAngles,
    GetAnglesExt,
    Other { id: u8 },
}
