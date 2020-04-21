#[macro_use]
pub(crate) mod macros;
pub(crate) mod constants;

mod control;
mod motors_off;
mod read_params_3;

pub use self::control::*;
pub use self::motors_off::*;
pub use self::read_params_3::*;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use enumflags2::BitFlags;
use std::convert::{TryFrom, TryInto};

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

#[derive(Copy, Clone, Debug, PartialEq)]
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
        roll_imu_angle: i32,

        /// Target angles in 14-bit resolution per full turn
        /// Units: 0,02197265625 degree
        roll_target_angle: i32,

        /// Target speed that gimbal should keep, over Euler axes
        /// Units: 0,1220740379 degree/sec
        roll_target_speed: i32,
        
        /// Imu angles in 14-bit resolution per full turn
        /// Units: 0,02197265625 degree
        pitch_imu_angle: i32,

        /// Target angles in 14-bit resolution per full turn
        /// Units: 0,02197265625 degree
        pitch_target_angle: i32,

        /// Target speed that gimbal should keep, over Euler axes
        /// Units: 0,1220740379 degree/sec
        pitch_target_speed: i32,

        /// Imu angles in 14-bit resolution per full turn
        /// Units: 0,02197265625 degree
        yaw_imu_angle: i32,

        /// Target angles in 14-bit resolution per full turn
        /// Units: 0,02197265625 degree
        yaw_target_angle: i32,

        /// Target speed that gimbal should keep, over Euler axes
        /// Units: 0,1220740379 degree/sec
        yaw_target_speed: i32,
    },
    ReadParams(Params3Data),
    ReadParams3(Params3Data),
}

#[derive(Copy, Clone, Debug, PartialEq)]
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
