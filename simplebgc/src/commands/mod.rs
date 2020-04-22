#[macro_use]
pub(crate) mod macros;
pub(crate) mod constants;

mod board_info;
mod control;
mod motors_off;
mod read_params_3;

pub use self::board_info::*;
pub use self::control::*;
pub use self::motors_off::*;
pub use self::read_params_3::*;

use crate::{Payload, PayloadParseError};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use enumflags2::BitFlags;

#[derive(Clone, Debug, PartialEq)]
pub struct RollPitchYaw<T: Payload> {
    roll: T,
    pitch: T,
    yaw: T,
}

impl<T: Payload + Copy> Copy for RollPitchYaw<T> {}

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
    BoardInfo(BoardInfo),
    GetAngles(AngleInfo),
    BoardInfo {
        board_version: Version,
        firmware_version: Version,
        state: StateFlags1,
        connection_flag: ConnectionFlag,
        frw_extra_id: u32,
        reserved: [u8; 7],
    },
    GetAngles(RollPitchYaw<AngleInfo>),
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
