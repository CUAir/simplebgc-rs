#[macro_use]
pub(crate) mod macros;
pub(crate) mod constants;

mod board_info;
mod control;
mod get_angles;
mod motors_off;
mod read_params;

pub use self::board_info::*;
pub use self::control::*;
pub use self::get_angles::*;
pub use self::motors_off::*;
pub use self::read_params::*;

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

roll_pitch_yaw!(u8, 1);
roll_pitch_yaw!(i8, 1);

#[derive(Clone, Debug, PartialEq)]
pub enum IncomingCommand {
    BoardInfo(BoardInfo),
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
    ReadParams(ParamsQuery),
    ReadParams3(ParamsQuery),
    ReadParamsExt(ParamsQuery),
    ReadParamsExt2(ParamsQuery),
    ReadParamsExt3(ParamsQuery),
    WriteParams(Params3Data),
    WriteParams3(Params3Data),
    GetAngles,
    GetAnglesExt,
    Other { id: u8 },
}
