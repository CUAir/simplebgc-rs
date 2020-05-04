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
use bytes::{BufMut, Bytes, BytesMut};

#[derive(Clone, Debug, PartialEq)]
pub struct RollPitchYaw<T> {
    pub roll: T,
    pub pitch: T,
    pub yaw: T,
}

impl<T> RollPitchYaw<T> {
    pub fn combine<U>(self, other: RollPitchYaw<U>) -> RollPitchYaw<(T, U)> {
        RollPitchYaw {
            roll: (self.roll, other.roll),
            pitch: (self.pitch, other.pitch),
            yaw: (self.yaw, other.yaw),
        }
    }

    pub fn combine_ref<'a, U>(&self, other: &'a RollPitchYaw<U>) -> RollPitchYaw<(&T, &'a U)> {
        RollPitchYaw {
            roll: (&self.roll, &other.roll),
            pitch: (&self.pitch, &other.pitch),
            yaw: (&self.yaw, &other.yaw),
        }
    }

    pub fn combine_mut<'a, U>(
        &mut self,
        other: &'a mut RollPitchYaw<U>,
    ) -> RollPitchYaw<(&mut T, &'a mut U)> {
        RollPitchYaw {
            roll: (&mut self.roll, &mut other.roll),
            pitch: (&mut self.pitch, &mut other.pitch),
            yaw: (&mut self.yaw, &mut other.yaw),
        }
    }

    pub fn map<U, F: Fn(T) -> U>(self, op: F) -> RollPitchYaw<U> {
        RollPitchYaw {
            roll: op(self.roll),
            pitch: op(self.pitch),
            yaw: op(self.yaw),
        }
    }

    pub fn update<U, F: Fn(&mut T) -> U>(&mut self, op: F) -> RollPitchYaw<U> {
        RollPitchYaw {
            roll: op(&mut self.roll),
            pitch: op(&mut self.pitch),
            yaw: op(&mut self.yaw),
        }
    }

    pub fn exec<U, F: Fn(&T) -> U>(&self, op: F) {
        op(&self.roll);
        op(&self.pitch);
        op(&self.yaw);
    }
}

impl<T> Into<(T, T, T)> for RollPitchYaw<T> {
    fn into(self) -> (T, T, T) {
        (self.roll, self.pitch, self.yaw)
    }
}

impl<T> From<(T, T, T)> for RollPitchYaw<T> {
    fn from(t: (T, T, T)) -> Self {
        RollPitchYaw {
            roll: t.0,
            pitch: t.1,
            yaw: t.2,
        }
    }
}

impl<T: Copy> Copy for RollPitchYaw<T> {}

rpy_payload!(u8, 1);
rpy_payload!(i8, 1);

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
